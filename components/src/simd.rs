// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

//! SIMD abstractions.
//!
//! Most of this is manually generated code.

use core::{
	f32,
	f64,
	ops::*,
};

use wide::*;

/// Non-vector float abstraction: implemented for [`prim@f32`] and [`prim@f64`].
pub trait ScalarFloat:
	SimdFloat + PartialEq<Self> + PartialOrd<Self> + Rem<Self> + RemAssign<Self> + Into<f64> + From<f32>
{
	fn to_f64(self) -> f64 {
		self.into()
	}
}

impl ScalarFloat for f32 {}
impl ScalarFloat for f64 {}

/// Float abstraction: implemented for [`prim@f32`], [`prim@f64`], [`f32x4`] and [`f64x2`].
pub trait SimdFloat:
	Copy
	+ Neg<Output = Self>
	+ Add<Output = Self>
	+ Sub<Output = Self>
	+ Mul<Output = Self>
	+ Div<Output = Self>
	+ AddAssign
	+ SubAssign
	+ MulAssign
	+ DivAssign
{
	type Value;

	// consts {
	const E: Self;
	const FRAC_1_PI: Self;
	const FRAC_1_SQRT_2: Self;
	const FRAC_2_PI: Self;
	const FRAC_2_SQRT_PI: Self;
	const FRAC_PI_2: Self;
	const FRAC_PI_3: Self;
	const FRAC_PI_4: Self;
	const FRAC_PI_6: Self;
	const FRAC_PI_8: Self;
	const HALF: Self;
	const LN_10: Self;
	const LN_2: Self;
	const LOG10_2: Self;
	const LOG10_E: Self;
	const LOG2_10: Self;
	const LOG2_E: Self;
	const ONE: Self;
	const PI: Self;
	const SQRT_2: Self;
	const TAU: Self;
	const ZERO: Self;
	// }

	// Comparison methods {
	fn all_eq(self, rhs: Self) -> bool;
	fn any_eq(self, rhs: Self) -> bool;
	fn none_eq(self, rhs: Self) -> bool;
	fn all_ne(self, rhs: Self) -> bool;
	fn any_ne(self, rhs: Self) -> bool;
	fn none_ne(self, rhs: Self) -> bool;
	fn all_le(self, rhs: Self) -> bool;
	fn any_le(self, rhs: Self) -> bool;
	fn none_le(self, rhs: Self) -> bool;
	fn all_lt(self, rhs: Self) -> bool;
	fn any_lt(self, rhs: Self) -> bool;
	fn none_lt(self, rhs: Self) -> bool;
	fn all_ge(self, rhs: Self) -> bool;
	fn any_ge(self, rhs: Self) -> bool;
	fn none_ge(self, rhs: Self) -> bool;
	fn all_gt(self, rhs: Self) -> bool;
	fn any_gt(self, rhs: Self) -> bool;
	fn none_gt(self, rhs: Self) -> bool;
	// }

	// methods {
	fn value(self) -> Self::Value;
	fn new(val: Self::Value) -> Self;
	fn first_as_f64(self) -> f64;

	fn from_f32(val: f32) -> Self {
		Self::splat(val as f64)
	}

	fn abs(self) -> Self;
	fn acos(self) -> Self;
	fn asin(self) -> Self;
	fn atan(self) -> Self;
	fn atan2(self, x: Self) -> Self;
	fn ceil(self) -> Self;
	fn copysign(self, sign: Self) -> Self;
	fn cos(self) -> Self;
	fn floor(self) -> Self;
	fn ln(self) -> Self;
	fn log10(self) -> Self;
	fn log2(self) -> Self;
	fn max(self, rhs: Self) -> Self;
	fn min(self, rhs: Self) -> Self;
	fn mul_add(self, m: Self, a: Self) -> Self;
	fn mul_sub(self, m: Self, a: Self) -> Self;
	fn pow(self, y: Self) -> Self;
	fn round(self) -> Self;
	fn sin(self) -> Self;
	fn sin_cos(self) -> (Self, Self);
	fn splat(elem: f64) -> Self;
	fn sqrt(self) -> Self;
	fn tan(self) -> Self;
	fn to_degrees(self) -> Self;
	fn to_radians(self) -> Self;

	fn clamp(self, min: Self, max: Self) -> Self {
		self.max(min).min(max)
	}
	// }
}

