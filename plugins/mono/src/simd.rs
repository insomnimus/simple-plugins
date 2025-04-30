// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

use simdeez::{
	avx2::*,
	scalar::*,
	sse2::*,
	sse41::*,
};

simd_runtime_generate! {
	pub fn mix(ch1: &mut [f32], ch2: &[f32]) {
		nih_plug::nih_debug_assert!(ch1.len() == ch2.len());

		unsafe {
			let len = ch1.len();
			let v_half = S::set1_ps(0.5);
			let mut i = 0;

			while i + S::VF32_WIDTH <= len {
				// Load a chunk of samples into the SIMD vector.
				// Don't worry about we seemingly providing 1 value here (&input[i]), it does unsafe things to load an appropriate amount starting from index `i`.
				let v1= S::loadu_ps(&ch1[i]);
				let v2= S::loadu_ps(&ch2[i]);

				// Mix the vectors equally.
				let x = (v1 + v2) * v_half;

				// Save the result
				S::storeu_ps(&mut ch1[i], x);

				i += S::VF64_WIDTH;
			}

			// In case we have leftovers.
			for (s1, s2) in ch1[i..].iter_mut().zip(&ch2[i..]) {
				*s1 = (*s1 + *s2) / 2.0;
			}
		}
	}
}
