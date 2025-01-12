// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

use simdeez::{
	avx2::*,
	scalar::*,
	sse2::*,
	sse41::*,
};

use crate::{
	Component,
	ComponentMeta,
	SimdFloat,
};

simd_runtime_generate! {
	fn process(gain: f32, samples: &mut [f32]) {
		unsafe {
			let mut n = 0;
			let gain_vector = S::set1_ps(gain);

			while n + S::VF32_WIDTH <= samples.len() {
				// Load a chunk of samples into the SIMD vector.
				// Don't worry about we seemingly providing 1 value here (&samples[n]), it does unsafe things to load an appropriate amount starting from index `n`.
				let sample_vector = S::loadu_ps(&samples[n]);
				// Do the thing
				let result = sample_vector * gain_vector;
				// Save the result
				S::storeu_ps(&mut samples[n], result);

				n += S::VF32_WIDTH;
			}

			// In case we have leftovers.
			for sample in &mut samples[n..] {
				*sample *= gain;
			}
		}
	}
}

/// Efficiently apply voltage gain to an f32 slice.
pub fn apply_gain(gain: f32, samples: &mut [f32]) {
	if gain != 1.0 {
		process_runtime_select(gain, samples);
	}
}

/// Efficiently apply voltage gain to the first 2 channels in a slice of channels.
#[inline]
pub fn apply_gain_mono_stereo<T: AsMut<[f32]>>(gain: f32, channels: &mut [T]) {
	for channel in channels.iter_mut().take(2) {
		apply_gain(gain, channel.as_mut());
	}
}

/// A clean gain [`Component`].
#[derive(Debug, Copy, Clone)]
pub struct Gain<T>(pub T);

impl<T> ComponentMeta for Gain<T> {}

impl<T: SimdFloat> Component<T> for Gain<T> {
	#[inline]
	fn process(&mut self, sample: T) -> T {
		sample * self.0
	}
}