impl SimdFloat for f64x2 {
	type Value = [f64; 2];

	// consts {
	const E: Self = f64x2::E;
	const FRAC_1_PI: Self = f64x2::FRAC_1_PI;
	const FRAC_1_SQRT_2: Self = f64x2::FRAC_1_SQRT_2;
	const FRAC_2_PI: Self = f64x2::FRAC_2_PI;
	const FRAC_2_SQRT_PI: Self = f64x2::FRAC_2_SQRT_PI;
	const FRAC_PI_2: Self = f64x2::FRAC_PI_2;
	const FRAC_PI_3: Self = f64x2::FRAC_PI_3;
	const FRAC_PI_4: Self = f64x2::FRAC_PI_4;
	const FRAC_PI_6: Self = f64x2::FRAC_PI_6;
	const FRAC_PI_8: Self = f64x2::FRAC_PI_8;
	const HALF: Self = f64x2::HALF;
	const LN_10: Self = f64x2::LN_10;
	const LN_2: Self = f64x2::LN_2;
	const LOG10_2: Self = f64x2::LOG10_2;
	const LOG10_E: Self = f64x2::LOG10_E;
	const LOG2_10: Self = f64x2::LOG2_10;
	const LOG2_E: Self = f64x2::LOG2_E;
	const ONE: Self = f64x2::ONE;
	const PI: Self = f64x2::PI;
	const SQRT_2: Self = f64x2::SQRT_2;
	const TAU: Self = f64x2::TAU;
	const ZERO: Self = f64x2::ZERO;

	// }

	// Comparison methods {
	#[inline]
	fn all_eq(self, rhs: Self) -> bool {
		self.cmp_eq(rhs).all()
	}

	#[inline]
	fn none_eq(self, rhs: Self) -> bool {
		self.cmp_eq(rhs).none()
	}

	#[inline]
	fn any_eq(self, rhs: Self) -> bool {
		self.cmp_eq(rhs).any()
	}

	#[inline]
	fn all_ne(self, rhs: Self) -> bool {
		self.cmp_ne(rhs).all()
	}

	#[inline]
	fn none_ne(self, rhs: Self) -> bool {
		self.cmp_ne(rhs).none()
	}

	#[inline]
	fn any_ne(self, rhs: Self) -> bool {
		self.cmp_ne(rhs).any()
	}

	#[inline]
	fn all_gt(self, rhs: Self) -> bool {
		self.cmp_gt(rhs).all()
	}

	#[inline]
	fn none_gt(self, rhs: Self) -> bool {
		self.cmp_gt(rhs).none()
	}

	#[inline]
	fn any_gt(self, rhs: Self) -> bool {
		self.cmp_gt(rhs).any()
	}

	#[inline]
	fn all_ge(self, rhs: Self) -> bool {
		self.cmp_ge(rhs).all()
	}

	#[inline]
	fn none_ge(self, rhs: Self) -> bool {
		self.cmp_ge(rhs).none()
	}

	#[inline]
	fn any_ge(self, rhs: Self) -> bool {
		self.cmp_ge(rhs).any()
	}

	#[inline]
	fn all_lt(self, rhs: Self) -> bool {
		self.cmp_lt(rhs).all()
	}

	#[inline]
	fn none_lt(self, rhs: Self) -> bool {
		self.cmp_lt(rhs).none()
	}

	#[inline]
	fn any_lt(self, rhs: Self) -> bool {
		self.cmp_lt(rhs).any()
	}

	#[inline]
	fn all_le(self, rhs: Self) -> bool {
		self.cmp_le(rhs).all()
	}

	#[inline]
	fn none_le(self, rhs: Self) -> bool {
		self.cmp_le(rhs).none()
	}

	#[inline]
	fn any_le(self, rhs: Self) -> bool {
		self.cmp_le(rhs).any()
	}

	// }

	// methods {
	#[inline]
	fn value(self) -> Self::Value {
		self.to_array()
	}

