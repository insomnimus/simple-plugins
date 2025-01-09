// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

use crate::{
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
