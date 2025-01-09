// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

//! Adapted from https://www.musicdsp.org/en/latest/Filters/25-moog-vcf-variation-1.html

use crate::{
	replace_float_literals,
	Component,
	ComponentMeta,
	SimdFloat,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum MoogType {
	LowPass,
	BandPass,
	HighPass,
}

pub struct Moog<T> {
	pub filter_type: MoogType,
	f: T,
	p: T,
	q: T,
	b: [T; 5],
}

impl<T: SimdFloat> Moog<T> {
	#[replace_float_literals(T)]
	fn new(filter_type: MoogType, sample_rate: T, fq: T, res: T) -> Self {
		let fq = fq / (sample_rate * 0.5);

		let q = 1.0 - fq;
		let p = fq + 0.8 * fq * q;
		let f = p + p - 1.0;
		let q = res * (1.0 + 0.5 * q * (1.0 - q + 5.6 * q * q));

		Self {
			filter_type,
			q,
			p,
			f,
			b: [0.0; 5],
		}
	}

	#[inline]
	#[replace_float_literals(T)]
	fn consume_sample(&mut self, sample: T) -> T {
		let sample = sample - self.q * self.b[4]; //feedback
		let b = &mut self.b;

		let mut t1 = b[1];
		b[1] = (sample + b[0]) * self.p - b[1] * self.f;

		let t2 = b[2];
		b[2] = (b[1] + t1) * self.p - b[2] * self.f;

		t1 = b[3];
		b[3] = (b[2] + t2) * self.p - b[3] * self.f;

		b[4] = (b[3] + t1) * self.p - b[4] * self.f;
		b[4] = b[4] - b[4] * b[4] * b[4] * 0.166667; //clipping

		b[0] = sample;

		// Lowpass  output:  b4
		// Highpass output:  in - b4;
		// Bandpass output:  3.0f * (b3 - b4);
		sample
	}

	pub fn low_pass(sample_rate: T, fq: T, res: T) -> Self {
		Self::new(MoogType::LowPass, sample_rate, fq, res)
	}

	pub fn high_pass(sample_rate: T, fq: T, res: T) -> Self {
		Self::new(MoogType::HighPass, sample_rate, fq, res)
	}

	pub fn band_pass(sample_rate: T, fq: T, res: T) -> Self {
		Self::new(MoogType::BandPass, sample_rate, fq, res)
	}
}

impl<T: SimdFloat> ComponentMeta for Moog<T> {
	fn reset(&mut self) {
		self.b = [T::ZERO; 5];
	}

	fn latency(&self) -> usize {
		4
	}
}

impl<T: SimdFloat> Component<T> for Moog<T> {
	fn process(&mut self, sample: T) -> T {
		let sample = self.consume_sample(sample);

		match self.filter_type {
			MoogType::LowPass => self.b[4],
			MoogType::HighPass => sample - self.b[4],
			MoogType::BandPass => T::splat(3.0) * (self.b[3] - self.b[4]),
		}
	}
}
