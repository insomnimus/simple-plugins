// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

use std::sync::Arc;

use components::{
	Component,
	ComponentMeta,
	SimdFloat,
	Simper,
	SimperCoefficients,
};
use nih_plug::prelude::*;

const HPF_OFF_FQ: f32 = 20.0;
const LPF_OFF_FQ: f32 = 20000.0;

#[derive(Debug, Params)]
pub struct FilterParam {
	#[id = "g"]
	pub gain: FloatParam,
	#[id = "f"]
	pub fq: FloatParam,
	#[id = "q"]
	pub q: FloatParam,
}

impl FilterParam {
	pub fn new(name: &str, default_fq: f32, min_fq: f32, max_fq: f32) -> Self {
		Self {
			fq: fq_param(format!("{name} Freq"), default_fq, min_fq, max_fq),
			q: q_param(format!("{name} Q")),
			gain: FloatParam::new(
				format!("{name} Gain"),
				0.0,
				FloatRange::Linear {
					min: -20.0,
					max: 20.0,
				},
			)
			.with_unit("dB")
			.with_step_size(0.05),
		}
	}
}

#[derive(Debug, Params)]
pub struct EqParams {
	#[id = "eq_by"]
	pub bypass: BoolParam,

	#[id = "hp_fq"]
	hp: FloatParam,
	#[id = "lp_fq"]
	pub lp: FloatParam,

	#[nested(id_prefix = "low", group = "Low")]
	pub low: FilterParam,

	#[nested(id_prefix = "lm", group = "Low Mid")]
	pub low_mid: FilterParam,

	#[nested(id_prefix = "m", group = "Mid")]
	pub mid: FilterParam,

	#[nested(id_prefix = "hi", group = "High")]
	pub high: FilterParam,

	#[nested(id_prefix = "top", group = "Top")]
	pub top: FilterParam,
}

impl Default for EqParams {
	fn default() -> Self {
		Self {
			bypass: BoolParam::new("Eq Bypass", false)
				.with_value_to_string(formatters::v2s_bool_bypass())
				.with_string_to_value(formatters::s2v_bool_bypass()),
			hp: fq_param("HPF Frequency", HPF_OFF_FQ, 19.9, 15000.0).with_value_to_string(
				Arc::new(|val| {
					if val <= HPF_OFF_FQ {
						"Off".to_owned()
					} else {
						format_fq(val)
					}
				}),
			),
			lp: fq_param("LPF Frequency", LPF_OFF_FQ, 200.0, 20000.1).with_value_to_string(
				Arc::new(|val| {
					if val >= LPF_OFF_FQ {
						"Off".to_owned()
					} else {
						format_fq(val)
					}
				}),
			),

			low: FilterParam::new("Low", 60.0, 20.0, 1500.0),
			low_mid: FilterParam::new("Low Mid", 350.0, 60.0, 3500.0),
			mid: FilterParam::new("Mid", 1500.0, 250.0, 5000.0),
			high: FilterParam::new("High", 4000.0, 1500.0, 8000.0),
			top: FilterParam::new("Top", 8000.0, 2500.0, 20000.0),
		}
	}
}
fn fq_param(name: impl Into<String>, default: f32, min: f32, max: f32) -> FloatParam {
	FloatParam::new(
		name,
		default,
		FloatRange::Skewed {
			min,
			max,
			factor: FloatRange::skew_factor(-2.0),
		},
	)
	.with_string_to_value(formatters::s2v_f32_hz_then_khz())
	.with_value_to_string(Arc::new(format_fq))
	// .with_value_to_string(formatters::v2s_f32_hz_then_khz(2))
}

fn q_param(name: impl Into<String>) -> FloatParam {
	FloatParam::new(
		name,
		Simper::BUTTERWORTH_Q,
		FloatRange::Skewed {
			min: 0.1,
			max: 4.8,
			factor: FloatRange::skew_factor(-1.0),
		},
	)
	.with_value_to_string(formatters::v2s_f32_rounded(2))
}

fn format_fq(fq: f32) -> String {
	if fq < 1000.0 {
		format!("{fq:.2} hz")
	} else {
		format!("{:.2} khz", fq / 1000.0)
	}
}

pub struct Eq<T> {
	bypassed: bool,
	sr: T,
	hp_active: bool,
	lp_active: bool,

