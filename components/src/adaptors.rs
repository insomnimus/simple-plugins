// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

use crate::{
	ComponentBlock,
	ComponentMeta,
};

/// Make a [ComponentBlock] out of a function.
#[derive(Debug, Clone)]
pub struct BlockProcess<F> {
	pub func: F,
	pub latency: usize,
}

impl<F> BlockProcess<F> {
	pub fn new(func: F) -> Self {
		Self { func, latency: 0 }
	}

	pub fn with_latency(mut self, latency: usize) -> Self {
		self.latency = latency;
		self
	}
}

impl<F: FnMut(&mut [f32])> ComponentBlock for BlockProcess<F> {
	fn process_block(&mut self, samples: &mut [f32]) {
		(self.func)(samples);
	}
}

impl<F> ComponentMeta for BlockProcess<F> {
	fn latency(&self) -> usize {
		self.latency
	}
}
