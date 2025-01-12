// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

use crate::{
	f64x2,
	replace_float_literals,
	Component,
	ComponentMeta,
	SimdFloat,
};

/// A subtle tube saturation [`Component`].
///
/// Ported from [AirWindows' Tube2 plugin](https://www.airwindows.com/tube2/), but with SIMD.
#[derive(Debug, Clone)]
pub struct Tube<T> {
	asym_pad: T,
	overall_scale: f64,
	output_scaling: T,
	gain_scaling: T,
	power_factor: i64,

	prev_a: T,
	prev_c: T,
	prev_e: T,
}

impl<T: SimdFloat> Tube<T> {
	/// Create a new [`Tube`].
	///
	/// Panics on debug builds if `sample_rate <= 0.0`.
	#[replace_float_literals(T)]
	pub fn new(sample_rate: f64) -> Self {
		nih_plug::nih_debug_assert!(sample_rate > 0f64);

		let mut x = Self {
			prev_a: 0.0,
			prev_c: 0.0,
			prev_e: 0.0,

			overall_scale: 1_f64 / 44100_f64 * sample_rate,
			// These will be adjusted by set_amount()
			asym_pad: 0.0,
			power_factor: 0,
			gain_scaling: 0.0,
			output_scaling: 0.0,
		};

		x.set_amount(0_f64);
		x
	}

	/// Set tube amount, from `0.0` to `1.0`.
	///
	/// Panics on debug builds if `amount` is outside the range `[0..1]`.
	pub fn set_amount(&mut self, amount: f64) {
		nih_plug::nih_debug_assert!((0.0..=1.0).contains(&amount));

		let iterations = 1.0 - amount;
		let power_factor = (9.0 * iterations) as i64 + 1;

		self.power_factor = power_factor;
		self.asym_pad = T::splat(power_factor as f64);
		self.gain_scaling = T::splat(1.0 / (power_factor + 1) as f64);
		self.output_scaling = T::splat(1.0 + (1.0 / power_factor as f64));
	}

	/// See [`Tube::set_amount`].
	pub fn with_amount(mut self, amount: f64) -> Self {
		self.set_amount(amount);
		self
	}
}

impl<T: SimdFloat> ComponentMeta for Tube<T> {
	fn reset(&mut self) {
		self.prev_a = T::ZERO;
		self.prev_c = T::ZERO;
		self.prev_e = T::ZERO;
	}
}

impl Component<f64> for Tube<f64> {
	fn process(&mut self, mut sample: f64) -> f64 {
		// Originally there's an input gain parameter that starts off at 0.5.
		// It makes up for the level boost the tube causes, so I prefer it to remain constant as this is a Component.
		sample *= 0.5;

		//for high sample rates on this plugin we are going to do a simple average
		if self.overall_scale > 1.9 {
			let stored = sample;

			sample += self.prev_a;
			self.prev_a = stored;
			sample *= 0.5;
		}

		sample = sample.clamp(-1.0, 1.0);

		//flatten bottom, point top of sine waveshaper L
		sample /= self.asym_pad;
		let mut sharpen = -sample;

		if sharpen > 0.0 {
			sharpen = 1.0 + f64::sqrt(sharpen);
		} else {
			sharpen = 1.0 - f64::sqrt(-sharpen);
		}
		sample -= sample * f64::abs(sample) * sharpen * 0.25;
		//this will take input from exactly -1.0 to 1.0 max
		sample *= self.asym_pad;

		// [Right channel code omitted]
		//end first asym section: later boosting can mitigate the extreme
		//softclipping of one side of the wave
		//and we are asym clipping more when Tube is cranked, to compensate

		//original Tube algorithm: power_factor widens the more linear region of the wave
		let mut factor = sample;
		for _ in 0..self.power_factor {
			factor *= sample;
		}

		if self.power_factor % 2 == 1 && sample != 0.0 {
			factor = (factor / sample) * f64::abs(sample);
		}
		factor *= self.gain_scaling;
		sample -= factor;
		sample *= self.output_scaling;

		// [Right channel code omitted]

		//for high sample rates on this plugin we are going to do a simple average
		if self.overall_scale > 1.9 {
			let stored = sample;
			sample += self.prev_c;
			self.prev_c = stored;
			sample *= 0.5;
		}
		//end original Tube. Now we have a boosted fat sound peaking at 0dB exactly

		//hysteresis and spiky fuzz
		let mut slew = self.prev_e - sample;

		if self.overall_scale > 1.9 {
			let stored = sample;
			sample += self.prev_e;
			self.prev_e = stored;
			sample *= 0.5;
		} else {
			self.prev_e = sample; //for this, need self.prev_c always
		}

		if slew > 0.0 {
			slew = 1.0 + (f64::sqrt(slew) * 0.5);
		} else {
			slew = 1.0 - (f64::sqrt(-slew) * 0.5);
		}

		//reusing self.gain_scaling that's part of another algorithm
		sample -= sample * f64::abs(sample) * slew * self.gain_scaling;

		sample = sample.clamp(-0.52, 0.52);
		sample *= 1.923076923076923;
		//end hysteresis and spiky fuzz section

		// [Dither code omitted]

		sample
	}
}

