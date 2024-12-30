// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

mod half_band;
mod oversample;
pub mod simper;
mod stilson_moog;

pub use self::{
	half_band::HalfBand,
	oversample::Oversampler,
	simper::Simper,
	stilson_moog::LowPassSM,
};

pub trait ComponentMeta {
	fn reset(&mut self) {}

	fn latency(&self) -> usize {
		0
	}
}

pub trait Component: ComponentMeta {
	fn process(&mut self, sample: f64) -> f64;
}

pub trait ComponentBlock: ComponentMeta {
	fn process_block(&mut self, block: &mut [f32]);
}

#[derive(Debug, Clone)]
pub struct Cascade<C, const N: usize>(pub [C; N]);

impl<C: ComponentMeta, const N: usize> ComponentMeta for Cascade<C, N> {
	fn reset(&mut self) {
		for c in &mut self.0 {
			c.reset();
		}
	}

	fn latency(&self) -> usize {
		self.0.iter().map(C::latency).sum()
	}
}

impl<C: Component, const N: usize> Component for Cascade<C, N> {
	#[inline]
	fn process(&mut self, mut sample: f64) -> f64 {
		for f in &mut self.0 {
			sample = f.process(sample);
		}

		sample
	}
}
