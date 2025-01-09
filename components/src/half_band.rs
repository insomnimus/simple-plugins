// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

//! Adapted from <https://github.com/SolarLiner/valib/blob/main/crates/valib-filters/src/halfband.rs>

use crate::{
	Cascade,
	Component,
	ComponentMeta,
	SimdFloat,
};

/// Specialized 2nd-order AllPass filter.
#[derive(Debug, Copy, Clone)]
struct AllPass<T> {
	a: T,
	x: [T; 3],
	y: [T; 3],
}

impl<T: SimdFloat> AllPass<T> {
	fn new(a: T) -> Self {
		Self {
			a,
			x: [T::ZERO; 3],
			y: [T::ZERO; 3],
		}
	}

	fn rotate_state(&mut self, x: T) {
		let [x0, x1, _] = self.x;
		self.x = [x, x0, x1];
		self.y.rotate_right(1);
	}
}

impl<T: SimdFloat> ComponentMeta for AllPass<T> {
	fn latency(&self) -> usize {
		2
	}

	fn reset(&mut self) {
		self.x = [T::ZERO; 3];
		self.y = [T::ZERO; 3];
	}
}

impl<T: SimdFloat> Component<T> for AllPass<T> {
	fn process(&mut self, x: T) -> T {
		self.rotate_state(x);
		self.y[0] = self.x[2] + ((x - self.y[2]) * self.a);
		// self.y[0] = self.x[2];

		self.y[0]
	}
}

/// Half-band filter of order `2 * ORDER`.
#[derive(Debug, Clone)]
pub struct HalfBand<T, const ORDER: usize> {
	filter_a: Cascade<AllPass<T>, ORDER>,
	filter_b: Cascade<AllPass<T>, ORDER>,
	y0: T,
}

impl<T: SimdFloat, const ORDER: usize> ComponentMeta for HalfBand<T, ORDER> {
	fn latency(&self) -> usize {
		self.filter_a.latency() + self.filter_b.latency()
	}

	fn reset(&mut self) {
		self.filter_a.reset();
		self.filter_b.reset();
	}
}

impl<T: SimdFloat, const ORDER: usize> Component<T> for HalfBand<T, ORDER> {
	#[inline]
	fn process(&mut self, x: T) -> T {
		let y = (self.filter_a.process(x) + self.y0) * T::HALF;
		self.y0 = self.filter_b.process(x);

		y
	}
}

impl<T: SimdFloat, const ORDER: usize> HalfBand<T, ORDER> {
	fn from_coeffs(k_a: [T; ORDER], k_b: [T; ORDER]) -> Self {
		Self {
			filter_a: Cascade(core::array::from_fn(|i| AllPass::new(k_a[i]))),
			filter_b: Cascade(core::array::from_fn(|i| AllPass::new(k_b[i]))),
			y0: T::ZERO,
		}
	}
}

impl<T: SimdFloat> HalfBand<T, 6> {
	/// Construct a steep half-band filter of order 12.
	pub fn steep_order12() -> Self {
		Self::from_coeffs(
			[
				0.036681502163648017,
				0.2746317593794541,
				0.5610989697879195,
				0.769741833862266,
				0.8922608180038789,
				0.962094548378084,
			]
			.map(T::splat),
			[
				0.13654762463195771,
				0.42313861743656667,
				0.6775400499741616,
				0.839889624849638,
				0.9315419599631839,
				0.9878163707328971,
			]
			.map(T::splat),
		)
	}
}

impl<T: SimdFloat> HalfBand<T, 5> {
	/// Construct a steep half-band filter of order 10.
	pub fn steep_order10() -> Self {
		Self::from_coeffs(
			[
				0.051457617441190984,
				0.35978656070567017,
				0.6725475931034693,
				0.8590884928249939,
				0.9540209867860787,
			]
			.map(T::splat),
			[
				0.18621906251989334,
				0.529951372847964,
				0.7810257527489514,
				0.9141815687605308,
				0.985475023014907,
			]
			.map(T::splat),
		)
	}
}
