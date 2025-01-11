// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

mod adaptors;
mod cascade;
mod dc_blocker;
mod gain;
mod half_band;
mod oversample;
mod simd;
mod simper;
mod sweeten;
mod tube;

pub use util::replace_float_literals;
/// SIMD accelerated 2 [`f64`]s, useful for stereo processing.
pub use wide::f32x4;
/// SIMD accelerated 4 [`f32`]s, useful for multi-channel processing.
pub use wide::f64x2;

pub use self::{
	adaptors::*,
	cascade::*,
	dc_blocker::*,
	gain::*,
	half_band::HalfBand,
	oversample::*,
	simd::*,
	simper::*,
	sweeten::*,
	tube::*,
};

include!(concat!(env!("OUT_DIR"), "/component_impls.rs"));

/// Trait for describing components.
pub trait ComponentMeta {
	fn reset(&mut self) {}

	fn latency(&self) -> usize {
		0
	}
}

/// Trait for per-sample audio processing components.
pub trait Component<T>: ComponentMeta {
	fn process(&mut self, sample: T) -> T;
}

impl<C: ComponentMeta> ComponentMeta for &mut C {
	fn latency(&self) -> usize {
		C::latency(self)
	}

	fn reset(&mut self) {
		C::reset(self)
	}
}

impl<T, C: Component<T>> Component<T> for &mut C {
	fn process(&mut self, sample: T) -> T {
		C::process(self, sample)
	}
}

/// A [Component] that does nothing.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Noop;

impl ComponentMeta for Noop {}
impl<T> Component<T> for Noop {
	fn process(&mut self, sample: T) -> T {
		sample
	}
}

/// Apply `mono` to the first channel if there's 1 channel, `stereo` to the first 2 channels if there's at least 2 channels, and do nothing if there are no channels.
pub fn apply_mono_stereo<M, S>(mut mono: M, mut stereo: S, channels: &mut [&mut [f32]])
where
	M: Component<f64>,
	S: Component<f64x2>,
{
	match channels {
		[left, right, ..] => {
			for (l, r) in left.iter_mut().zip(right.iter_mut()) {
				let sample = f64x2::new([*l as f64, *r as f64]);
				let [val_l, val_r] = stereo.process(sample).value();
				*l = val_l as _;
				*r = val_r as _;
			}
		}

		[mono_samples] => {
			for sample in mono_samples.iter_mut() {
				*sample = mono.process(*sample as f64) as _;
			}
		}

		_ => (),
	}
}