	fn new(value: Self::Value) -> Self {
		Self::new(value)
	}

	#[inline]
	fn first_as_f64(self) -> f64 {
		self.to_array()[0]
	}

	#[inline]
	fn abs(self) -> Self {
		self.abs()
	}

	#[inline]
	fn acos(self) -> Self {
		self.acos()
	}

	// }

	#[inline]
	fn asin(self) -> Self {
		self.asin()
	}

	#[inline]
	fn atan(self) -> Self {
		self.atan()
	}

	#[inline]
	fn atan2(self, x: Self) -> Self {
		self.atan2(x)
	}

	#[inline]
	fn ceil(self) -> Self {
		self.ceil()
	}

	#[inline]
	fn copysign(self, sign: Self) -> Self {
		self.copysign(sign)
	}

	#[inline]
	fn cos(self) -> Self {
		self.cos()
	}

	#[inline]
	fn floor(self) -> Self {
		self.floor()
	}

	#[inline]
	fn ln(self) -> Self {
		self.ln()
	}

	#[inline]
	fn log10(self) -> Self {
		self.log10()
	}

	#[inline]
	fn log2(self) -> Self {
		self.log2()
	}

	#[inline]
	fn max(self, rhs: Self) -> Self {
		self.fast_max(rhs)
	}

	#[inline]
	fn min(self, rhs: Self) -> Self {
		self.fast_min(rhs)
	}

	#[inline]
	fn mul_add(self, m: Self, a: Self) -> Self {
		self.mul_add(m, a)
	}

	#[inline]
	fn mul_sub(self, m: Self, a: Self) -> Self {
		self.mul_sub(m, a)
	}

	#[inline]
	fn pow(self, y: Self) -> Self {
		self.pow_f64x2(y)
	}

	#[inline]
	fn round(self) -> Self {
		self.round()
	}

	#[inline]
	fn sin(self) -> Self {
		self.sin()
	}

	#[inline]
	fn sin_cos(self) -> (Self, Self) {
		self.sin_cos()
	}

	#[inline]
	fn splat(elem: f64) -> Self {
		Self::splat(elem)
	}

	#[inline]
	fn sqrt(self) -> Self {
		self.sqrt()
	}

	#[inline]
	fn tan(self) -> Self {
		self.tan()
	}

	#[inline]
	fn to_degrees(self) -> Self {
		self.to_degrees()
	}

	#[inline]
	fn to_radians(self) -> Self {
		self.to_radians()
	}
	// }
}

impl SimdFloat for f32x4 {
	type Value = [f32; 4];

	// consts {
	const E: Self = f32x4::E;
	const FRAC_1_PI: Self = f32x4::FRAC_1_PI;
	const FRAC_1_SQRT_2: Self = f32x4::FRAC_1_SQRT_2;
	const FRAC_2_PI: Self = f32x4::FRAC_2_PI;
	const FRAC_2_SQRT_PI: Self = f32x4::FRAC_2_SQRT_PI;
	const FRAC_PI_2: Self = f32x4::FRAC_PI_2;
	const FRAC_PI_3: Self = f32x4::FRAC_PI_3;
	const FRAC_PI_4: Self = f32x4::FRAC_PI_4;
	const FRAC_PI_6: Self = f32x4::FRAC_PI_6;
	const FRAC_PI_8: Self = f32x4::FRAC_PI_8;
	const HALF: Self = f32x4::HALF;
	const LN_10: Self = f32x4::LN_10;
	const LN_2: Self = f32x4::LN_2;
	const LOG10_2: Self = f32x4::LOG10_2;
	const LOG10_E: Self = f32x4::LOG10_E;
	const LOG2_10: Self = f32x4::LOG2_10;
	const LOG2_E: Self = f32x4::LOG2_E;
	const ONE: Self = f32x4::ONE;
	const PI: Self = f32x4::PI;
	const SQRT_2: Self = f32x4::SQRT_2;
	const TAU: Self = f32x4::TAU;
	const ZERO: Self = f32x4::ZERO;

	// }

	// Comparison methods {
	#[inline]
	fn all_eq(self, rhs: Self) -> bool {
		self.cmp_eq(rhs).all()
	}

