// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

#![allow(unused_unsafe)]

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
use core::arch::aarch64::*;
#[cfg(target_arch = "x86")]
use core::arch::x86::*;
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;
use core::ops::{
	Add,
	AddAssign,
	Div,
	DivAssign,
	Mul,
	MulAssign,
	Neg,
	Sub,
	SubAssign,
};

pub trait SimdValue {
	type Value;
	fn value(self) -> Self::Value;
}
#[derive(Debug, Copy, Clone)]
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub struct F64X2(__m128d);

#[cfg(not(any(
	target_arch = "x86",
	target_arch = "x86_64",
	all(target_arch = "aarch64", target_feature = "neon")
)))]
pub use scalar::*;

#[derive(Debug, Copy, Clone)]
#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
pub struct F64X2(float64x2_t);

macro_rules! generate {
	[
		$t:ty => $out:ty;
		from_f64: $from_f64:path,
		neg: $neg:path,
		add: $add:path,
		sub: $sub:path,
		mul: $mul:path,
		div: $div:path,
	] => {
		impl SimdValue for $t {
			type Value = $out;
			#[inline]
			fn value(self) -> Self::Value {
				unsafe {  core::mem::transmute(self.0) }
			}
		}

		impl From<f64> for $t {
			#[inline]
			fn from(f: f64) -> Self {
				unsafe { Self($from_f64(f)) }
			}
		}
		impl Neg for $t {
			type Output = Self;
			#[inline]
			fn neg(self) -> Self {
				unsafe { Self($neg(self.0)) }
			}
		}

		impl Add for $t {
			type Output = Self;
			#[inline]
			fn add(self, rhs: Self) -> Self {
				unsafe { Self($add(self.0, rhs.0)) }
			}
		}

		impl AddAssign for $t {
			fn add_assign(&mut self, rhs: Self) {
				*self = *self + rhs;
			}
		}

		impl Sub for $t {
			type Output = Self;
			#[inline]
			fn sub(self, rhs: Self) -> Self {
				unsafe { Self($sub(self.0, rhs.0)) }
			}
		}

		impl SubAssign for $t {
			fn sub_assign(&mut self, rhs: Self) {
				*self = *self - rhs;
			}
		}

		impl Mul for $t {
			type Output = Self;
			#[inline]
			fn mul(self, rhs: Self) -> Self {
				unsafe { Self($mul(self.0, rhs.0)) }
			}
		}

		impl MulAssign for $t {
			fn mul_assign(&mut self, rhs: Self) {
				*self = *self * rhs;
			}
		}

		impl Div for $t {
			type Output = Self;
			#[inline]
			fn div(self, rhs: Self) -> Self {
				unsafe { Self($div(self.0, rhs.0)) }
			}
		}

		impl DivAssign for $t {
			fn div_assign(&mut self, rhs: Self) {
				*self = *self / rhs;
			}
		}
	};
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[inline]
fn f64x2_neg(v: __m128d) -> __m128d {
	unsafe {
		let sign_mask = _mm_set1_pd(-0.0);
		_mm_xor_pd(v, sign_mask)
	}
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
generate! {
	F64X2 => [f64; 2];
	from_f64: _mm_set1_pd,
	neg: f64x2_neg,
	add: _mm_add_pd,
	sub: _mm_sub_pd,
	mul: _mm_mul_pd,
	div: _mm_div_pd,
}

#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
generate! {
	F64X2 => [f64; 2];
	from_f64: vdupq_n_f64,
	neg: vnegq_f64,
	add: vaddq_f64,
	sub: vsubq_f64,
	mul: vmulq_f64,
	div: vdivq_f64,
}

mod scalar {
	use core::ops::{
		Add,
		AddAssign,
		Div,
		DivAssign,
		Mul,
		MulAssign,
		Neg,
		Sub,
		SubAssign,
	};

	#[derive(Debug, Copy, Clone)]
	pub struct F64X2([f64; 2]);
	#[derive(Debug, Copy, Clone)]
	pub struct F32X2([f32; 2]);

	macro_rules! generate {
		[$($t:ty),+ $(,)?] => {$(
			impl From<f64> for $t {
				#[inline]
				fn from(f: f64) -> Self {
					Self(core::array::from_fn(|_| f as _))
				}
				}

				impl Neg for $t {
					type Output = Self;
					#[inline]
					fn neg(self) -> Self {
						Self(self.0.map(|x| -x))
					}
				}

				impl Add for $t {
					type Output = Self;
					#[inline]
					fn add(mut self, rhs: Self) -> Self {
						for (a, b) in self.0.iter_mut().zip(rhs.0) {
							*a += b;
						}
						self
					}
					}

					impl AddAssign for $t {
						fn add_assign(&mut self, rhs: Self) {
							*self = *self + rhs;
						}
					}

					impl Sub for $t {
						type Output = Self;
						#[inline]
						fn sub(mut self, rhs: Self) -> Self {
							for (a, b) in self.0.iter_mut().zip(rhs.0) {
								*a -= b;
							}

							self
						}
					}

					impl SubAssign for $t {
						fn sub_assign(&mut self, rhs: Self) {
							*self = *self - rhs;
						}
					}

					impl Mul for $t {
						type Output = Self;
						#[inline]
						fn mul(mut self, rhs: Self) -> Self {
							for (a, b) in self.0.iter_mut().zip(rhs.0) {
								*a *= b;
							}

							self
						}
					}

	impl MulAssign for $t {
		fn mul_assign(&mut self, rhs: Self) {
			*self = *self * rhs;
		}
	}

					impl Div for $t {
						type Output = Self;
						#[inline]
						fn div(mut self, rhs: Self) -> Self {
							for (a, b) in self.0.iter_mut().zip(rhs.0) {
								*a /= b;
							}

							self
						}
					}

	impl DivAssign for $t {
		fn div_assign(&mut self, rhs: Self) {
			*self = *self / rhs;
		}
	}
		)+};
	}

	generate![F64X2, F32X2];
}
