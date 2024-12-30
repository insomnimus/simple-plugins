// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

use simdeez::{
	avx2::*,
	scalar::*,
	sse2::*,
	sse41::*,
};

simd_runtime_generate! {
	pub fn process64(threshold: f64, input_gain: f64, output_gain: f64, samples: &mut [f64]) {
		unsafe {
			let v_min = S::set1_pd(-threshold);
			let v_max = S::set1_pd(threshold);
			let v_input_gain = S::set1_pd(input_gain);
			let v_output_gain = S::set1_pd(output_gain);
			let mut i = 0;

			while i + S::VF64_WIDTH <= samples.len() {
				// Load a chunk of samples into the SIMD vector.
				// Don't worry about we seemingly providing 1 value here (&input[i]), it does unsafe things to load an appropriate amount starting from index `i`.
				let v_sample = S::loadu_pd(&samples[i]);

				// Do the thing
				let mut x = v_sample * v_input_gain;
				x = S::min_pd(x, v_max);
				x = S::max_pd(x, v_min);
				x *= v_output_gain;

				// Save the result
				S::storeu_pd(&mut samples[i], x);

				i += S::VF64_WIDTH;
			}

			// In case we have leftovers.
			for sample in &mut samples[i..] {
				*sample = f64::clamp(*sample * input_gain, -threshold, threshold) * output_gain;
			}
		}
	}
}

simd_runtime_generate! {
	pub fn process32(threshold: f32, input_gain: f32, output_gain: f32, samples: &mut [f32]) {
		unsafe {
			let v_min = S::set1_ps(-threshold);
			let v_max = S::set1_ps(threshold);
			let v_input_gain = S::set1_ps(input_gain);
			let v_output_gain = S::set1_ps(output_gain);
			let mut i = 0;

			while i + S::VF32_WIDTH <= samples.len() {
				// Load a chunk of samples into the SIMD vector.
				// Don't worry about we seemingly providing 1 value here (&input[i]), it does unsafe things to load an appropriate amount starting from index `i`.
				let v_sample = S::loadu_ps(&samples[i]);

				// Do the thing
				let mut x = v_sample * v_input_gain;
				x = S::min_ps(x, v_max);
				x = S::max_ps(x, v_min);
				x *= v_output_gain;

				// Save the result
				S::storeu_ps(&mut samples[i], x);

				i += S::VF64_WIDTH;
			}

			// In case we have leftovers.
			for sample in &mut samples[i..] {
				*sample = f32::clamp(*sample * input_gain, -threshold, threshold) * output_gain;
			}
		}
	}
}
