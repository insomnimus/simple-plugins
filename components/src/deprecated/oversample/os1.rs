// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

use nih_plug::{
	nih_debug_assert,
	nih_debug_assert_ne,
};

use crate::{
	half_band::HalfBand,
	Component,
	ComponentMeta,
};

#[derive(Debug, Clone)]
struct Inner {
	max_factor: u8,
	buffer: Box<[f64]>,
	down_filters: Box<[HalfBand<f64, 6>]>,
	up_filters: Box<[HalfBand<f64, 6>]>,
}

impl Inner {
	fn new(block_size: usize, max_factor: u8) -> Self {
		let filters = vec![HalfBand::steep_order12(); max_factor as usize].into_boxed_slice();

		Self {
			max_factor,
			buffer: vec![0.0; block_size * usize::pow(2, max_factor as _)].into_boxed_slice(),
			down_filters: filters.clone(),
			up_filters: filters,
		}
	}

	fn upsample(&mut self, input: &[f32], factor: u8) -> &mut [f64] {
		nih_debug_assert!(factor <= self.max_factor);
		nih_debug_assert_ne!(factor, 0);

		// First stage
		let f = &mut self.up_filters[0];

		for (i, &sample) in input.iter().enumerate() {
			self.buffer[i * 2] = f.process(sample as f64 + sample as f64);
			self.buffer[i * 2 + 1] = f.process(0.0);
		}

		for stage in 1..factor {
			// We have to do it in 2 iterations: zero-stuff, and then apply the filters.
			// This is because there's only one buffer.
			let len = input.len() * usize::pow(2, stage as u32 + 1);
			// Also have to iterate backwards so we don't overwrite any samples.
			for i in (0..len / 2).rev() {
				// Double the sample to preserve levels.
				self.buffer[i * 2] = self.buffer[i] + self.buffer[i];
				self.buffer[i * 2 + 1] = 0.0;
			}

			// Apply the filter.
			let f = &mut self.up_filters[stage as usize];
			for sample in &mut self.buffer[..len] {
				*sample = f.process(*sample);
			}
		}

		&mut self.buffer[..input.len() * usize::pow(2, factor as u32)]
	}

	fn downsample(&mut self, output: &mut [f32], factor: u8) {
		nih_debug_assert!(factor as usize <= self.up_filters.len());
		nih_debug_assert_ne!(factor, 0);

		for stage in (1..factor).rev() {
			let f = &mut self.down_filters[stage as usize];
			// The length of the buffer we're about to "produce".
			let len = output.len() * usize::pow(2, stage as u32);
			for i in 0..len {
				self.buffer[i] = f.process(self.buffer[i * 2]);
				// The filter needs to consume the other sample as well to be accurate.
				let _ = f.process(self.buffer[i * 2 + 1]);
			}
		}

		// First (last) stage
		// Same as the loop above except as an optimization, we're writing directly to the output buffer.
		// Also we need to cast samples to f32 anyway...
		let f = &mut self.down_filters[0];
		for (i, sample) in output.iter_mut().enumerate() {
			*sample = f.process(self.buffer[i * 2]) as f32;
			let _ = f.process(self.buffer[i * 2 + 1]);
		}
	}

	fn reset(&mut self) {
		for f in &mut self.up_filters {
			f.reset();
		}
		for f in &mut self.down_filters {
			f.reset();
		}
	}
}

/// A minimum phase oversampler using half-band filters.
#[derive(Debug, Clone)]
pub struct Oversampler {
	max_factor: u8,
	factor: u8,
	block_size: usize,
	inner: Inner,
}

impl Oversampler {
	/// Create a new `Oversampler`.
	///
	/// # Panics
	/// Panics if `max_factor > 8` or `initial_factor > max_factor` or `block_size == 0`.
	pub fn new(block_size: usize, max_factor: u8, initial_factor: u8) -> Self {
		assert_ne!(block_size, 0);
		assert!(max_factor <= 8, "oversampling 2^{max_factor} times is extremely wasteful and unnecessary; the `max_factor` value must be below 9");
		assert!(
			initial_factor <= max_factor,
			"the initial oversampling factor `initial_factor` can't be more than `max_factor` ({initial_factor} and {max_factor})"
		);

		Self {
			factor: initial_factor,
			max_factor,
			block_size,
			inner: Inner::new(block_size, max_factor),
		}
	}

	/// Set the active oversampling factor. Note that the final oversampling ratio will be `pow(2, factor)`.
	///
	/// # Panics
	/// If debug assertions are enabled, this function panics if `factor` is greater than `max_factor` provided in [new()][Self::new].
	pub fn set_oversampling_factor(&mut self, factor: u8) {
		nih_debug_assert!(factor <= self.max_factor);
		let factor = u8::min(factor, self.max_factor);

		#[allow(clippy::comparison_chain)]
		if self.factor < factor {
			self.reset();
		} else if self.factor > factor {
			// Only reset the inactive filters.
			// This probably helps the quality.
			for f in &mut self.inner.up_filters[factor as usize..] {
				f.reset();
			}
			for f in &mut self.inner.down_filters.iter_mut().rev().take(factor as _) {
				f.reset();
			}
		}

		self.factor = factor;
	}

	/// Get the latency of this Oversampler, in samples.
	pub fn latency(&self) -> usize {
		match self.factor {
			0 => 0,
			1 => 13,
			2 => 19,
			3 => 22,
			4 => 23,
			5 => 24,
			// It actually maxes out at 25 samples (well, it keeps going fractionally but never gets there).
			_ => 25,
		}
	}

	/// Process a block of samples, applying a closure to the oversampled samples transparently.
	///
	/// The `process` function will be given the upsampled signal; at the end, the downsampled samples are written to the input `samples` automatically.
	pub fn process_block(&mut self, samples: &mut [f32], mut process: impl FnMut(&mut [f64])) {
		if self.factor == 0 {
			// Temporarily use self.inner.buffer for f64 samples.
			for block in samples.chunks_mut(self.block_size) {
				for (i, sample) in block.iter().enumerate() {
					self.inner.buffer[i] = *sample as f64;
				}

				process(&mut self.inner.buffer[..block.len()]);

				// Copy it back to `block`.
				for (i, sample) in block.iter_mut().enumerate() {
					*sample = self.inner.buffer[i] as _;
				}
			}
			return;
		}

		for chunk in samples.chunks_mut(self.block_size) {
			process(self.inner.upsample(chunk, self.factor));
			self.inner.downsample(chunk, self.factor);
		}
	}

	/// Reset state associated with this [Oversampler].
	pub fn reset(&mut self) {
		self.inner.reset();
	}
}
