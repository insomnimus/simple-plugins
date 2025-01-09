// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

use std::sync::Arc;

use components::{
	f64x2,
	ComponentMeta,
	Simper,
	SimperCoefficients,
};
use nih_plug::{
	prelude::*,
	util::db_to_gain,
};

nih_export_clap!(FilterPlugin);

const HPF_OFF_FQ: f32 = 20.0;
const LPF_OFF_FQ: f32 = 20000.0;

fn fq_param(name: &str, default: f32) -> FloatParam {
	FloatParam::new(
		name,
		default,
		FloatRange::Skewed {
			min: 19.9,
			max: 20000.1,
			factor: FloatRange::skew_factor(-2.0),
		},
	)
	.with_string_to_value(formatters::s2v_f32_hz_then_khz())
	// .with_value_to_string(formatters::v2s_f32_hz_then_khz(2))
}

fn q_param(name: &str) -> FloatParam {
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

#[derive(Debug, Params)]
struct FilterParam {
	#[id = "fq"]
	fq: FloatParam,
	#[id = "q"]
	q: FloatParam,
}

#[derive(Debug, Params)]
struct FilterParams {
	#[nested(id_prefix = "hp", group = "High-Pass Filter")]
	hp: FilterParam,
	#[nested(id_prefix = "lp", group = "Low-Pass Filter")]
	lp: FilterParam,
	#[id = "g_gain"]
	global_gain: FloatParam,
}

impl Default for FilterParams {
	fn default() -> Self {
		Self {
			global_gain: FloatParam::new(
				"Global gain",
				0.0,
				FloatRange::Linear {
					min: -50.0,
					max: 50.0,
				},
			)
			.with_unit("dB")
			.with_step_size(0.1),
			hp: FilterParam {
				q: q_param("HPF Q"),
				fq: fq_param("HPF Frequency", HPF_OFF_FQ).with_value_to_string(Arc::new(|fq| {
					if fq <= HPF_OFF_FQ {
						"Off".to_string()
					} else {
						format_fq(fq)
					}
				})),
			},

			lp: FilterParam {
				q: q_param("LPF Q"),
				fq: fq_param("LPF Frequency", LPF_OFF_FQ).with_value_to_string(Arc::new(|fq| {
					if fq >= LPF_OFF_FQ {
						"Off".to_string()
					} else {
						format_fq(fq)
					}
				})),
			},
		}
	}
}

struct FilterPlugin {
	params: Arc<FilterParams>,

	sr: f64,
	sr_f64x2: f64x2,

	hp_mono: Simper<f64>,
	lp_mono: Simper<f64>,
	hp_stereo: Simper<f64x2>,
	lp_stereo: Simper<f64x2>,
}

impl Default for FilterPlugin {
	fn default() -> Self {
		Self {
			params: Arc::new(FilterParams::default()),
			sr: 44100.0,
			sr_f64x2: f64x2::splat(44100.0),

			hp_mono: Simper::high_pass(44100.0, 20.0, Simper::BUTTERWORTH_Q),
			lp_mono: Simper::low_pass(44100.0, 20e3, Simper::BUTTERWORTH_Q),
			hp_stereo: Simper::high_pass(
				f64x2::splat(44100.0),
				f64x2::splat(20.0),
				Simper::BUTTERWORTH_Q,
			),
			lp_stereo: Simper::low_pass(
				f64x2::splat(44100.0),
				f64x2::splat(20e3),
				Simper::BUTTERWORTH_Q,
			),
		}
	}
}

impl FilterPlugin {
	fn reset_lp(&mut self) {
		self.lp_mono.reset();
		self.lp_stereo.reset();
	}

	fn reset_hp(&mut self) {
		self.hp_mono.reset();
		self.hp_stereo.reset();
	}

	fn update_hp(&mut self, fq: f32, q: f32) {
		self.hp_mono
			.set_parameters(SimperCoefficients::high_pass(self.sr, fq as _, q as _));
		self.hp_stereo.set_parameters(SimperCoefficients::high_pass(
			self.sr_f64x2,
			f64x2::splat(fq as _),
			f64x2::splat(q as _),
		));
	}

	fn update_lp(&mut self, fq: f32, q: f32) {
		self.lp_mono
			.set_parameters(SimperCoefficients::low_pass(self.sr, fq as _, q as _));
		self.lp_stereo.set_parameters(SimperCoefficients::low_pass(
			self.sr_f64x2,
			f64x2::splat(fq as _),
			f64x2::splat(q as _),
		));
	}
}

impl ClapPlugin for FilterPlugin {
	const CLAP_DESCRIPTION: Option<&'static str> = Some("Simple high and low-pass filters");
	const CLAP_FEATURES: &'static [ClapFeature] =
		&[ClapFeature::Filter, ClapFeature::Mono, ClapFeature::Stereo];
	const CLAP_ID: &'static str = "insomnia.simple-filter";
	const CLAP_MANUAL_URL: Option<&'static str> = None;
	const CLAP_SUPPORT_URL: Option<&'static str> = Some(env!("CARGO_PKG_HOMEPAGE"));
}

impl Plugin for FilterPlugin {
	type BackgroundTask = ();
	type SysExMessage = ();

	const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
		AudioIOLayout {
			main_input_channels: Some(new_nonzero_u32(2)),
			main_output_channels: Some(new_nonzero_u32(2)),
			aux_input_ports: &[],
			aux_output_ports: &[],
			names: PortNames {
				layout: Some("Stereo"),
				main_input: Some("Input"),
				main_output: Some("Output"),
				aux_inputs: &[],
				aux_outputs: &[],
			},
		},
		AudioIOLayout {
			main_input_channels: Some(new_nonzero_u32(1)),
			main_output_channels: Some(new_nonzero_u32(1)),
			aux_input_ports: &[],
			aux_output_ports: &[],
			names: PortNames {
				layout: Some("Mono"),
				main_input: Some("Input"),
				main_output: Some("Output"),
				aux_inputs: &[],
				aux_outputs: &[],
			},
		},
	];
	const EMAIL: &'static str = "";
	const NAME: &'static str = "Simple Filter";
	const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
	const VENDOR: &'static str = "Insomnia";
	const VERSION: &'static str = env!("CARGO_PKG_VERSION");

	fn params(&self) -> Arc<dyn Params> {
		self.params.clone()
	}

	fn reset(&mut self) {
		self.reset_hp();
		self.reset_lp();
	}

	fn initialize(
		&mut self,
		_layout: &AudioIOLayout,
		buffer_config: &BufferConfig,
		_context: &mut impl InitContext<Self>,
	) -> bool {
		// self.is_mono = layout.main_input_channels == Some(new_nonzero_u32(1));
		self.sr = buffer_config.sample_rate as _;
		self.sr_f64x2 = f64x2::splat(buffer_config.sample_rate as _);

		self.update_lp(20e3, Simper::<f64>::BUTTERWORTH_Q as _);
		self.update_hp(20.0, Simper::<f64>::BUTTERWORTH_Q as _);

		true
	}

	fn process(
		&mut self,
		buffer: &mut Buffer,
		_aux: &mut AuxiliaryBuffers,
		context: &mut impl ProcessContext<Self>,
	) -> ProcessStatus {
		let hp = self.params.hp.fq.value();
		let lp = self.params.lp.fq.value();
		let hp_q = self.params.hp.q.value();
		let lp_q = self.params.lp.q.value();
		let gain = db_to_gain(self.params.global_gain.value());

		match (hp <= HPF_OFF_FQ, lp >= LPF_OFF_FQ) {
			(true, true) => {
				// Filters are off
				context.set_latency_samples(0);
			}

			(false, true) => {
				// LP is off
				context.set_latency_samples(self.hp_mono.latency() as _);
				self.update_hp(hp, hp_q);
				self.reset_lp();

				components::apply_mono_stereo(
					&mut self.hp_mono,
					&mut self.hp_stereo,
					buffer.as_slice(),
				);
			}

			(true, false) => {
				// HP is off
				context.set_latency_samples(self.lp_mono.latency() as _);
				self.update_lp(lp, lp_q);
				self.reset_hp();

				components::apply_mono_stereo(
					&mut self.lp_mono,
					&mut self.lp_stereo,
					buffer.as_slice(),
				);
			}

			(false, false) => {
				// Both filters are active.
				context.set_latency_samples(
					self.hp_mono.latency() as u32 + self.lp_mono.latency() as u32,
				);
				self.update_hp(hp, hp_q);
				self.update_lp(lp, lp_q);

				components::apply_mono_stereo(
					(&mut self.hp_mono, &mut self.lp_mono),
					(&mut self.hp_stereo, &mut self.lp_stereo),
					buffer.as_slice(),
				);
			}
		}

		if gain != 0.0 {
			for samples in buffer.as_slice().iter_mut().take(2) {
				components::apply_gain(gain, samples);
			}
		}

		ProcessStatus::Normal
	}
}
