//! State variable filter (SVF), designed by Andrew Simper of Cytomic.

use crate::{
	Component,
	ComponentMeta,
};

const M_PI: f64 = core::f64::consts::PI;
pub const BUTTERWORTH_Q: f64 = 0.707;

/**State variable filter (SVF), designed by Andrew Simper of Cytomic.

The frequency response of this filter is the same as of BZT filters.<br>
This is a second-order filter. It has a cutoff slope of 12 dB/octave.<br>
Q = 0.707 means no resonant peaking.<br>
Translated from https://gist.github.com/hollance/2891d89c57adc71d9560bcf0e1e55c4b
**/
#[derive(Debug, Clone)]
pub struct Simper {
	x: Coefficients,
	// State variables
	ic1eq: f64,
	ic2eq: f64,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FilterType {
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

#[derive(Debug, Clone)]
pub struct Coefficients {
	kind: FilterType,
	// Mix coefficients
	m0: f64,
	m1: f64,
	m2: f64,
	// Filter coefficients
	a1: f64,
	a2: f64,
	a3: f64,
	g: f64,
	k: f64,
}

impl Coefficients {
	// For internal use. It doesn't really make sense otherwise.
	fn new(kind: FilterType) -> Self {
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

	fn set_coefficients(&mut self, sample_rate: f64, fq: f64, q: f64) {
		self.g = f64::tan(M_PI * fq / sample_rate);
		self.k = 1.0 / q;

		self.a1 = 1.0 / (1.0 + self.g * (self.g + self.k));
		self.a2 = self.g * self.a1;
		self.a3 = self.g * self.a2;
	}

	pub fn low_pass(sample_rate: f64, fq: f64, q: f64) -> Self {
		let mut x = Self::new(FilterType::LowPass);
		x.set_coefficients(sample_rate, fq, q);

		x.m0 = 0.0;
		x.m1 = 0.0;
		x.m2 = 1.0;

		x
	}

	pub fn high_pass(sample_rate: f64, fq: f64, q: f64) -> Self {
		let mut x = Self::new(FilterType::HighPass);
		x.set_coefficients(sample_rate, fq, q);

		x.m0 = 1.0;
		x.m1 = -x.k;
		x.m2 = -1.0;

		x
	}

	pub fn band_pass(sample_rate: f64, fq: f64, q: f64) -> Self {
		let mut x = Self::new(FilterType::BandPass);
		x.set_coefficients(sample_rate, fq, q);

		x.m0 = 0.0;
		x.m1 = x.k; // paper says 1, but that is not same as RBJ bandpass
		x.m2 = 0.0;

		x
	}

	pub fn notch(sample_rate: f64, fq: f64, q: f64) -> Self {
		let mut x = Self::new(FilterType::Notch);
		x.set_coefficients(sample_rate, fq, q);

		x.m0 = 1.0;
		x.m1 = -x.k;
		x.m2 = 0.0;

		x
	}

	pub fn all_pass(sample_rate: f64, fq: f64, q: f64) -> Self {
		let mut x = Self::new(FilterType::AllPass);
		x.set_coefficients(sample_rate, fq, q);

		x.m0 = 1.0;
		x.m1 = -2.0 * x.k;
		x.m2 = 0.0;

		x
	}

	// Note: This is not the same as the RBJ peaking filter, since no db_gain.
	pub fn peaking(sample_rate: f64, fq: f64, q: f64) -> Self {
		let mut x = Self::new(FilterType::Peaking);
		x.set_coefficients(sample_rate, fq, q);

		x.m0 = 1.0;
		x.m1 = -x.k;
		x.m2 = -2.0;

		x
	}

	// Note: This is the same as the RBJ peaking EQ.
	pub fn bell(sample_rate: f64, fq: f64, q: f64, db_gain: f64) -> Self {
		let mut x = Self::new(FilterType::Bell);
		let a = f64::powf(10.0, db_gain / 40.0);

		x.g = f64::tan(M_PI * fq / sample_rate);
		x.k = 1.0 / (q * a);

		x.a1 = 1.0 / (1.0 + x.g * (x.g + x.k));
		x.a2 = x.g * x.a1;
		x.a3 = x.g * x.a2;

		x.m0 = 1.0;
		x.m1 = x.k * (a * a - 1.0);
		x.m2 = 0.0;

		x
	}

	pub fn low_shelf(sample_rate: f64, fq: f64, q: f64, db_gain: f64) -> Self {
		let mut x = Self::new(FilterType::LowShelf);
		let a = f64::powf(10.0, db_gain / 40.0);

		x.g = f64::tan(M_PI * fq / sample_rate) / f64::sqrt(a);
		x.k = 1.0 / q;

		x.a1 = 1.0 / (1.0 + x.g * (x.g + x.k));
		x.a2 = x.g * x.a1;
		x.a3 = x.g * x.a2;

		x.m0 = 1.0;
		x.m1 = x.k * (a - 1.0);
		x.m2 = a * a - 1.0;

		x
	}

	pub fn high_shelf(sample_rate: f64, fq: f64, q: f64, db_gain: f64) -> Self {
		let mut x = Self::new(FilterType::HighShelf);
		let a = f64::powf(10.0, db_gain / 40.0);

		x.g = f64::tan(M_PI * fq / sample_rate) * f64::sqrt(a);
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
			pub fn $name(sample_rate: f64, fq: f64, q: f64) -> Self {
				Self::new(Coefficients::$name(sample_rate, fq, q))
			}
		)+
	};
}

impl Simper {
	constr_no_gain![low_pass, high_pass, band_pass, notch, all_pass, peaking];

	pub fn bell(sample_rate: f64, fq: f64, q: f64, db_gain: f64) -> Self {
		Self {
			x: Coefficients::bell(sample_rate, fq, q, db_gain),
			ic1eq: 0.0,
			ic2eq: 0.0,
		}
	}

	pub fn low_shelf(sample_rate: f64, fq: f64, q: f64, db_gain: f64) -> Self {
		Self {
			x: Coefficients::low_shelf(sample_rate, fq, q, db_gain),
			ic1eq: 0.0,
			ic2eq: 0.0,
		}
	}

	pub fn high_shelf(sample_rate: f64, fq: f64, q: f64, db_gain: f64) -> Self {
		Self {
			x: Coefficients::high_shelf(sample_rate, fq, q, db_gain),
			ic1eq: 0.0,
			ic2eq: 0.0,
		}
	}

	pub fn new(x: Coefficients) -> Self {
		Self {
			x,
			ic1eq: 0.0,
			ic2eq: 0.0,
		}
	}

	pub fn filter_type(&self) -> FilterType {
		self.x.kind
	}

	/// Update coefficients without resetting state.
	///
	/// `db_gain` is ignored if [`self.filter_type()`](Self::filter_type) is not `FilterType::Bell`, `FilterType::LowShelf` or `FilterType::HighShelf`.
	///
	/// If you know the filter type, or want to replace it, use [`Simper::set_parameters()`] instead, as it will bypass matching on [`FilterType`].
	pub fn update_parameters(&mut self, sample_rate: f64, fq: f64, q: f64, db_gain: f64) {
		use FilterType::*;

		let sr = sample_rate;
		self.x = match self.filter_type() {
			LowPass => Coefficients::low_pass(sr, fq, q),
			HighPass => Coefficients::high_pass(sr, fq, q),
			BandPass => Coefficients::band_pass(sr, fq, q),
			Notch => Coefficients::notch(sr, fq, q),
			AllPass => Coefficients::all_pass(sr, fq, q),
			Peaking => Coefficients::peaking(sr, fq, q),

			Bell => Coefficients::bell(sr, fq, q, db_gain),
			LowShelf => Coefficients::low_shelf(sr, fq, q, db_gain),
			HighShelf => Coefficients::high_shelf(sr, fq, q, db_gain),
		};
	}

	/// Replace parameters without clearing state.
	pub fn set_parameters(&mut self, x: Coefficients) {
		self.x = x;
	}
}

impl ComponentMeta for Simper {
	fn reset(&mut self) {
		self.ic1eq = 0.0;
		self.ic2eq = 0.0;
	}
}

impl Component for Simper {
	/// Process a sample.
	#[inline]
	fn process(&mut self, v0: f64) -> f64 {
		let x = &mut self.x;
		let v3 = v0 - self.ic2eq;
		let v1 = x.a1 * self.ic1eq + x.a2 * v3;
		let v2 = self.ic2eq + x.a2 * self.ic1eq + x.a3 * v3;
		self.ic1eq = 2.0 * v1 - self.ic1eq;
		self.ic2eq = 2.0 * v2 - self.ic2eq;

		x.m0 * v0 + x.m1 * v1 + x.m2 * v2
	}
}
