// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

//! State variable filter (SVF), designed by Andrew Simper of Cytomic.

use crate::{
	replace_float_literals,
	Component,
	ComponentMeta,
	SimdFloat,
};

/**State variable filter (SVF), designed by Andrew Simper of Cytomic.

The frequency response of this filter is the same as of BZT filters.<br>
This is a second-order filter. It has a cutoff slope of 12 dB/octave.<br>
Q = 0.707 means no resonant peaking.<br>
Translated from <https://gist.github.com/hollance/2891d89c57adc71d9560bcf0e1e55c4b>
**/
#[derive(Debug, Clone)]
pub struct Simper<T> {
	x: SimperCoefficients<T>,
	// State variables
	ic1eq: T,
	ic2eq: T,
}

/// [`Simper`] filter type.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SimperType {
	LowPass,
	HighPass,
	BandPass,
	Notch,
	AllPass,
	Peaking,
	Bell,
	LowShelf,
	HighShelf,
}

/// Coefficients for a [`Simper`] filter.
#[derive(Debug, Clone)]
pub struct SimperCoefficients<T> {
	kind: SimperType,
	// Mix coefficients
	m0: T,
	m1: T,
	m2: T,
	// Filter coefficients
	a1: T,
	a2: T,
	a3: T,
	g: T,
	k: T,
}

impl<T: SimdFloat> SimperCoefficients<T> {
	// For internal use. It doesn't really make sense otherwise.
	#[replace_float_literals(T)]
	fn new(kind: SimperType) -> Self {
		Self {
			kind,
			// Mix coefficients
			m0: 0.0,
			m1: 0.0,
			m2: 0.0,
			// Filter coefficients
			a1: 0.0,
			a2: 0.0,
			a3: 0.0,
			g: 0.0,
			k: 0.0,
		}
	}

	#[replace_float_literals(T)]
	fn set_coefficients(&mut self, sample_rate: T, fq: T, q: T) {
		self.g = T::tan(T::PI * fq / sample_rate);
		self.k = 1.0 / q;

		self.a1 = 1.0 / (1.0 + self.g * (self.g + self.k));
		self.a2 = self.g * self.a1;
		self.a3 = self.g * self.a2;
	}

	#[replace_float_literals(T)]
	pub fn low_pass(sample_rate: T, fq: T, q: T) -> Self {
		let mut x = Self::new(SimperType::LowPass);
		x.set_coefficients(sample_rate, fq, q);

		x.m0 = 0.0;
		x.m1 = 0.0;
		x.m2 = 1.0;

		x
	}

	#[replace_float_literals(T)]
	pub fn high_pass(sample_rate: T, fq: T, q: T) -> Self {
		let mut x = Self::new(SimperType::HighPass);
		x.set_coefficients(sample_rate, fq, q);

		x.m0 = 1.0;
		x.m1 = -x.k;
		x.m2 = -1.0;

		x
	}

	#[replace_float_literals(T)]
	pub fn band_pass(sample_rate: T, fq: T, q: T) -> Self {
		let mut x = Self::new(SimperType::BandPass);
		x.set_coefficients(sample_rate, fq, q);

		x.m0 = 0.0;
		x.m1 = x.k; // paper says 1, but that is not same as RBJ bandpass
		x.m2 = 0.0;

		x
	}

	#[replace_float_literals(T)]
	pub fn notch(sample_rate: T, fq: T, q: T) -> Self {
		let mut x = Self::new(SimperType::Notch);
		x.set_coefficients(sample_rate, fq, q);

		x.m0 = 1.0;
		x.m1 = -x.k;
		x.m2 = 0.0;

		x
	}

	#[replace_float_literals(T)]
	pub fn all_pass(sample_rate: T, fq: T, q: T) -> Self {
		let mut x = Self::new(SimperType::AllPass);
		x.set_coefficients(sample_rate, fq, q);

		x.m0 = 1.0;
		x.m1 = -2.0 * x.k;
		x.m2 = 0.0;

		x
	}

	// Note: This is not the same as the RBJ peaking filter, since no db_gain.
	#[replace_float_literals(T)]
	pub fn peaking(sample_rate: T, fq: T, q: T) -> Self {
		let mut x = Self::new(SimperType::Peaking);
		x.set_coefficients(sample_rate, fq, q);

		x.m0 = 1.0;
		x.m1 = -x.k;
		x.m2 = -2.0;

		x
	}

	// Note: This is the same as the RBJ peaking EQ.
	#[replace_float_literals(T)]
	pub fn bell(sample_rate: T, fq: T, q: T, db_gain: T) -> Self {
		let mut x = Self::new(SimperType::Bell);
		let a = T::pow(10.0, db_gain / 40.0);

		x.g = T::tan(T::PI * fq / sample_rate);
		x.k = 1.0 / (q * a);

		x.a1 = 1.0 / (1.0 + x.g * (x.g + x.k));
		x.a2 = x.g * x.a1;
		x.a3 = x.g * x.a2;

		x.m0 = 1.0;
		x.m1 = x.k * (a * a - 1.0);
		x.m2 = 0.0;

		x
	}

