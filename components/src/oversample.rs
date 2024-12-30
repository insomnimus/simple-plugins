// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

use core::array;

use crate::{
	half_band::HalfBand,
	Component,
	ComponentMeta,
};

#[derive(Debug, Clone)]
struct Inner<const N: usize> {
	buffer: Box<[f64]>,
	down_filters: [HalfBand<6>; N],
	up_filters: [HalfBand<6>; N],
}

impl<const N: usize> Inner<N> {
	fn new(block_size: usize) -> Self {
		let filters = array::from_fn(|_| HalfBand::steep_order12());

		Self {
			buffer: vec![0.0; block_size * usize::pow(2, N as _)].into_boxed_slice(),
			down_filters: filters.clone(),
			up_filters: filters,
		}
	}

	fn upsample(&mut self, input: &[f32], times: usize) -> &mut [f64] {
		debug_assert!(times <= N);
		debug_assert_ne!(times, 0);

		// First stage
		let f = &mut self.up_filters[0];

		for (i, &sample) in input.iter().enumerate() {
			self.buffer[i * 2] = f.process(sample as f64 + sample as f64);
			self.buffer[i * 2 + 1] = f.process(0.0);
		}

		for stage in 1..times {
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
			let f = &mut self.up_filters[stage];
			for sample in &mut self.buffer[..len] {
				*sample = f.process(*sample);
			}
		}

		&mut self.buffer[..input.len() * usize::pow(2, times as u32)]
	}

	fn downsample(&mut self, output: &mut [f32], times: usize) {
		debug_assert!(times <= N);
		debug_assert_ne!(times, 0);

		for stage in (1..times).rev() {
			let f = &mut self.down_filters[stage];
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

/// An oversampler with a built-in anti-aliasing filter.
///
/// The `N` constant refers to maximum number of oversampling stages; this means that the actual oversampling ratio is `pow(2, N)`.
/// Also note that `N` can't be greater than `8` as there's absolutely no reason to use ratios above it (and the memory use scales linearly with the oversampling ratio, so it quickly becomes impossible).
#[derive(Debug, Clone)]
pub struct Oversampler<const N: usize> {
	times: usize,
	block_size: usize,
	inner: Inner<N>,
}

impl<const N: usize> Oversampler<N> {
	/// Create a new `Oversampler`.
	///
	/// ## Panics
	/// Panics if `N > 8` or `initial_os_times > N` or `block_size == 0`.
	pub fn new(block_size: usize, initial_os_times: usize) -> Self {
		assert_ne!(block_size, 0);
		assert!(N <= 8, "oversampling 2^{N} times is extremely wasteful and unnecessary; the const generic value must be below 9");
		assert!(
			initial_os_times <= N,
			"the oversampling amount can't be more than N ({initial_os_times} and {N})"
		);

		Self {
			block_size,
			times: initial_os_times,
			inner: Inner::new(block_size),
		}
	}

	/// Set the number of active oversampling stages. Note that the final oversampling amount will be `pow(2, times)`.
	pub fn set_oversampling_times(&mut self, times: usize) {
		debug_assert!(times <= N);
		let times = usize::min(times, N);

		#[allow(clippy::comparison_chain)]
		if self.times < times {
			self.reset();
		} else if self.times > times {
			// Only reset the inactive filters.
			// This probably helps the quality.
			for f in &mut self.inner.up_filters[times..] {
				f.reset();
			}
			for f in &mut self.inner.down_filters.iter_mut().rev().take(times) {
				f.reset();
			}
		}

		if self.times != times {
			self.inner.reset();
		}
		self.times = usize::min(times, N);
	}

	/// Get the latency of this [Oversampler], in samples.
	pub fn latency(&self) -> usize {
		match self.times {
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
	pub fn process_block<F>(&mut self, samples: &mut [f32], mut process: F)
	where
		F: for<'a> FnMut(&'a mut [f64]),
	{
		if self.times == 0 {
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
			process(self.inner.upsample(chunk, self.times));
			self.inner.downsample(chunk, self.times);
		}
	}

	/// Reset state associated with this [Oversampler].
	pub fn reset(&mut self) {
		self.inner.reset();
	}
}
