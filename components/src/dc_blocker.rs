// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

use std::ops::{
	Deref,
	DerefMut,
};

use crate::{
	Component,
	ComponentMeta,
	SimdFloat,
	Simper,
};

/// A [`Simper`] based DC blocker, which is just a 5hz high-pass filter.
#[derive(Debug, Clone)]
pub struct DcBlocker<T> {
	filter: Simper<T>,
}

impl<T: SimdFloat> ComponentMeta for DcBlocker<T> {
	fn reset(&mut self) {
		self.filter.reset();
	}

	fn latency(&self) -> usize {
		self.filter.latency()
	}
}

impl<T: SimdFloat> Component<T> for DcBlocker<T> {
	fn process(&mut self, sample: T) -> T {
		self.filter.process(sample)
	}
}

impl<T: SimdFloat> DcBlocker<T> {
	pub fn new(sample_rate: f64) -> Self {
		Self {
			filter: Simper::high_pass(T::splat(sample_rate), T::splat(5.0), Simper::BUTTERWORTH_Q),
		}
	}

	pub fn set_sample_rate(&mut self, sample_rate: T) {
		self.filter = Simper::high_pass(sample_rate, T::splat(5.0), Simper::BUTTERWORTH_Q);
	}
}

/// Wraps a [`Component`] with a [`DcBlocker`] after it.
#[derive(Debug, Clone)]
pub struct DcBlocked<T, C> {
	blocker: DcBlocker<T>,
	pub inner: C,
}

impl<T, C> Deref for DcBlocked<T, C> {
	type Target = C;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl<T, C> DerefMut for DcBlocked<T, C> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.inner
	}
}

impl<T: SimdFloat, C: ComponentMeta> ComponentMeta for DcBlocked<T, C> {
	fn latency(&self) -> usize {
		self.blocker.latency() + self.inner.latency()
	}

	fn reset(&mut self) {
		self.blocker.reset();
		self.inner.reset();
	}
}

impl<T: SimdFloat, C: Component<T>> Component<T> for DcBlocked<T, C> {
	#[inline]
	fn process(&mut self, sample: T) -> T {
		self.blocker.process(self.inner.process(sample))
	}
}
