// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

mod simd;

use std::sync::Arc;

use components::Oversampler1 as Oversampler;
use nih_plug::{
	prelude::*,
	util::db_to_gain,
};

nih_export_clap!(ClipperPlugin);

const MAX_OVERSAMPLE: u8 = 8;

#[derive(Debug, Params)]
struct ClipperParams {
	#[id = "threshold"]
	threshold: FloatParam,
	#[id = "input-gain"]
	input_gain: FloatParam,
	#[id = "output-gain"]
	output_gain: FloatParam,
	#[id = "oversample"]
	oversample: BoolParam,
	#[id = "au-os"]
	auto_oversample: BoolParam,
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
			oversample: BoolParam::new("Oversample", false).with_value_to_string(Arc::new(
				|enabled| {
					if enabled {
						format!("{MAX_OVERSAMPLE}x")
					} else {
						"Off".to_owned()
					}
				},
			)),
			auto_oversample: BoolParam::new("Oversample On Render", true).with_value_to_string(
				Arc::new(|enabled| {
					if enabled {
						"On".to_owned()
					} else {
						"Off".to_owned()
					}
				}),
			),
		}
	}
}

#[derive(Default)]
struct ClipperPlugin {
	params: Arc<ClipperParams>,
	oversamplers: Vec<Oversampler>,
	oversampler_was_active: bool,
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
		let block_size = usize::min(buffer_config.max_buffer_size as usize / 2, 32);

		let channels = layout.main_input_channels.map_or(1, |x| x.get()).min(2);

		if self.oversamplers.len() != channels as usize {
			self.oversamplers.clear();
			self.oversamplers.extend((0..channels).map(|_| {
				Oversampler::new(block_size, buffer_config.sample_rate as _, MAX_OVERSAMPLE)
			}));
		} else if self.oversamplers[0].orig_sample_rate() == buffer_config.sample_rate as usize
			&& self.oversamplers[0].block_size() == block_size
		{
			// Saves us some reallocations.
			for os in &mut self.oversamplers {
				os.reset();
			}
		} else {
			self.oversamplers.clear();
			self.oversamplers.extend((0..channels).map(|_| {
				Oversampler::new(block_size, buffer_config.sample_rate as _, MAX_OVERSAMPLE)
			}));
		}

		true
	}

	fn reset(&mut self) {
		for os in &mut self.oversamplers {
			os.reset();
		}
	}

	fn process(
		&mut self,
		buffer: &mut Buffer,
		_aux: &mut AuxiliaryBuffers,
		context: &mut impl ProcessContext<Self>,
	) -> ProcessStatus {
		let threshold = db_to_gain(self.params.threshold.value());
		let input_gain = db_to_gain(self.params.input_gain.value());
		let output_gain = db_to_gain(self.params.output_gain.value());

		let oversample = self.params.oversample.value()
			|| (self.is_rendering && self.params.auto_oversample.value());

		if oversample && !self.oversampler_was_active {
			// Clear out the buffers so we don't work with stale samples.
			for os in &mut self.oversamplers {
				os.reset();
			}
			self.oversampler_was_active = true;
		}

		self.oversampler_was_active = oversample;

		if oversample {
			for (samples, oversampler) in buffer.as_slice().iter_mut().zip(&mut self.oversamplers) {
				context.set_latency_samples(oversampler.latency() as _);

				oversampler.process_block(samples, |samples| {
					simd::process32_runtime_select(threshold, input_gain, output_gain, samples);
				})
			}
		} else {
			context.set_latency_samples(0);

			for samples in buffer.as_slice().iter_mut().take(2) {
				simd::process32_runtime_select(threshold, input_gain, output_gain, samples);
			}
		}

		ProcessStatus::Normal
	}
}
