// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

use std::sync::Arc;

use components::{
	Component,
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

#[derive(Default)]
struct TubePlugin {
	params: Arc<TubeParams>,
	tubes: Vec<Tube<f64>>,
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
		for t in &mut self.tubes {
			t.reset();
		}
	}

	fn initialize(
		&mut self,
		layout: &AudioIOLayout,
		buffer_config: &BufferConfig,
		_context: &mut impl InitContext<Self>,
	) -> bool {
		let channels = layout.main_input_channels.map_or(1, |x| x.get()).min(2);
		self.tubes.clear();
		self.tubes
			.extend((0..channels).map(|_| Tube::new(buffer_config.sample_rate as _)));

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

		for (tube, samples) in self.tubes.iter_mut().zip(buffer.as_slice()) {
			tube.set_amount(amount);
			context.set_latency_samples(tube.latency() as _);

			if input_gain != 1.0 {
				components::apply_gain(input_gain, samples);
			}

			for sample in samples.iter_mut() {
				*sample = tube.process(*sample as _) as _;
			}

			if output_gain != 1.0 {
				components::apply_gain(output_gain, samples);
			}
		}

		ProcessStatus::Normal
	}
}
