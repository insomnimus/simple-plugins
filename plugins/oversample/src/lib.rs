// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

use std::sync::Arc;

use components::Oversampler2 as Oversampler;
use nih_plug::prelude::*;

nih_export_clap!(OversamplePlugin);

const OS: u8 = 8;

#[derive(Debug, Params)]
struct OsParams {
	#[id = "times"]
	times: IntParam,
}

impl Default for OsParams {
	fn default() -> Self {
		Self {
			times: IntParam::new(
				"Times",
				0,
				IntRange::Linear {
					min: 0,
					max: OS as _,
				},
			),
		}
	}
}

struct OversamplePlugin {
	params: Arc<OsParams>,
	l: Oversampler,
	r: Oversampler,
}

impl Default for OversamplePlugin {
	fn default() -> Self {
		Self {
			params: Arc::new(OsParams::default()),
			l: Oversampler::new(1, OS, 0),
			r: Oversampler::new(1, OS, 0),
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
		self.l.reset();
		self.r.reset();
	}

	fn initialize(
		&mut self,
		_layout: &AudioIOLayout,
		buffer_config: &BufferConfig,
		_context: &mut impl InitContext<Self>,
	) -> bool {
		self.l = Oversampler::new(buffer_config.max_buffer_size as _, OS, 0);
		self.r = Oversampler::new(buffer_config.max_buffer_size as _, OS, 0);

		true
	}

	fn process(
		&mut self,
		buffer: &mut Buffer,
		_aux: &mut AuxiliaryBuffers,
		context: &mut impl ProcessContext<Self>,
	) -> ProcessStatus {
		if let [l, r] = buffer.as_slice() {
			let times = self.params.times.value();
			self.l.set_oversampling_factor(times as _);
			self.r.set_oversampling_factor(times as _);
			context.set_latency_samples(self.l.latency() as _);
			nih_plug::nih_log!("latency: {}", self.l.latency());
			self.l.process_block(l, |_| ());
			self.r.process_block(r, |_| ());
		}

		ProcessStatus::Normal
	}
}