	hp: Simper<T>,
	lp: Simper<T>,
	low: Simper<T>,
	low_mid: Simper<T>,
	mid: Simper<T>,
	high: Simper<T>,
	top: Simper<T>,
}

impl<T: SimdFloat> ComponentMeta for Eq<T> {
	fn latency(&self) -> usize {
		if self.bypassed {
			return 0;
		}

		let mut latency = self.low.latency()
			+ self.low_mid.latency()
			+ self.mid.latency()
			+ self.high.latency()
			+ self.top.latency();

		if self.hp_active {
			latency += self.hp.latency();
		}
		if self.lp_active {
			latency += self.lp.latency();
		}

		latency
	}

	fn reset(&mut self) {
		self.hp.reset();
		self.lp.reset();
		self.low.reset();
		self.low_mid.reset();
		self.mid.reset();
		self.high.reset();
		self.top.reset();
	}
}

impl<T: SimdFloat> Component<T> for Eq<T> {
	fn process(&mut self, mut sample: T) -> T {
		if self.bypassed {
			return sample;
		}

		let bands = [
			&mut self.low,
			&mut self.low_mid,
			&mut self.mid,
			&mut self.high,
			&mut self.top,
		];

		for band in bands {
			sample = band.process(sample);
		}

		if self.hp_active {
			sample = self.hp.process(sample);
		}
		if self.lp_active {
			sample = self.lp.process(sample);
		}

		sample
	}
}

impl<T: SimdFloat> Eq<T> {
	pub fn new(sr: T, params: &EqParams) -> Self {
		let bell = |param: &FilterParam| {
			Simper::bell(
				sr,
				T::splat(param.fq.value() as _),
				T::splat(param.q.value() as _),
				T::splat(param.gain.value() as _),
			)
		};

		Self {
			bypassed: params.bypass.value(),
			sr,
			hp_active: params.hp.value() > HPF_OFF_FQ,
			lp_active: params.lp.value() < LPF_OFF_FQ,
			hp: Simper::high_pass(sr, T::splat(params.hp.value() as _), Simper::BUTTERWORTH_Q),
			lp: Simper::low_pass(sr, T::splat(params.hp.value() as _), Simper::BUTTERWORTH_Q),

			low: Simper::low_shelf(
				sr,
				T::splat(params.low.fq.value() as _),
				T::splat(params.low.q.value() as _),
				T::splat(params.low.gain.value() as _),
			),

			low_mid: bell(&params.low_mid),
			mid: bell(&params.mid),
			high: bell(&params.high),

			top: Simper::high_shelf(
				sr,
				T::splat(params.top.fq.value() as _),
				T::splat(params.top.q.value() as _),
				T::splat(params.top.gain.value() as _),
			),
		}
	}

	pub fn update_parameters(&mut self, params: &EqParams) {
		let bypassed = params.bypass.value();
		if bypassed {
			if !self.bypassed {
				self.reset();
			}
			self.bypassed = true;
			return;
		}

		self.bypassed = bypassed;

		let filters = [
			(&mut self.low, &params.low),
			(&mut self.low_mid, &params.low_mid),
			(&mut self.mid, &params.mid),
			(&mut self.high, &params.high),
			(&mut self.top, &params.top),
		];

		for (filter, param) in filters {
			filter.update_parameters(
				self.sr,
				T::splat(param.fq.value() as _),
				T::splat(param.q.value() as _),
				T::splat(param.gain.value() as _),
			);
		}

		self.hp.set_parameters(SimperCoefficients::high_pass(
			self.sr,
			T::splat(params.hp.value() as _),
			Simper::BUTTERWORTH_Q,
		));
		self.lp.set_parameters(SimperCoefficients::low_pass(
			self.sr,
			T::splat(params.lp.value() as _),
			Simper::BUTTERWORTH_Q,
		));

		let hp_off = params.hp.value() <= HPF_OFF_FQ;
		let lp_off = params.lp.value() >= LPF_OFF_FQ;

		if hp_off && self.hp_active {
			self.hp.reset();
		}
		if lp_off && self.lp_active {
			self.lp.reset();
		}

		self.hp_active = !hp_off;
		self.lp_active = !lp_off;
	}
}
