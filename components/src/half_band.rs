// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

//! Adapted from https://github.com/SolarLiner/valib/blob/main/crates/valib-filters/src/halfband.rs

use crate::{
	Cascade,
	Component,
	ComponentMeta,
};

/// Specialized 2nd-order AllPass filter.
#[derive(Debug, Copy, Clone)]
struct AllPass {
	a: f64,
	x: [f64; 3],
	y: [f64; 3],
}

impl AllPass {
	fn new(a: f64) -> Self {
		Self {
			a,
			x: [0.0; 3],
			y: [0.0; 3],
		}
	}

	fn rotate_state(&mut self, x: f64) {
		let [x0, x1, _] = self.x;
		self.x = [x, x0, x1];
		self.y.rotate_right(1);
	}
}

impl ComponentMeta for AllPass {
	fn latency(&self) -> usize {
		2
	}

	fn reset(&mut self) {
		self.x = [0.0; 3];
		self.y = [0.0; 3];
	}
}

impl Component for AllPass {
	fn process(&mut self, x: f64) -> f64 {
		self.rotate_state(x);
		self.y[0] = self.x[2] + ((x - self.y[2]) * self.a);
		// self.y[0] = self.x[2];

		self.y[0]
	}
}

/// Half-band filter of order `2 * ORDER`.
#[derive(Debug, Clone)]
pub struct HalfBand<const ORDER: usize> {
	filter_a: Cascade<AllPass, ORDER>,
	filter_b: Cascade<AllPass, ORDER>,
	y0: f64,
}

impl<const ORDER: usize> ComponentMeta for HalfBand<ORDER> {
	fn latency(&self) -> usize {
		self.filter_a.latency() + self.filter_b.latency()
	}

	fn reset(&mut self) {
		self.filter_a.reset();
		self.filter_b.reset();
	}
}

impl<const ORDER: usize> Component for HalfBand<ORDER> {
	#[inline]
	fn process(&mut self, x: f64) -> f64 {
		let y = (self.filter_a.process(x) + self.y0) * 0.5;
		self.y0 = self.filter_b.process(x);

		y
	}
}

impl<const ORDER: usize> HalfBand<ORDER> {
	fn from_coeffs(k_a: [f64; ORDER], k_b: [f64; ORDER]) -> Self {
		Self {
			filter_a: Cascade(std::array::from_fn(|i| AllPass::new(k_a[i]))),
			filter_b: Cascade(std::array::from_fn(|i| AllPass::new(k_b[i]))),
			y0: 0.0,
		}
	}
}

impl HalfBand<6> {
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
			],
			[
				0.13654762463195771,
				0.42313861743656667,
				0.6775400499741616,
				0.839889624849638,
				0.9315419599631839,
				0.9878163707328971,
			],
		)
	}
}

impl HalfBand<5> {
	/// Construct a steep half-band filter of order 10.
	pub fn steep_order10() -> Self {
		Self::from_coeffs(
			[
				0.051457617441190984,
				0.35978656070567017,
				0.6725475931034693,
				0.8590884928249939,
				0.9540209867860787,
			],
			[
				0.18621906251989334,
				0.529951372847964,
				0.7810257527489514,
				0.9141815687605308,
				0.985475023014907,
			],
		)
	}
}
