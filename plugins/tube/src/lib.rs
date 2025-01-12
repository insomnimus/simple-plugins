// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

use std::sync::Arc;

use components::{
	f64x2,
	ComponentMeta,
	Tube,
};
use nih_plug::{
	prelude::*,
	util::db_to_gain,
};

nih_export_clap!(TubePlugin);

#[derive(Debug, Params)]
struct TubeParams {
	#[id = "amnt"]
	amount: FloatParam,
	#[id = "in-gn"]
	input_gain: FloatParam,
	#[id = "ou-gn"]
	output_gain: FloatParam,
}

impl Default for TubeParams {
	fn default() -> Self {
		let gain = |name| {
			FloatParam::new(
				name,
				0.0,
				FloatRange::Linear {
					min: -50.0,
					max: 50.0,
				},
			)
			.with_unit("dB")
			.with_step_size(0.1)
		};

		Self {
			input_gain: gain("Input Gain"),
			output_gain: gain("Output Gain"),
			amount: FloatParam::new("Amount", 0.0, FloatRange::Linear { min: 0.0, max: 1.0 })
				.with_step_size(0.01)
				.with_value_to_string(Arc::new(|a| format!("{:.0}", a * 100.0)))
				.with_string_to_value(Arc::new(|s| {
					Some(s.parse::<f32>().unwrap_or(0.0).clamp(0.0, 100.0) / 100.0)
				})),
		}
	}
}

struct TubePlugin {
	params: Arc<TubeParams>,
	mono: Tube<f64>,
	stereo: Tube<f64x2>,
}

impl Default for TubePlugin {
	fn default() -> Self {
		Self {
			params: Arc::default(),
			mono: Tube::new(44100.0),
			stereo: Tube::new(44100.0),
		}
	}
}

impl ClapPlugin for TubePlugin {
	const CLAP_DESCRIPTION: Option<&'static str> = Some("Simple tube saturation");
	const CLAP_FEATURES: &'static [ClapFeature] = &[
		ClapFeature::AudioEffect,
		ClapFeature::Mono,
		ClapFeature::Stereo,
		ClapFeature::Distortion,
	];
	const CLAP_ID: &'static str = "insomnia.simple-tube";
	const CLAP_MANUAL_URL: Option<&'static str> = None;
	const CLAP_SUPPORT_URL: Option<&'static str> = Some(env!("CARGO_PKG_HOMEPAGE"));
}

impl Plugin for TubePlugin {
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
	const NAME: &'static str = "Simple Tube";
	const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
	const VENDOR: &'static str = "Insomnia";
	const VERSION: &'static str = env!("CARGO_PKG_VERSION");

	fn params(&self) -> Arc<dyn Params> {
		self.params.clone()
	}

	fn reset(&mut self) {
		self.mono.reset();
		self.stereo.reset();
	}

	fn initialize(
		&mut self,
		_layout: &AudioIOLayout,
		buffer_config: &BufferConfig,
		_context: &mut impl InitContext<Self>,
	) -> bool {
		self.stereo = Tube::new(buffer_config.sample_rate as _);
		self.mono = Tube::new(buffer_config.sample_rate as _);

		true
	}

	fn process(
		&mut self,
		buffer: &mut Buffer,
		_aux: &mut AuxiliaryBuffers,
		context: &mut impl ProcessContext<Self>,
	) -> ProcessStatus {
		let input_gain = db_to_gain(self.params.input_gain.value());
		let output_gain = db_to_gain(self.params.output_gain.value());
		let amount = self.params.amount.value() as f64;

		self.mono.set_amount(amount);
		self.stereo.set_amount(amount);

		components::apply_gain_mono_stereo(input_gain, buffer.as_slice());

		let latency =
			components::apply_mono_stereo(&mut self.mono, &mut self.stereo, buffer.as_slice());
		context.set_latency_samples(latency as _);

		components::apply_gain_mono_stereo(output_gain, buffer.as_slice());

		ProcessStatus::Normal
	}
}
