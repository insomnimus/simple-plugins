use core::array;

use crate::simper::{
	Simper,
	BUTTERWORTH_Q,
};

/// 4 cascaded Simper low-pass filters.
#[derive(Debug, Clone)]
struct LowPass {
	filters: [Simper; 4],
}

impl LowPass {
	fn new(sr: f64, cutoff: f64) -> Self {
		Self {
			filters: core::array::from_fn(move |_| Simper::low_pass(sr, cutoff, BUTTERWORTH_Q)),
		}
	}

	fn process(&mut self, mut sample: f64) -> f64 {
		for f in &mut self.filters {
			sample = f.process(sample);
		}

		sample
	}

	fn reset(&mut self) {
		for f in &mut self.filters {
			f.reset();
		}
	}
}

#[derive(Debug, Clone)]
struct Inner<const N: usize> {
	buffers: [Vec<f32>; N],
	down_filters: [LowPass; N],
	up_filters: [LowPass; N],
}

impl<const N: usize> Inner<N> {
	fn new(sr: f64, block_size: usize) -> Self {
		// Cut just below nyquist
		let cutoff = sr * 0.5 * 0.98;
		let filters =
			array::from_fn(|i| LowPass::new(sr * usize::pow(2, i as u32 + 1) as f64, cutoff));

		Self {
			buffers: array::from_fn(|i| {
				Vec::with_capacity(usize::pow(2, i as u32 + 1) * block_size)
			}),
			down_filters: filters.clone(),
			up_filters: filters,
		}
	}

	fn set_sample_rate(&mut self, sr: f64) {
		let cutoff = sr * 0.5 * 0.98;
		for (i, f) in self.up_filters.iter_mut().enumerate() {
			*f = LowPass::new(sr * usize::pow(2, i as u32 + 1) as f64, cutoff);
		}
		for (i, f) in self.down_filters.iter_mut().enumerate() {
			*f = LowPass::new(sr * usize::pow(2, i as u32 + 1) as f64, cutoff);
		}
	}

	fn upsample(&mut self, input: &[f32], times: usize) -> &mut [f32] {
		debug_assert!(times <= N);
		debug_assert_ne!(times, 0);

		// First stage
		let f = &mut self.up_filters[0];
		let buffer = &mut self.buffers[0];
		buffer.clear();

		for sample in input {
			buffer.push(f.process(*sample as f64) as _);
			buffer.push(f.process(0.0) as _);
		}

		for stage in 1..times {
			let [ref mut input, ref mut buffer] = &mut self.buffers[stage - 1..stage + 1] else {
				unreachable!();
			};

			let f = &mut self.up_filters[stage];
			buffer.clear();

			for sample in input {
				buffer.push(f.process(*sample as _) as _);
				buffer.push(f.process(0.0) as _);
			}
		}

		&mut self.buffers[times - 1]
	}

	fn downsample(&mut self, output: &mut [f32], times: usize) {
		debug_assert!(times <= N);
		debug_assert_ne!(times, 0);

		for stage in (1..times).rev() {
			let f = &mut self.down_filters[stage];
			let [ref mut output, ref mut buffer] = &mut self.buffers[stage - 1..stage + 1] else {
				unreachable!();
			};

			for (i, out_sample) in output.iter_mut().enumerate() {
				*out_sample = f.process(buffer[i * 2] as _) as _;
			}
		}

		// First (last) stage
		let f = &mut self.down_filters[0];
		let buffer = &self.buffers[0];
		for (i, out_sample) in output.iter_mut().enumerate() {
			*out_sample = f.process(buffer[i * 2] as _) as _;
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

#[derive(Debug, Clone)]
pub struct Oversampler<const N: usize> {
	block_size: usize,
	inner: Inner<N>,
}

impl<const N: usize> Oversampler<N> {
	pub fn new(sample_rate: f64, block_size: usize) -> Self {
		assert_ne!(block_size, 0);

		Self {
			block_size,
			inner: Inner::new(sample_rate, block_size),
		}
	}

	pub fn process_block<F>(&mut self, samples: &mut [f32], times: usize, mut process: F)
	where
		F: for<'a> FnMut(&'a mut [f32]),
	{
		debug_assert!(times <= N);

		if times == 0 {
			return;
		}

		let times = usize::min(times, N);

		for chunk in samples.chunks_mut(self.block_size) {
			process(self.inner.upsample(chunk, times));
			self.inner.downsample(chunk, times);
		}
	}

	pub fn reset(&mut self) {
		self.inner.reset();
	}

	pub fn set_sample_rate(&mut self, sample_rate: f64) {
		self.inner.set_sample_rate(sample_rate);
	}
}
