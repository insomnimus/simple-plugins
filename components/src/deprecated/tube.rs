// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

//! Port of [Airwindows' Tube plugin](https://github.com/airwindows/airwindows).

use crate::{
	replace_float_literals,
	Component,
	ComponentMeta,
	ScalarFloat,
};

/// A subtle tube saturator.
#[derive(Debug, Clone)]
pub struct Tube<T> {
	overall_scale: T,
	// prev_sample_b and prev_sample_d are for the right channel in the original code, but this is mono.
	prev_sample_a: T,
	prev_sample_c: T,

	gain: T,
	iterations: T,
	power_factor: i64,
	gain_scaling: T,
	output_scaling: T,
}

impl<T: ScalarFloat> ComponentMeta for Tube<T> {
	fn reset(&mut self) {
		self.prev_sample_a = T::ZERO;
		self.prev_sample_c = T::ZERO;
	}
}

impl<T: ScalarFloat> Component<T> for Tube<T> {
	#[replace_float_literals(T)]
	fn process(&mut self, mut sample: T) -> T {
		// [Omitted dithering code]

		//for high sample rates on this plugin we are going to do a simple average
		if self.overall_scale > 1.9 {
			let stored = sample;

			sample += self.prev_sample_a;
			self.prev_sample_a = stored;
			sample *= 0.5;
		}

		sample *= self.gain;
		sample = sample.clamp(-1.0, 1.0);

		let mut factor = sample;
		//this applies more and more of a 'curve' to the transfer function
		for _ in 0..self.power_factor {
			factor *= sample;
		}

		//if we would've got an asymmetrical effect this undoes the last step, and then
		//redoes it using an absolute value to make the effect symmetrical again
		if self.power_factor % 2 == 1 && sample != 0.0 {
			factor = (factor / sample) * sample.abs();
		}

		factor *= self.gain_scaling;
		sample -= factor;
		sample *= self.output_scaling;

		// [Right channel code omitted]

		//for high sample rates on this plugin we are going to do a simple average
		if self.overall_scale > 1.9 {
			let stored = sample;

			sample += self.prev_sample_c;
			self.prev_sample_c = stored;
			sample *= 0.5;
		}

		// [Float dither code omitted]

		sample
	}
}

impl<T: ScalarFloat> Tube<T> {
	/// Create a new [`Tube`].
	///
	/// Panics on debug builds if `sample_rate <= 0.0`.
	#[replace_float_literals(T)]
	pub fn new(sample_rate: T) -> Self {
		nih_plug::nih_debug_assert!(sample_rate > T::ZERO);

		let mut x = Self {
			overall_scale: 1.0 / 44100.0 * sample_rate,
			prev_sample_a: T::ZERO,
			prev_sample_c: T::ZERO,
			gain: T::ZERO,
			iterations: T::ZERO,
			power_factor: 0,
			gain_scaling: T::ZERO,
			output_scaling: T::ZERO,
		};

		x.set_amount(0.0);
		x
	}

	/// Set the tube amount / drive.
	///
	/// Panics on debug builds if `amount < 0.0 || amount > 1.0`.
	#[replace_float_literals(T)]
	pub fn set_amount(&mut self, amount: T) {
		nih_plug::nih_debug_assert!(amount >= T::ZERO && amount <= T::ONE);

		self.gain = 1.0 + (amount * 0.2246161992650486);

		//this maxes out at +1.76dB, which is the exact difference between what a triangle/saw wave
		//would be, and a sine (the fullest possible wave at the same peak amplitude). Why do this?
		//Because the nature of this plugin is the 'more FAT TUUUUBE fatness!' knob, and because
		//sticking to this amount maximizes that effect on a 'normal' sound that is itself unclipped
		//while confining the resulting 'clipped' area to what is already 'fattened' into a flat
		//and distorted region. You can always put a gain trim in front of it for more distortion,
		//or cascade them in the DAW for more distortion.

		self.iterations = 1.0 - amount;
		self.power_factor = (5.0 * self.iterations + 1.0).to_f64() as i64;
		self.gain_scaling = 1.0 / T::splat((self.power_factor + 1) as f64);
		self.output_scaling = 1.0 + (1.0 / T::splat(self.power_factor as f64));
	}
}
