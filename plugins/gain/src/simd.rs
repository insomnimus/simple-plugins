use simdeez::{
	avx2::*,
	scalar::*,
	sse2::*,
	sse41::*,
};

simd_runtime_generate! {
	pub fn process_mono(gain: f32, samples: &mut [f32]) {
		unsafe {
			let mut n = 0;
			let gain_vector = S::set1_ps(gain);

			while n + S::VF32_WIDTH <= samples.len() {
				// a chunk of samples into the SIMD vector.
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
