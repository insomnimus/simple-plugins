// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

use components::{
	f64x2,
	Cascade,
	Component,
	ComponentMeta,
	Simper,
};
use nih_plug::util::db_to_gain;

pub struct SundaraEq {
	filters: Cascade<Simper<f64x2>, 3>,
	preamp: f64x2,
}

impl Default for SundaraEq {
	fn default() -> Self {
		let filters = Cascade(core::array::from_fn(|_| {
			Simper::bell(
				f64x2::splat(44100.0),
				f64x2::splat(5000.0),
				Simper::BUTTERWORTH_Q,
				f64x2::splat(0.0),
			)
		}));

		let preamp = f64x2::splat(db_to_gain(-4.0) as _);

		Self { filters, preamp }
	}
}

impl SundaraEq {
	pub fn new(sample_rate: f32) -> Self {
		let sr = f64x2::splat(sample_rate as _);
		#[allow(clippy::type_complexity)]
		let filters: [(fn(f64x2, f64x2, f64x2, f64x2) -> Simper<f64x2>, _, _, _); 3] = [
			(Simper::low_shelf, 50.0, 1.0, 3.5),
			(Simper::bell, 2112.0, 1.5, 2.5),
			(Simper::bell, 6300.0, 4.0, -3.0),
		];

		Self {
			filters: Cascade(filters.map(|(f, fq, q, gain)| {
				f(sr, f64x2::splat(fq), f64x2::splat(q), f64x2::splat(gain))
			})),
			..Self::default()
		}
	}
}

impl ComponentMeta for SundaraEq {
	fn latency(&self) -> usize {
		self.filters.latency()
	}

	fn reset(&mut self) {
		self.filters.reset();
	}
}

impl Component<f64x2> for SundaraEq {
	fn process(&mut self, sample: f64x2) -> f64x2 {
		self.filters.process(self.preamp * sample)
	}
}
