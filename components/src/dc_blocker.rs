// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

use std::ops::{
	Deref,
	DerefMut,
};

use crate::{
	Component,
	ComponentBlock,
	ComponentMeta,
	Simper,
};

// A Simper based DC blocker, which is just a 5hz high-pass filter.
#[derive(Debug, Clone)]
pub struct DcBlocker {
	filter: Simper,
}

impl ComponentMeta for DcBlocker {
	fn reset(&mut self) {
		self.filter.reset();
	}

	fn latency(&self) -> usize {
		self.filter.latency()
	}
}

impl Component for DcBlocker {
	fn process(&mut self, sample: f64) -> f64 {
		self.filter.process(sample)
	}
}

impl DcBlocker {
	pub fn new(sample_rate: f64) -> Self {
		Self {
			filter: Simper::high_pass(sample_rate, 5.0, Simper::BUTTERWORTH_Q),
		}
	}

	pub fn set_sample_rate(&mut self, sample_rate: f64) {
		self.filter = Simper::high_pass(sample_rate, 5.0, Simper::BUTTERWORTH_Q);
	}
}

#[derive(Debug, Clone)]
pub struct DcBlocked<C> {
	blocker: DcBlocker,
	pub inner: C,
}

impl<C> Deref for DcBlocked<C> {
	type Target = C;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl<C> DerefMut for DcBlocked<C> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.inner
	}
}

impl<C: ComponentMeta> ComponentMeta for DcBlocked<C> {
	fn latency(&self) -> usize {
		self.blocker.latency() + self.inner.latency()
	}

	fn reset(&mut self) {
		self.blocker.reset();
		self.inner.reset();
	}
}

impl<C: Component> Component for DcBlocked<C> {
	#[inline]
	fn process(&mut self, sample: f64) -> f64 {
		self.blocker.process(self.inner.process(sample))
	}
}

impl<C: ComponentBlock> ComponentBlock for DcBlocked<C> {
	#[inline]
	fn process_block(&mut self, samples: &mut [f32]) {
		self.inner.process_block(samples);
		self.blocker.process_block(samples);
	}
}