	#[inline]
	fn none_eq(self, rhs: Self) -> bool {
		self.cmp_eq(rhs).none()
	}

	#[inline]
	fn any_eq(self, rhs: Self) -> bool {
		self.cmp_eq(rhs).any()
	}

	#[inline]
	fn all_ne(self, rhs: Self) -> bool {
		self.cmp_ne(rhs).all()
	}

	#[inline]
	fn none_ne(self, rhs: Self) -> bool {
		self.cmp_ne(rhs).none()
	}

	#[inline]
	fn any_ne(self, rhs: Self) -> bool {
		self.cmp_ne(rhs).any()
	}

	#[inline]
	fn all_gt(self, rhs: Self) -> bool {
		self.cmp_gt(rhs).all()
	}

	#[inline]
	fn none_gt(self, rhs: Self) -> bool {
		self.cmp_gt(rhs).none()
	}

	#[inline]
	fn any_gt(self, rhs: Self) -> bool {
		self.cmp_gt(rhs).any()
	}

	#[inline]
	fn all_ge(self, rhs: Self) -> bool {
		self.cmp_ge(rhs).all()
	}

	#[inline]
	fn none_ge(self, rhs: Self) -> bool {
		self.cmp_ge(rhs).none()
	}

	#[inline]
	fn any_ge(self, rhs: Self) -> bool {
		self.cmp_ge(rhs).any()
	}

	#[inline]
	fn all_lt(self, rhs: Self) -> bool {
		self.cmp_lt(rhs).all()
	}

	#[inline]
	fn none_lt(self, rhs: Self) -> bool {
		self.cmp_lt(rhs).none()
	}

	#[inline]
	fn any_lt(self, rhs: Self) -> bool {
		self.cmp_lt(rhs).any()
	}

	#[inline]
	fn all_le(self, rhs: Self) -> bool {
		self.cmp_le(rhs).all()
	}

	#[inline]
	fn none_le(self, rhs: Self) -> bool {
		self.cmp_le(rhs).none()
	}

	#[inline]
	fn any_le(self, rhs: Self) -> bool {
		self.cmp_le(rhs).any()
	}

	// }

	// methods {
	#[inline]
	fn value(self) -> Self::Value {
		self.to_array()
	}

	fn new(value: Self::Value) -> Self {
		Self::new(value)
	}

	#[inline]
	fn first_as_f64(self) -> f64 {
		self.to_array()[0] as f64
	}

	#[inline]
	fn abs(self) -> Self {
		self.abs()
	}

	#[inline]
	fn acos(self) -> Self {
		self.acos()
	}

	#[inline]
	fn asin(self) -> Self {
		self.asin()
	}

	#[inline]
	fn atan(self) -> Self {
		self.atan()
	}

	#[inline]
	fn atan2(self, x: Self) -> Self {
		self.atan2(x)
	}

	#[inline]
	fn ceil(self) -> Self {
		self.ceil()
	}

	#[inline]
	fn copysign(self, sign: Self) -> Self {
		self.copysign(sign)
	}

	#[inline]
	fn cos(self) -> Self {
		self.cos()
	}

	#[inline]
	fn floor(self) -> Self {
		self.floor()
	}

	#[inline]
	fn ln(self) -> Self {
		self.ln()
	}

	#[inline]
	fn log10(self) -> Self {
		self.log10()
	}

	#[inline]
	fn log2(self) -> Self {
		self.log2()
	}

	#[inline]
	fn max(self, rhs: Self) -> Self {
		self.fast_max(rhs)
	}

	#[inline]
	fn min(self, rhs: Self) -> Self {
		self.fast_min(rhs)
	}

	#[inline]
	fn mul_add(self, m: Self, a: Self) -> Self {
		self.mul_add(m, a)
	}

	#[inline]
	fn mul_sub(self, m: Self, a: Self) -> Self {
		self.mul_sub(m, a)
	}

	#[inline]
	fn pow(self, y: Self) -> Self {
		self.pow_f32x4(y)
	}

	#[inline]
	fn round(self) -> Self {
		self.round()
	}

	#[inline]
	fn sin(self) -> Self {
		self.sin()
	}

