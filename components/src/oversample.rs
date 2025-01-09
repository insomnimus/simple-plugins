// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

use rubato::{
	FftFixedInOut,
	Resampler,
};

#[derive(Debug, Clone)]
struct Buffered {
	internal_buffer_size: usize,
	internal_input: Box<[f32]>,
	internal_output: Box<[f32]>,
	counter: usize,
}

impl Buffered {
	pub fn new(internal_buffer_size: usize) -> Self {
		Self {
			internal_buffer_size,
			internal_input: vec![0.0; internal_buffer_size].into_boxed_slice(),
			internal_output: vec![0.0; internal_buffer_size].into_boxed_slice(),
			counter: 0,
		}
	}

	pub fn process_block<I, O, F>(&mut self, num_frames: usize, inpt: I, mut outp: O, mut f: F)
	where
		I: AsRef<[f32]>,
		O: AsMut<[f32]>,
		F: FnMut(&[f32], &mut [f32]),
	{
		let mut num_processed = 0;
		while num_processed < num_frames {
			// Compute how many samples to copy.
			let count = (num_frames - num_processed).min(self.internal_buffer_size - self.counter);

			// Copy from input to internal input, internal output to output.
			{
				let internal_input = &mut self.internal_input;
				let internal_output = &self.internal_output;

				let input = inpt.as_ref();
				let output = outp.as_mut();

				// Copy external input to internal.
				internal_input[self.counter..(self.counter + count)]
					.copy_from_slice(&input[num_processed..(num_processed + count)]);

				// Copy external input to output. Note that this is delayed by one buffer size.
				output[num_processed..(num_processed + count)]
					.copy_from_slice(&internal_output[self.counter..(self.counter + count)]);
			}

			// Increment counters.
			self.counter += count;
			num_processed += count;

			// Process if the internal buffer is filled.
			if self.counter == self.internal_buffer_size {
				self.counter = 0;
				f(&self.internal_input, &mut self.internal_output);
			}
		}
	}

	pub fn reset(&mut self) {
		self.internal_input.fill(0.0);
		self.internal_output.fill(0.0);
		self.counter = 0;
	}
}

/// A linear-phase oversampler using [rubato::FftFixedInOut].
///
/// It has a considerable latency, making it not very suitable for live tracking.
pub struct Oversampler {
	orig_sample_rate: usize,
	ratio: u8,
	block_size: usize,
	buffered: Buffered,

	copy_buffer: Box<[f32]>,
	oversampled_buffer: Box<[f32]>,

	upsampler: FftFixedInOut<f32>,
	downsampler: FftFixedInOut<f32>,
}

impl Oversampler {
	/// Create a new [`Oversampler`].
	///
	/// # Parameters
	///
	/// - `block_size`: How many samples should be buffered. Lower values decrease latency but increase CPU load, higher values do the opposite. A good value is whatever the host provides you with, or half that. Panics if `0`.
	/// - `orig_sample_rate`: The sample rate of the input. Panics if `0`.
	/// - `ratio`: Oversampling amount. Panics if `0`.
	pub fn new(block_size: usize, orig_sample_rate: usize, ratio: u8) -> Self {
		assert_ne!(block_size, 0);
		assert_ne!(orig_sample_rate, 0);
		assert_ne!(ratio, 0);

		let upsample_rate = orig_sample_rate * ratio as usize;
		let buffered = Buffered::new(block_size);

		let oversampled_buffer = vec![0.0; block_size * ratio as usize].into_boxed_slice();
		let copy_buffer = vec![0.0; block_size].into_boxed_slice();

		let upsampler = FftFixedInOut::new(orig_sample_rate, upsample_rate, block_size, 1).unwrap();

		let downsampler = FftFixedInOut::new(
			upsample_rate,
			orig_sample_rate,
			block_size * ratio as usize,
			1,
		)
		.unwrap();

		Self {
			orig_sample_rate,
			ratio,
			block_size,
			copy_buffer,
			oversampled_buffer,
			buffered,
			upsampler,
			downsampler,
		}
	}

	/// Process a block of samples at a higher sampling rate.
	pub fn process_block<F>(&mut self, samples: &mut [f32], mut f: F)
	where
		F: FnMut(&mut [f32]),
	{
		for chunk in samples.chunks_mut(self.block_size) {
			let chunk_len = chunk.len();
			self.copy_buffer[..chunk_len].copy_from_slice(chunk);
			let oversampled_buffer = &mut self.oversampled_buffer;
			let copy_buffer = &self.copy_buffer;

			self.buffered
				.process_block(chunk_len, &copy_buffer[..chunk_len], chunk, |inp, out| {
					// let mut oversampled_buffer = oversampled_buffer.borrow_mut();

					self.upsampler
						.process_into_buffer(&[inp], &mut [&mut **oversampled_buffer], None)
						.unwrap();

					f(oversampled_buffer);

					self.downsampler
						.process_into_buffer(&[&*oversampled_buffer], &mut [out], None)
						.unwrap();
				});
		}
	}

