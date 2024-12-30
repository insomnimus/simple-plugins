// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

mod os1;
mod os2;

use nih_plug::nih_debug_assert;

pub use self::os1::Oversampler as Oversampler1;
use self::os2::Lanczos3Oversampler;

/// A barebones multi-stage linear-phase oversampler that uses the lanzcos kernel.
pub struct Oversampler2 {
	max_factor: u8,
	factor: u8,
	block_size: usize,
	inner: Lanczos3Oversampler,
}

impl Oversampler2 {
	pub fn new(block_size: usize, max_factor: u8, initial_factor: u8) -> Self {
		assert!(
			initial_factor <= max_factor,
			"`initial_factor` can't exceed `max_factor`"
		);
		assert!(
			max_factor <= 8,
			"the maximum oversampling factor `max_factor` can't be greater than 8"
		);

		Self {
			factor: initial_factor,
			max_factor,
			block_size,
			inner: Lanczos3Oversampler::new(block_size, max_factor as _),
		}
	}

	/// Set the active oversampling factor.
	///
	/// Note that the oversampling ratio is `pow(2, factor)`, so when `factor` is 3, the oversampling ratio is 8.
	pub fn set_oversampling_factor(&mut self, factor: u8) {
		nih_debug_assert!(factor <= self.max_factor);
		self.factor = u8::min(factor, self.max_factor);
	}

	/// Reset state associated with this oversampler.
	pub fn reset(&mut self) {
		self.inner.reset();
	}

	/// Process a block of samples with `process`, after upsampling it, writing the downsampled samples back to the input block.
	///
	/// This handles the oversampling latency internally: the samples' phase is preserved.
	pub fn process_block(&mut self, samples: &mut [f32], mut process: impl FnMut(&mut [f32])) {
		for block in samples.chunks_mut(self.block_size) {
			self.inner.process(block, self.factor as _, &mut process);
		}
	}

	/// Get the latency of this oversampler.
	pub fn latency(&self) -> usize {
		0
	}

	/// Get the amount of shifting this oversampler does ot keep input and output phase the same.
	pub fn amount_of_shift(&self) -> usize {
		self.inner.latency(self.factor as _) as _
	}

	/// An upsample-only version of `process` that returns the upsampled version of the signal that
	/// would normally be passed to `process`'s callback. Useful for upsampling control signals.
	///
	/// # Panics
	/// Panics if `sample`'s length is longer than the maximum block size specified in [Oversampler2::new].
	pub fn upsample_only<'a>(&'a mut self, samples: &'a mut [f32]) -> &'a mut [f32] {
		self.inner.upsample_only(samples, self.factor as _)
	}
}
