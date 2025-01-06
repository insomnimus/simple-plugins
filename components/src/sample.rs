// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

use core::ops::{
	Add,
	Div,
	Mul,
	Sub,
};

pub trait Sample: Copy + Add + Sub + Mul + Div {}

macro_rules! gen_impls {
	[$($t:ty),+ $(,)?] => {
		$(impl Sample for $t {})+
	}
}

gen_impls!(f32, f64);