	#[inline]
	fn sin_cos(self) -> (Self, Self) {
		self.sin_cos()
	}

	#[inline]
	fn splat(elem: f64) -> Self {
		Self::splat(elem as f32)
	}

	#[inline]
	fn sqrt(self) -> Self {
		self.sqrt()
	}

	#[inline]
	fn tan(self) -> Self {
		self.tan()
	}

	#[inline]
	fn to_degrees(self) -> Self {
		self.to_degrees()
	}

	#[inline]
	fn to_radians(self) -> Self {
		self.to_radians()
	}

	// }
}

impl SimdFloat for f64 {
	type Value = f64;

	// consts {
	const E: Self = f64::consts::E;
	const FRAC_1_PI: Self = f64::consts::FRAC_1_PI;
	const FRAC_1_SQRT_2: Self = f64::consts::FRAC_1_SQRT_2;
	const FRAC_2_PI: Self = f64::consts::FRAC_2_PI;
	const FRAC_2_SQRT_PI: Self = f64::consts::FRAC_2_SQRT_PI;
	const FRAC_PI_2: Self = f64::consts::FRAC_PI_2;
	const FRAC_PI_3: Self = f64::consts::FRAC_PI_3;
	const FRAC_PI_4: Self = f64::consts::FRAC_PI_4;
	const FRAC_PI_6: Self = f64::consts::FRAC_PI_6;
	const FRAC_PI_8: Self = f64::consts::FRAC_PI_8;
	const HALF: Self = 0.5;
	const LN_10: Self = f64::consts::LN_10;
	const LN_2: Self = f64::consts::LN_2;
	const LOG10_2: Self = f64::consts::LOG10_2;
	const LOG10_E: Self = f64::consts::LOG10_E;
	const LOG2_10: Self = f64::consts::LOG2_10;
	const LOG2_E: Self = f64::consts::LOG2_E;
	const ONE: Self = 1.0;
	const PI: Self = f64::consts::PI;
	const SQRT_2: Self = f64::consts::SQRT_2;
	const TAU: Self = f64::consts::TAU;
	const ZERO: Self = 0.0;

	// }

	// Comparison methods {
	#[inline]
	fn all_eq(self, rhs: Self) -> bool {
		self == rhs
	}

	#[inline]
	fn any_eq(self, rhs: Self) -> bool {
		self == rhs
	}

	#[inline]
	fn none_eq(self, rhs: Self) -> bool {
		self != rhs
	}

	#[inline]
	fn all_ne(self, rhs: Self) -> bool {
		self != rhs
	}

	#[inline]
	fn any_ne(self, rhs: Self) -> bool {
		self != rhs
	}

	#[inline]
	fn none_ne(self, rhs: Self) -> bool {
		self == rhs
	}

	#[inline]
	fn all_le(self, rhs: Self) -> bool {
		self <= rhs
	}

	#[inline]
	fn any_le(self, rhs: Self) -> bool {
		self <= rhs
	}

	#[inline]
	fn none_le(self, rhs: Self) -> bool {
		self > rhs
	}

	#[inline]
	fn all_lt(self, rhs: Self) -> bool {
		self < rhs
	}

	#[inline]
	fn any_lt(self, rhs: Self) -> bool {
		self < rhs
	}

	#[inline]
	fn none_lt(self, rhs: Self) -> bool {
		self >= rhs
	}

	#[inline]
	fn all_ge(self, rhs: Self) -> bool {
		self >= rhs
	}

	#[inline]
	fn any_ge(self, rhs: Self) -> bool {
		self >= rhs
	}

	#[inline]
	fn none_ge(self, rhs: Self) -> bool {
		self < rhs
	}

	#[inline]
	fn all_gt(self, rhs: Self) -> bool {
		self > rhs
	}

	#[inline]
	fn any_gt(self, rhs: Self) -> bool {
		self > rhs
	}

	#[inline]
	fn none_gt(self, rhs: Self) -> bool {
		self <= rhs
	}

	// }

	// methods {
	fn value(self) -> Self::Value {
		self
	}

	fn new(val: Self::Value) -> Self {
		val
	}

	fn first_as_f64(self) -> f64 {
		self
	}