	/// Get the latency ofthis [`Oversampler`].
	pub fn latency(&self) -> usize {
		self.block_size
			+ self.upsampler.output_delay() / self.ratio as usize
			+ self.downsampler.output_delay()
	}

	pub fn reset(&mut self) {
		self.upsampler.reset();
		self.downsampler.reset();
		self.buffered.reset();
	}

	/// Retreive the configured block size.
	pub fn block_size(&self) -> usize {
		self.block_size
	}

	/// Retreive the configured original sample rate.
	pub fn orig_sample_rate(&self) -> usize {
		self.orig_sample_rate
	}

	/// Retreive the configured oversampling ratio.
	pub fn ratio(&self) -> u8 {
		self.ratio
	}
}

/// An oversampler you can change the ratio of.
///
/// This wraps an [`Oversampler`] per oversampling factor requested in [`AdjustableOversampler::new`].
/// That is, for a maximum factor of 3, this will wrap 3 [Oversamplers][Oversampler].
/// Note that the memory use grows exponentially as more oversamplers are used.
pub struct AdjustableOversampler {
	orig_sample_rate: usize,
	block_size: usize,
	factor: u8,
	oversamplers: Box<[Oversampler]>,
}

impl AdjustableOversampler {
	/// Create a new [`AdjustableOversampler`].
	///
	/// # Parameters
	///
	/// - `block_size`: How many samples should be buffered. Lower values decrease latency but increase CPU load, higher values do the opposite. A good value is whatever the host provides you with, or half that. Panics if `0`.
	/// - `orig_sample_rate`: The sample rate of the input. Panics if `0`.
	/// - `max_factor`: Maximum oversampling factor. The actual oversampling amount is `2 ^ factor`. Panics if Greater than `7`.
	/// - `initial_factor`: The initial oversampling factor. Panics if `initial_factor > max_factor`.
	pub fn new(
		block_size: usize,
		orig_sample_rate: usize,
		max_factor: u8,
		initial_factor: u8,
	) -> Self {
		assert!(
			max_factor <= 7,
			"The maximum oversampling factor `max_factor` can't be greater than 7"
		);
		assert!(
			initial_factor <= max_factor,
			"`initial_factor` can't be greater than `max_factor`"
		);
		assert_ne!(block_size, 0);
		assert_ne!(orig_sample_rate, 0);

		Self {
			orig_sample_rate,
			block_size,
			factor: initial_factor,
			oversamplers: (0..max_factor)
				.map(|f| Oversampler::new(block_size, orig_sample_rate, u8::pow(2, f as u32 + 1)))
				.collect(),
		}
	}

	/// Process a block of samples at a higher sampling rate, unless the active oversampling factor is `0`.
	pub fn process_block<F>(&mut self, samples: &mut [f32], mut f: F)
	where
		F: FnMut(&mut [f32]),
	{
		if self.factor == 0 {
			f(samples);
			return;
		}

		self.oversamplers[(self.factor - 1) as usize].process_block(samples, f);
	}

	/// Get the latency of this [`AdjustableOversampler`].
	///
	/// This value will change depending on the active oversampling factor.
	pub fn latency(&self) -> usize {
		if self.factor == 0 {
			0
		} else {
			self.oversamplers[(self.factor - 1) as usize].latency()
		}
	}

	/// Reset state associated with this [`AdjustableOversampler`].
	pub fn reset(&mut self) {
		if self.factor > 0 {
			self.oversamplers[(self.factor - 1) as usize].reset();
		}
	}

	/// Set the active oversampling factor.
	///
	/// Does nothing if the active factor is equal to `factor`, otherwise resets state as well.
	///
	/// Panics if `factor` is greater than `max_factor` provided in [`AdjustableOversampler::new`].
	pub fn set_oversampling_factor(&mut self, factor: u8) {
		assert!(
			factor as usize <= self.oversamplers.len(),
			"`factor` can't be greater than `max_factor`"
		);
		if self.factor != factor {
			self.factor = factor;
			self.reset();
		}
	}

	/// Retreive the configured maximum oversampling factor.
	pub fn max_factor(&self) -> u8 {
		self.oversamplers.len() as _
	}

	/// Retreive the configured original sample rate.
	pub fn orig_sample_rate(&self) -> usize {
		self.orig_sample_rate
	}

	/// Retreive the active oversampling factor.
	pub fn oversampling_factor(&self) -> u8 {
		self.factor
	}

	/// Retreive the configured block size.
	pub fn block_size(&self) -> usize {
		self.block_size
	}
}
