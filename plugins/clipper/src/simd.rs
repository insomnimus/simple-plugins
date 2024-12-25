use simdeez::{
	avx2::*,
	scalar::*,
	sse2::*,
	sse41::*,
};

simd_runtime_generate! {
	pub fn process(
		threshold: f32,
		input_gain: f32,
		output_gain: f32,
		input: &[f32],
		output: &mut [f32],
	) {
		unsafe {
			let v_min = S::set1_ps(-threshold);
			let v_max = S::set1_ps(threshold);
			let v_input_gain = S::set1_ps(input_gain);
			let v_output_gain = S::set1_ps(output_gain);
			let mut i = 0;

			while i + S::VF32_WIDTH <= input.len() {
				// Load a chunk of samples into the SIMD vector.
				// Don't worry about we seemingly providing 1 value here (&input[i]), it does unsafe things to load an appropriate amount starting from index `i`.
				let v_sample = S::loadu_ps(&input[i]);

				// Do the thing
				let mut x = v_sample * v_input_gain;
				x = S::min_ps(x, v_min);
				x = S::max_ps(x, v_max);
				x *= v_output_gain;

				// Save the result
				S::storeu_ps(&mut output[i], x);

				i += S::VF32_WIDTH;
			}

			// In case we have leftovers.
			for i in i..input.len() {
				let x = input[i];
				output[i] = f32::clamp(x * input_gain, -threshold, threshold) * output_gain;
			}
		}
	}
}