	#[inline]
	fn abs(self) -> Self {
		self.abs()
	}

	#[inline]
	fn acos(self) -> Self {
		self.acos()
	}

	#[inline]
	fn asin(self) -> Self {
		self.asin()
	}

	#[inline]
	fn atan(self) -> Self {
		self.atan()
	}

	#[inline]
	fn atan2(self, x: Self) -> Self {
		self.atan2(x)
	}

	#[inline]
	fn ceil(self) -> Self {
		self.ceil()
	}

	#[inline]
	fn copysign(self, sign: Self) -> Self {
		self.copysign(sign)
	}

	#[inline]
	fn cos(self) -> Self {
		self.cos()
	}

	#[inline]
	fn floor(self) -> Self {
		self.floor()
	}

	#[inline]
	fn ln(self) -> Self {
		self.ln()
	}

	#[inline]
	fn log10(self) -> Self {
		self.log10()
	}

	#[inline]
	fn log2(self) -> Self {
		self.log2()
	}

	#[inline]
	fn max(self, rhs: Self) -> Self {
		self.max(rhs)
	}

	#[inline]
	fn min(self, rhs: Self) -> Self {
		self.min(rhs)
	}

	#[inline]
	fn mul_add(self, m: Self, a: Self) -> Self {
		self.mul_add(m, a)
	}

	#[inline]
	fn mul_sub(self, m: Self, a: Self) -> Self {
		self.mul_add(m, -a)
	}

	#[inline]
	fn pow(self, y: Self) -> Self {
		self.powf(y)
	}

	#[inline]
	fn round(self) -> Self {
		self.round()
	}

	#[inline]
	fn sin(self) -> Self {
		self.sin()
	}

	#[inline]
	fn sin_cos(self) -> (Self, Self) {
		self.sin_cos()
	}

	#[inline]
	fn splat(elem: f64) -> Self {
		elem
	}

	#[inline]
	fn sqrt(self) -> Self {
		self.sqrt()
	}

	#[inline]
	fn tan(self) -> Self {
		self.tan()
	}

	#[inline]
	fn to_degrees(self) -> Self {
		self.to_degrees()
	}

	#[inline]
	fn to_radians(self) -> Self {
		self.to_radians()
	}

	fn clamp(self, min: Self, max: Self) -> Self {
		self.clamp(min, max)
	}
	// }
}

impl SimdFloat for f32 {
	type Value = f32;

	// consts {
	const E: Self = f32::consts::E;
	const FRAC_1_PI: Self = f32::consts::FRAC_1_PI;
	const FRAC_1_SQRT_2: Self = f32::consts::FRAC_1_SQRT_2;
	const FRAC_2_PI: Self = f32::consts::FRAC_2_PI;
	const FRAC_2_SQRT_PI: Self = f32::consts::FRAC_2_SQRT_PI;
	const FRAC_PI_2: Self = f32::consts::FRAC_PI_2;
	const FRAC_PI_3: Self = f32::consts::FRAC_PI_3;
	const FRAC_PI_4: Self = f32::consts::FRAC_PI_4;
	const FRAC_PI_6: Self = f32::consts::FRAC_PI_6;
	const FRAC_PI_8: Self = f32::consts::FRAC_PI_8;
	const HALF: Self = 0.5;
	const LN_10: Self = f32::consts::LN_10;
	const LN_2: Self = f32::consts::LN_2;
	const LOG10_2: Self = f32::consts::LOG10_2;
	const LOG10_E: Self = f32::consts::LOG10_E;
	const LOG2_10: Self = f32::consts::LOG2_10;
	const LOG2_E: Self = f32::consts::LOG2_E;
	const ONE: Self = 1.0;
	const PI: Self = f32::consts::PI;
	const SQRT_2: Self = f32::consts::SQRT_2;
	const TAU: Self = f32::consts::TAU;
	const ZERO: Self = 0.0;

	// }

	// Comparison methods {
	#[inline]
	fn all_eq(self, rhs: Self) -> bool {
		self == rhs
	}

	#[inline]
	fn any_eq(self, rhs: Self) -> bool {
		self == rhs
	}

