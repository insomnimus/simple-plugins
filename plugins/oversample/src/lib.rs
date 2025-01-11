// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

use std::sync::Arc;

use components::{
	AdOversampler1,
	AdOversampler2,
};
use nih_plug::prelude::*;

nih_export_clap!(OversamplePlugin);

const OS: u8 = 7;

#[derive(Copy, Clone, Eq, PartialEq, Enum)]
enum OsModel {
	Os1,
	Os2,
}

#[derive(Debug, Params)]
struct OsParams {
	#[id = "factor"]
	factor: IntParam,
	#[id = "model"]
	model: EnumParam<OsModel>,
}

impl Default for OsParams {
	fn default() -> Self {
		Self {
			factor: IntParam::new(
				"factor",
				0,
				IntRange::Linear {
					min: 0,
					max: OS as _,
				},
			),
			model: EnumParam::new("Model", OsModel::Os1),
		}
	}
}

struct OversamplePlugin {
	params: Arc<OsParams>,
	os1: Vec<AdOversampler1>,
	os2: Vec<AdOversampler2>,
	model: OsModel,
}

impl Default for OversamplePlugin {
	fn default() -> Self {
		Self {
			params: Arc::new(OsParams::default()),
			os1: Vec::new(),
			os2: Vec::new(),
			model: OsModel::Os1,
		}
	}
}

impl ClapPlugin for OversamplePlugin {
	const CLAP_DESCRIPTION: Option<&'static str> = Some("Useless, just oversamples");
	const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::Filter, ClapFeature::Stereo];
	const CLAP_ID: &'static str = "insomnia.oversample";
	const CLAP_MANUAL_URL: Option<&'static str> = None;
	const CLAP_SUPPORT_URL: Option<&'static str> = Some(env!("CARGO_PKG_HOMEPAGE"));
}

impl Plugin for OversamplePlugin {
	type BackgroundTask = ();
	type SysExMessage = ();

	const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
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
	}];
	const EMAIL: &'static str = "";
	const NAME: &'static str = "Oversample";
	const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
	const VENDOR: &'static str = "Insomnia";
	const VERSION: &'static str = env!("CARGO_PKG_VERSION");

	fn params(&self) -> Arc<dyn Params> {
		self.params.clone()
	}

	fn reset(&mut self) {
		for x in &mut self.os1 {
			x.reset();
		}
		for x in &mut self.os2 {
			x.reset();
		}
	}

	fn initialize(
		&mut self,
		layout: &AudioIOLayout,
		buffer_config: &BufferConfig,
		_context: &mut impl InitContext<Self>,
	) -> bool {
		let channels = layout.main_input_channels.map_or(1, |x| x.get()).min(2);
		self.os1.clear();
		self.os2.clear();

		for _ in 0..channels {
			self.os1.push(AdOversampler1::new(
				usize::min(buffer_config.max_buffer_size as usize / 2, 32),
				buffer_config.sample_rate as _,
				OS,
				0,
			));

			self.os2.push(AdOversampler2::new(
				buffer_config.max_buffer_size as _,
				OS,
				0,
			));
		}

		true
	}

	fn process(
		&mut self,
		buffer: &mut Buffer,
		_aux: &mut AuxiliaryBuffers,
		context: &mut impl ProcessContext<Self>,
	) -> ProcessStatus {
		let factor = self.params.factor.value() as u8;

		let model = self.params.model.value();
		if model != self.model {
			match model {
				OsModel::Os2 => self.os2.iter_mut().for_each(|x| x.reset()),
				OsModel::Os1 => self.os1.iter_mut().for_each(|x| x.reset()),
			}

			self.model = model;
		}

		let mut latency = 0;

		match self.model {
			OsModel::Os1 => {
				for (os, channel) in self.os1.iter_mut().zip(buffer.as_slice()) {
					os.set_oversampling_factor(factor);
					latency = os.latency();
					os.process_block(channel, |_| ());
				}
			}

			OsModel::Os2 => {
				for (os, channel) in self.os2.iter_mut().zip(buffer.as_slice()) {
					os.set_oversampling_factor(factor);
					latency = os.latency();
					os.process_block(channel, |_| ());
				}
			}
		}

		context.set_latency_samples(latency as _);
		ProcessStatus::Normal
	}
}
