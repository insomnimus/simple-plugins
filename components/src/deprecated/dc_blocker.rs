// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

#[derive(Debug, Clone)]
pub struct DcBlocker<T> {
	x: T,
	y: T,
	r: T,
}

impl<T: SimdFloat> DcBlocker<T> {
	pub fn new(sample_rate: f64) -> Self {
		Self {
			x: T::ZERO,
			y: T::ZERO,
			// r: T::ONE - T::splat(126.0) / T::splat(sample_rate),
			r: T::ONE - T::splat(190.0) / T::splat(sample_rate),
		}
	}
}

impl<T: SimdFloat> ComponentMeta for DcBlocker<T> {
	fn reset(&mut self) {
		self.x = T::ZERO;
		self.y = T::ZERO;
	}
}

impl<T: SimdFloat> Component<T> for DcBlocker<T> {
	fn process(&mut self, sample: T) -> T {
		self.y = sample - self.x + self.r * self.y;
		self.x = sample;

		self.y
	}
}