impl Component<f64x2> for Tube<f64x2> {
	#[replace_float_literals(f64x2)]
	fn process(&mut self, mut sample: f64x2) -> f64x2 {
		// Originally there's an input gain parameter that starts off at 0.5.
		// It makes up for the level boost the tube causes, so I prefer it to remain constant as this is a Component.
		sample *= 0.5;

		//for high sample rates on this plugin we are going to do a simple average
		if self.overall_scale > 1.9_f64 {
			let stored = sample;

			sample += self.prev_a;
			self.prev_a = stored;
			sample *= 0.5;
		}

		sample = sample.clamp(-1.0, 1.0);

		//flatten bottom, point top of sine waveshaper L
		sample /= self.asym_pad;

		// We have to account for each channel in the f64x2 vector here.
		let mut sharpen = -sample;
		let sharpen_sign = f64x2::copysign(1.0, sharpen);
		sharpen = 1.0 + f64x2::sqrt(sharpen.abs()).copysign(sharpen_sign);

		sample -= sample * f64x2::abs(sample) * sharpen * 0.25;
		//this will take input from exactly -1.0 to 1.0 max
		sample *= self.asym_pad;

		// [Right channel code omitted]
		//end first asym section: later boosting can mitigate the extreme
		//softclipping of one side of the wave
		//and we are asym clipping more when Tube is cranked, to compensate

		//original Tube algorithm: self.power_factor widens the more linear region of the wave

		// Oddly, this loop's about 1% faster than pow:
		// let mut factor = sample.pow(f64x2::splat((self.power_factor) as _));
		let mut factor = sample;
		for _ in 0..self.power_factor {
			factor *= sample;
		}

		if self.power_factor % 2 == 1 {
			if sample.all_ne(0.0) {
				factor = (factor / sample) * f64x2::abs(sample);
			} else {
				// We can't avoid splitting the SIMD vector into individual channels here.
				// This is because at least one value in the vector is zero, and we're dividing by it.
				let [a, b] = sample.value();
				let [fa, fb] = factor.value();

				if a != 0_f64 {
					factor = f64x2::new([(fa / a) * f64::abs(a), fb]);
				} else if b != 0_f64 {
					factor = f64x2::new([fa, (fb / b) * f64::abs(fb)]);
				}
			}
		}

		factor *= self.gain_scaling;
		sample -= factor;
		sample *= self.output_scaling;

		// [Right channel code omitted]

		//for high sample rates on this plugin we are going to do a simple average
		if self.overall_scale > 1.9_f64 {
			let stored = sample;
			sample += self.prev_c;
			self.prev_c = stored;
			sample *= 0.5;
		}
		//end original Tube. Now we have a boosted fat sound peaking at 0dB exactly

		//hysteresis and spiky fuzz
		let mut slew = self.prev_e - sample;

		if self.overall_scale > 1.9_f64 {
			let stored = sample;
			sample += self.prev_e;
			self.prev_e = stored;
			sample *= 0.5;
		} else {
			self.prev_e = sample; //for this, need self.prev_c always
		}

		// Again, we have to mind each channel; that's why we do copysign.
		let slew_sign = f64x2::copysign(1.0, slew);
		slew = 1.0 + (f64x2::sqrt(slew.abs()).copysign(slew_sign) * 0.5);

		//reusing gainscaling that's part of another algorithm
		sample -= sample * f64x2::abs(sample) * slew * self.gain_scaling;

		sample = sample.clamp(-0.52, 0.52);
		sample *= 1.923076923076923;
		//end hysteresis and spiky fuzz section

		// [Dither code omitted]

		sample
	}
}