	#[replace_float_literals(T)]
	pub fn low_shelf(sample_rate: T, fq: T, q: T, db_gain: T) -> Self {
		let mut x = Self::new(SimperType::LowShelf);
		let a = T::pow(10.0, db_gain / 40.0);

		x.g = T::tan(T::PI * fq / sample_rate) / T::sqrt(a);
		x.k = 1.0 / q;

		x.a1 = 1.0 / (1.0 + x.g * (x.g + x.k));
		x.a2 = x.g * x.a1;
		x.a3 = x.g * x.a2;

		x.m0 = 1.0;
		x.m1 = x.k * (a - 1.0);
		x.m2 = a * a - 1.0;

		x
	}

	#[replace_float_literals(T)]
	pub fn high_shelf(sample_rate: T, fq: T, q: T, db_gain: T) -> Self {
		let mut x = Self::new(SimperType::HighShelf);
		let a = T::pow(10.0, db_gain / 40.0);

		x.g = T::tan(T::PI * fq / sample_rate) * T::sqrt(a);
		x.k = 1.0 / q;

		x.a1 = 1.0 / (1.0 + x.g * (x.g + x.k));
		x.a2 = x.g * x.a1;
		x.a3 = x.g * x.a2;

		x.m0 = a * a;
		x.m1 = x.k * (1.0 - a) * a;
		x.m2 = 1.0 - a * a;

		x
	}
}

macro_rules! constr_no_gain {
	[$($name:ident),+ $(,)?] => {
		$(
			pub fn $name(sample_rate: T, fq: T, q: T) -> Self {
				Self::new(SimperCoefficients::$name(sample_rate, fq, q))
			}
		)+
	};
}

impl<T: SimdFloat> Simper<T> {
	pub const BUTTERWORTH_Q: T = T::FRAC_1_SQRT_2;

	constr_no_gain![low_pass, high_pass, band_pass, notch, all_pass, peaking];

	pub fn bell(sample_rate: T, fq: T, q: T, db_gain: T) -> Self {
		Self {
			x: SimperCoefficients::bell(sample_rate, fq, q, db_gain),
			ic1eq: T::ZERO,
			ic2eq: T::ZERO,
		}
	}

	pub fn low_shelf(sample_rate: T, fq: T, q: T, db_gain: T) -> Self {
		Self {
			x: SimperCoefficients::low_shelf(sample_rate, fq, q, db_gain),
			ic1eq: T::ZERO,
			ic2eq: T::ZERO,
		}
	}

	pub fn high_shelf(sample_rate: T, fq: T, q: T, db_gain: T) -> Self {
		Self {
			x: SimperCoefficients::high_shelf(sample_rate, fq, q, db_gain),
			ic1eq: T::ZERO,
			ic2eq: T::ZERO,
		}
	}

	pub fn new(x: SimperCoefficients<T>) -> Self {
		Self {
			x,
			ic1eq: T::ZERO,
			ic2eq: T::ZERO,
		}
	}

	pub fn filter_type(&self) -> SimperType {
		self.x.kind
	}

	/// Update coefficients without resetting state.
	///
	/// `db_gain` is ignored if [`self.filter_type()`](Self::filter_type) is not `SimperType::Bell`, `SimperType::LowShelf` or `SimperType::HighShelf`.
	///
	/// If you know the filter type, or want to replace it, use [`Simper::set_parameters()`] instead, as it will bypass matching on [`SimperType`].
	pub fn update_parameters(&mut self, sample_rate: T, fq: T, q: T, db_gain: T) {
		use SimperType::*;

		let sr = sample_rate;
		self.x = match self.filter_type() {
			LowPass => SimperCoefficients::low_pass(sr, fq, q),
			HighPass => SimperCoefficients::high_pass(sr, fq, q),
			BandPass => SimperCoefficients::band_pass(sr, fq, q),
			Notch => SimperCoefficients::notch(sr, fq, q),
			AllPass => SimperCoefficients::all_pass(sr, fq, q),
			Peaking => SimperCoefficients::peaking(sr, fq, q),

			Bell => SimperCoefficients::bell(sr, fq, q, db_gain),
			LowShelf => SimperCoefficients::low_shelf(sr, fq, q, db_gain),
			HighShelf => SimperCoefficients::high_shelf(sr, fq, q, db_gain),
		};
	}

	/// Replace parameters without clearing state.
	pub fn set_parameters(&mut self, x: SimperCoefficients<T>) {
		self.x = x;
	}
}

impl<T: SimdFloat> ComponentMeta for Simper<T> {
	fn latency(&self) -> usize {
		0
	}

	fn reset(&mut self) {
		self.ic1eq = T::ZERO;
		self.ic2eq = T::ZERO;
	}
}

impl<T: SimdFloat> Component<T> for Simper<T> {
	#[inline]
	#[replace_float_literals(T)]
	fn process(&mut self, v0: T) -> T {
		let x = &mut self.x;
		let v3 = v0 - self.ic2eq;
		let v1 = x.a1 * self.ic1eq + x.a2 * v3;
		let v2 = self.ic2eq + x.a2 * self.ic1eq + x.a3 * v3;
		self.ic1eq = 2.0 * v1 - self.ic1eq;
		self.ic2eq = 2.0 * v2 - self.ic2eq;

		x.m0 * v0 + x.m1 * v1 + x.m2 * v2
	}
}
