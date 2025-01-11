// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

//! Port of [AirWindows'](https://github.com/airwindows/airwindows) Sweeten plugin, as a [`Component`].

use crate::{
	replace_float_literals,
	Component,
	ComponentMeta,
	SimdFloat,
};

/// A subtle saturator that produces 2nd order harmonics.
#[derive(Debug, Clone)]
pub struct Sweeten<T> {
	cycle_end: i32,
	sweet: T,
	savg: [T; 8],
}

impl<T: SimdFloat> ComponentMeta for Sweeten<T> {
	fn reset(&mut self) {
		self.savg.fill(T::ZERO);
	}
}

impl<T: SimdFloat> Component<T> for Sweeten<T> {
	#[inline]
	#[replace_float_literals(T)]
	fn process(&mut self, mut sample: T) -> T {
		let savg = &mut self.savg;

		let mut sweet_sample = sample;

		let mut sv = sweet_sample;
		sweet_sample = (sweet_sample + savg[0]) * 0.5;
		savg[0] = sv;

		if self.cycle_end > 1 {
			sv = sweet_sample;
			sweet_sample = (sweet_sample + savg[1]) * 0.5;
			savg[1] = sv;

			if self.cycle_end > 2 {
				sv = sweet_sample;
				sweet_sample = (sweet_sample + savg[2]) * 0.5;
				savg[2] = sv;

				if self.cycle_end > 3 {
					sv = sweet_sample;
					sweet_sample = (sweet_sample + savg[3]) * 0.5;
					savg[3] = sv;
				}
			} //if undersampling code is present, this gives a simple averaging that stacks up
		} //when high sample rates are present, for a more intense aliasing reduction. PRE nonlinearity

		sweet_sample = sweet_sample * sweet_sample * self.sweet; //second harmonic (nonlinearity)
		sv = sweet_sample;
		sweet_sample = (sweet_sample + savg[4]) * 0.5;
		savg[4] = sv;

		if self.cycle_end > 1 {
			sv = sweet_sample;
			sweet_sample = (sweet_sample + savg[5]) * 0.5;
			savg[5] = sv;

			if self.cycle_end > 2 {
				sv = sweet_sample;
				sweet_sample = (sweet_sample + savg[6]) * 0.5;
				savg[6] = sv;

				if self.cycle_end > 3 {
					sv = sweet_sample;
					sweet_sample = (sweet_sample + savg[7]) * 0.5;
					savg[7] = sv;
				}
			} //if undersampling code is present, this gives a simple averaging that stacks up
		} //when high sample rates are present, for a more intense aliasing reduction. POST nonlinearity

		sample -= sweet_sample; //apply the filtered second harmonic correction

		// [Right channel code omitted]
		// [Dither code omitted]

		sample
	}
}

impl<T: SimdFloat> Sweeten<T> {
	/// Create a new [`Sweeten`].
	pub fn new(sample_rate: f64) -> Self {
		nih_plug::nih_debug_assert!(sample_rate >= 0.0);

		let overall_scale = 1.0 / 44100.0 * sample_rate;

		Self {
			cycle_end: overall_scale.floor().clamp(1.0, 4.0) as _,
			sweet: T::splat(0.0009765625),
			savg: [T::ZERO; 8],
		}
	}

	/// Set saturation amount from `0.0` to `1.0`.
	///
	/// Panics in debug builds if `amount` is outside `0..=1`.
	pub fn set_amount(&mut self, amount: f64) {
		nih_plug::nih_debug_assert!((0.0..=1.0).contains(&amount));

		let sweet_bits = 10 - (amount * 10.0).floor() as i32;

		self.sweet = T::splat(match sweet_bits {
			11 => 0.00048828125,
			10 => 0.0009765625,
			9 => 0.001953125,
			8 => 0.00390625,
			7 => 0.0078125,
			6 => 0.015625,
			5 => 0.03125,
			4 => 0.0625,
			3 => 0.125,
			2 => 0.25,
			1 => 0.5,
			0 => 1.0,
			-1 => 2.0,
			_ => 1.0,
		});
	}
}
