// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

use core::ops::{
	Deref,
	DerefMut,
};

use crate::{
	f64x2,
	Component,
	ComponentMeta,
	SimdFloat,
};

/// A wrapped function, to implement [Component].
#[derive(Debug)]
pub struct Func<F>(pub F);

impl<F> ComponentMeta for Func<F> {}

impl<T: SimdFloat, F: FnMut(T) -> T> Component<T> for Func<F> {
	fn process(&mut self, sample: T) -> T {
		(self.0)(sample)
	}
}

/// Wraps 2 mono components, to make them compatible for [`Component`]`<`[`f64x2`]`>`.
#[derive(Debug, Clone)]
pub struct Stereo<L, R> {
	pub left: L,
	pub right: R,
}

/// An alias for [`Stereo`] where both the left and right channel components are of the same type.
pub type DoubleMono<C> = Stereo<C, C>;

impl<L: ComponentMeta, R: ComponentMeta> ComponentMeta for Stereo<L, R> {
	fn reset(&mut self) {
		self.left.reset();
		self.right.reset();
	}

	fn latency(&self) -> usize {
		usize::max(self.left.latency(), self.right.latency())
	}
}

impl<L: Component<f64>, R: Component<f64>> Component<f64x2> for Stereo<L, R> {
	fn process(&mut self, sample: f64x2) -> f64x2 {
		let [l, r] = sample.value();

		f64x2::new([self.left.process(l), self.right.process(r)])
	}
}

impl<C: Clone> Stereo<C, C> {
	/// Create a new [`DoubleMono<C, C>`][Self] by cloning `component` once.
	pub fn double(component: C) -> Self {
		Self {
			left: component.clone(),
			right: component,
		}
	}
}

/// A [`Component`] that can be (manually) toggled on / off.
#[derive(Debug, Clone)]
pub struct Toggle<C> {
	active: bool,
	/// If set to `true`, when toggling from bypassed to enabled, `self.inner.reset()` will be invoked.
	pub reset_on_toggle: bool,
	/// The inner [`Component`].
	pub inner: C,
}

impl<C: ComponentMeta> ComponentMeta for Toggle<C> {
	fn reset(&mut self) {
		self.inner.reset();
	}

	fn latency(&self) -> usize {
		if self.active {
			self.inner.latency()
		} else {
			0
		}
	}
}

impl<T, C: Component<T>> Component<T> for Toggle<C> {
	fn process(&mut self, sample: T) -> T {
		if self.active {
			self.inner.process(sample)
		} else {
			sample
		}
	}
}

impl<C: ComponentMeta> Toggle<C> {
	/// Create a new [`Toggle<C>`][Self].
	pub fn new(inner: C, start_active: bool, reset_on_toggle: bool) -> Self {
		Self {
			active: start_active,
			reset_on_toggle,
			inner,
		}
	}

	pub fn toggle(&mut self, active: bool) {
		if active && !self.active && self.reset_on_toggle {
			self.inner.reset();
		}
		self.active = active;
	}
}

impl<C> Deref for Toggle<C> {
	type Target = C;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl<C> DerefMut for Toggle<C> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.inner
	}
}
