// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

pub struct Oversampler {
	factor: u8,
	input_buf: Box<[f64]>,
	output_buf: Box<[f64]>,
	upsamplers: Box<[Resampler]>,
	downsamplers: Box<[Resampler]>,
}

impl Oversampler {
	pub fn new(block_size: usize, max_factor: u8, initial_factor: u8) -> Self {
		assert_ne!(block_size, 0);
		assert!(max_factor <= 8, "oversampling 2^{max_factor} times is extremely wasteful and unnecessary; the `max_factor` value must be below 9");
		assert!(
			initial_factor <= max_factor,
			"the initial oversampling factor `initial_factor` can't be more than `max_factor` ({initial_factor} and {max_factor})"
		);

		Self {
			factor: initial_factor,
			block_size,
			input_buf: vec![0.0; block_size * usize::pow(2, max_factor as _)].into_boxed_slice(),
			output_buf: vec![0.0; block_size * usize::pow(2, max_factor as _)].into_boxed_slice(),
			upsamplers: (0..max_factor).map(|i| Resampler::new(1.0, usize::pow(2, i as u32 + 1) as f64, block_size * usize::pow(2, i as u32 + 1), 2.0, PrecisionProfile::Bits32)).collect(),
			downsamplers: (0..max_factor).map(|i| Resampler::new(usize::pow(2, i as u32 + 1) as f64, 1.0, block_size * usize::pow(2, i as u32 + 1), 2.0, PrecisionProfile::Bits32)).collect(),
		}
	}

	/// Set the active oversampling factor. Note that the final oversampling ratio will be `pow(2, factor)`.
	///
	/// # Panics
	/// If debug assertions are enabled, this function panics if `factor` is greater than `max_factor` provided in [new()][Self::new].
	pub fn set_oversampling_factor(&mut self, factor: u8) {
		nih_debug_assert!(factor <= self.max_factor);
		let factor = u8::min(factor, self.max_factor);

	if factor != self.factor {
		self.factor = factor;
		for i in 0..self.downsamplers.len() {
			self.downsamplers[i].clear();
			self.upsamplers[i].clear();
		}
	}
	}

	/// Get the latency of this Oversampler, in samples.
	pub fn latency(&self) -> usize {
		0
	}

	/// Process a block of samples, applying a closure to the oversampled samples transparently.
	///
	/// The `process` function will be given the upsampled signal; at the end, the downsampled samples are written to the input `samples` automatically.
	pub fn process_block(&mut self, samples: &mut [f32], mut process: impl FnMut(&mut [f64])) {
		for (n_chunk, chunk) in samples.chunks_mut(self.block_size).enumerate() {
			// Copy samples to the buffer as f64, chunk by chunk.
			let offset = n_chunk * self.buffer_size;
			for (i, sample) in chunk.iter().enumerate() {
				self.input_buf[offset + i] = *sample as _;
			}
			
			// Upsample.
			let upsampler = &mut self.upsamplers[self.factor as usize];
			
			let downsampler = &mut self.downsamplers[self.factor as usize];
			
		}
	}

	/// Reset state associated with this [Oversampler].
	pub fn reset(&mut self) {
		self.inner.reset();
	}
}
