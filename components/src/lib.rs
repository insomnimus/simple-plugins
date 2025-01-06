// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

mod adaptors;
mod dc_blocker;
mod half_band;
mod oversample;
mod sample;
mod simd;
mod simper;
mod stilson_moog;

pub use self::{
	adaptors::*,
	dc_blocker::DcBlocker,
	half_band::HalfBand,
	oversample::*,
	sample::Sample,
	simper::{
		Simper,
		SimperCoefficients,
	},
	stilson_moog::LowPassSM,
};

include!(concat!(env!("OUT_DIR"), "/component_impls.rs"));

macro_rules! generate_block_impls {
	[$($t:ty),+ $(,)?] => {
		$(
			impl ComponentBlock for $t {
				#[inline]
				fn process_block(&mut self, samples: &mut [f32]) {
					for sample in samples {
						*sample = <$t as $crate::Component>::process(self, *sample as _) as _;
					}
				}
			}
		)+
	};
}

generate_block_impls![DcBlocker, Simper,];

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

/// Cascade a [Component] or a [ComponentBlock] several times.
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

impl<C: ComponentBlock, const N: usize> ComponentBlock for Cascade<C, N> {
	fn process_block(&mut self, samples: &mut [f32]) {
		for c in &mut self.0 {
			c.process_block(samples);
		}
	}
}

impl<C: ComponentMeta> ComponentMeta for &mut C {
	fn latency(&self) -> usize {
		C::latency(self)
	}

	fn reset(&mut self) {
		C::reset(self)
	}
}

impl<C: Component> Component for &mut C {
	fn process(&mut self, sample: f64) -> f64 {
		C::process(self, sample)
	}
}

impl<C: ComponentBlock> ComponentBlock for &mut C {
	fn process_block(&mut self, samples: &mut [f32]) {
		C::process_block(self, samples)
	}
}

pub fn apply_component<C: Component>(mut component: C, samples: &mut [f32]) {
	for sample in samples {
		*sample = component.process(*sample as _) as _;
	}
}
