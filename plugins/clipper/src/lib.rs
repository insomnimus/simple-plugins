// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

mod simd;

use std::sync::Arc;

use components::Oversampler2 as Oversampler;
use nih_plug::{
	prelude::*,
	util::db_to_gain,
};

nih_export_clap!(ClipperPlugin);

const MAX_OVERSAMPLE: u8 = 4;

#[derive(Debug, Params)]
struct ClipperParams {
	#[id = "threshold"]
	threshold: FloatParam,
	#[id = "input-gain"]
	input_gain: FloatParam,
	#[id = "output-gain"]
	output_gain: FloatParam,
	#[id = "oversample"]
	oversample: IntParam,
}

impl Default for ClipperParams {
	fn default() -> Self {
		let p = |name: &str, default: f32, min: f32, max: f32| {
			FloatParam::new(
				name,
				default,
				FloatRange::Linear {
					min,
					max,
					// factor: FloatRange::gain_skew_factor(min, max),
				},
			)
			.with_unit(" dB")
			.with_step_size(0.1)
			// .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
			// .with_string_to_value(formatters::s2v_f32_gain_to_db())
		};

		Self {
			threshold: p("Threshold", 0.0, -45.0, 15.0),
			input_gain: p("Input Gain", 0.0, -30.0, 30.0),
			output_gain: p("Output Gain", 0.0, -30.0, 30.0),
			oversample: IntParam::new(
				"Oversampling",
				0,
				IntRange::Linear {
					min: 0,
					max: MAX_OVERSAMPLE as _,
				},
			)
			.with_string_to_value(formatters::s2v_i32_power_of_two())
			.with_value_to_string(Arc::new(|n| {
				if n == 0 {
					"Off".to_owned()
				} else {
					format!("{}x", usize::pow(2, n as _))
				}
			})),
		}
	}
}

#[derive(Default)]
struct ClipperPlugin {
	params: Arc<ClipperParams>,
	os: Vec<Oversampler>,
	is_rendering: bool,
}

impl ClapPlugin for ClipperPlugin {
	const CLAP_DESCRIPTION: Option<&'static str> = Some("Simple hard clipper");
	const CLAP_FEATURES: &'static [ClapFeature] = &[
		ClapFeature::AudioEffect,
		ClapFeature::Distortion,
		ClapFeature::Mono,
		ClapFeature::Stereo,
	];
	const CLAP_ID: &'static str = "insomnia.simple-clipper";
	const CLAP_MANUAL_URL: Option<&'static str> = None;
	const CLAP_SUPPORT_URL: Option<&'static str> = Some(env!("CARGO_PKG_HOMEPAGE"));
}

impl Plugin for ClipperPlugin {
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
	const NAME: &'static str = "Simple Clipper";
	const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
	const VENDOR: &'static str = "Insomnia";
	const VERSION: &'static str = env!("CARGO_PKG_VERSION");

	fn params(&self) -> Arc<dyn Params> {
		self.params.clone()
	}

	fn initialize(
		&mut self,
		layout: &AudioIOLayout,
		buffer_config: &BufferConfig,
		_context: &mut impl InitContext<Self>,
	) -> bool {
		self.is_rendering = buffer_config.process_mode == ProcessMode::Offline;

		self.os.clear();
		self.os.extend(
			(0..layout.main_input_channels.map_or(0, |n| n.get()))
				.map(|_| Oversampler::new(buffer_config.max_buffer_size as _, MAX_OVERSAMPLE, 0)),
		);

		true
	}

	fn reset(&mut self) {
		for o in &mut self.os {
			o.reset();
		}
	}

	fn process(
		&mut self,
		buffer: &mut Buffer,
		_aux: &mut AuxiliaryBuffers,
		context: &mut impl ProcessContext<Self>,
	) -> ProcessStatus {
		let factor = if self.is_rendering {
			u8::max(3, self.params.oversample.value() as _)
		} else {
			self.params.oversample.value() as _
		};

		for o in &mut self.os {
			o.set_oversampling_factor(factor as _);
		}

		let latency = self.os.first().map(|o| o.latency()).unwrap_or(0);
		context.set_latency_samples(latency as _);

		let threshold = db_to_gain(self.params.threshold.value());
		let input_gain = db_to_gain(self.params.input_gain.value());
		let output_gain = db_to_gain(self.params.output_gain.value());

		for (os, channel) in self.os.iter_mut().zip(buffer.as_slice().iter_mut()) {
			os.process_block(channel, |samples| {
				simd::process32_runtime_select(threshold, input_gain, output_gain, samples)
			});
		}

		ProcessStatus::Normal
	}
}
