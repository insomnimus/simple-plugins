// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

use core::ops::{
	Deref,
	DerefMut,
};

use crate::{
	Component,
	ComponentMeta,
};

/// Cascade a [Component] several times.
#[derive(Debug, Copy, Clone)]
pub struct Cascade<C, const N: usize>(pub [C; N]);

impl<C: ComponentMeta, const N: usize> ComponentMeta for Cascade<C, N> {
	fn reset(&mut self) {
		for c in &mut self.0 {
			c.reset();
		}
	}

	fn latency(&self) -> usize {
		self.0.iter().map(C::latency).sum()
	}
}

impl<T, C: Component<T>, const N: usize> Component<T> for Cascade<C, N> {
	#[inline]
	fn process(&mut self, mut sample: T) -> T {
		for f in &mut self.0 {
			sample = f.process(sample);
		}

		sample
	}
}

impl<'a, C, const N: usize> IntoIterator for &'a mut Cascade<C, N> {
	type IntoIter = core::slice::IterMut<'a, C>;
	type Item = &'a mut C;

	fn into_iter(self) -> Self::IntoIter {
		self.0.iter_mut()
	}
}

impl<'a, C, const N: usize> IntoIterator for &'a Cascade<C, N> {
	type IntoIter = core::slice::Iter<'a, C>;
	type Item = &'a C;

	fn into_iter(self) -> Self::IntoIter {
		self.0.iter()
	}
}

impl<C, const N: usize> Deref for Cascade<C, N> {
	type Target = [C];

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<C, const N: usize> DerefMut for Cascade<C, N> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl<C, const N: usize> Cascade<C, N> {
	/// Apply `f` to every item in this [`Cascade`].
	pub fn apply<F>(&mut self, mut f: F)
	where
		F: FnMut(&mut C),
	{
		for x in &mut self.0 {
			f(x);
		}
	}
}

impl<C: Clone, const N: usize> Cascade<C, N> {
	/// Create a [`Cascade`] like [`std::array::from_fn`].
	pub fn from_fn<F>(f: F) -> Self
	where
		F: FnMut(usize) -> C,
	{
		Self(core::array::from_fn(f))
	}
}