	#[inline]
	fn none_eq(self, rhs: Self) -> bool {
		self != rhs
	}

	#[inline]
	fn all_ne(self, rhs: Self) -> bool {
		self != rhs
	}

	#[inline]
	fn any_ne(self, rhs: Self) -> bool {
		self != rhs
	}

	#[inline]
	fn none_ne(self, rhs: Self) -> bool {
		self == rhs
	}

	#[inline]
	fn all_le(self, rhs: Self) -> bool {
		self <= rhs
	}

	#[inline]
	fn any_le(self, rhs: Self) -> bool {
		self <= rhs
	}

	#[inline]
	fn none_le(self, rhs: Self) -> bool {
		self > rhs
	}

	#[inline]
	fn all_lt(self, rhs: Self) -> bool {
		self < rhs
	}

	#[inline]
	fn any_lt(self, rhs: Self) -> bool {
		self < rhs
	}

	#[inline]
	fn none_lt(self, rhs: Self) -> bool {
		self >= rhs
	}

	#[inline]
	fn all_ge(self, rhs: Self) -> bool {
		self >= rhs
	}

	#[inline]
	fn any_ge(self, rhs: Self) -> bool {
		self >= rhs
	}

	#[inline]
	fn none_ge(self, rhs: Self) -> bool {
		self < rhs
	}

	#[inline]
	fn all_gt(self, rhs: Self) -> bool {
		self > rhs
	}

	#[inline]
	fn any_gt(self, rhs: Self) -> bool {
		self > rhs
	}

	#[inline]
	fn none_gt(self, rhs: Self) -> bool {
		self <= rhs
	}

	// }

	// methods {
	fn value(self) -> Self::Value {
		self
	}

	fn new(val: Self::Value) -> Self {
		val
	}

	fn first_as_f64(self) -> f64 {
		self as f64
	}

	#[inline]
	fn abs(self) -> Self {
		self.abs()
	}

	#[inline]
	fn acos(self) -> Self {
		self.acos()
	}

	#[inline]
	fn asin(self) -> Self {
		self.asin()
	}

	#[inline]
	fn atan(self) -> Self {
		self.atan()
	}

	#[inline]
	fn atan2(self, x: Self) -> Self {
		self.atan2(x)
	}

	#[inline]
	fn ceil(self) -> Self {
		self.ceil()
	}

	#[inline]
	fn copysign(self, sign: Self) -> Self {
		self.copysign(sign)
	}

	#[inline]
	fn cos(self) -> Self {
		self.cos()
	}

	#[inline]
	fn floor(self) -> Self {
		self.floor()
	}

	#[inline]
	fn ln(self) -> Self {
		self.ln()
	}

	#[inline]
	fn log10(self) -> Self {
		self.log10()
	}

	#[inline]
	fn log2(self) -> Self {
		self.log2()
	}

	#[inline]
	fn max(self, rhs: Self) -> Self {
		self.max(rhs)
	}

	#[inline]
	fn min(self, rhs: Self) -> Self {
		self.min(rhs)
	}

	#[inline]
	fn mul_add(self, m: Self, a: Self) -> Self {
		self.mul_add(m, a)
	}

	#[inline]
	fn mul_sub(self, m: Self, a: Self) -> Self {
		self.mul_add(m, -a)
	}

	#[inline]
	fn pow(self, y: Self) -> Self {
		self.powf(y)
	}

	#[inline]
	fn round(self) -> Self {
		self.round()
	}

	#[inline]
	fn sin(self) -> Self {
		self.sin()
	}

	#[inline]
	fn sin_cos(self) -> (Self, Self) {
		self.sin_cos()
	}

	#[inline]
	fn splat(elem: f64) -> Self {
		elem as f32
	}

	#[inline]
	fn sqrt(self) -> Self {
		self.sqrt()
	}

	#[inline]
	fn tan(self) -> Self {
		self.tan()
	}

	#[inline]
	fn to_degrees(self) -> Self {
		self.to_degrees()
	}

	#[inline]
	fn to_radians(self) -> Self {
		self.to_radians()
	}

	fn clamp(self, min: Self, max: Self) -> Self {
		self.clamp(min, max)
	}
	// }
}
