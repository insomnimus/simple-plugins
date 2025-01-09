// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

use std::sync::Arc;

use nih_plug::{
	prelude::*,
	util::db_to_gain,
};

nih_export_clap!(GainMonoPlugin, GainPlugin);

fn gain_param(name: &str) -> FloatParam {
	FloatParam::new(
		name,
		0.0,
		FloatRange::Linear {
			min: -50.0,
			max: 50.0,
		},
	)
	.with_unit(" dB")
	.with_step_size(0.05)
}

#[derive(Debug, Params)]
struct GainMonoParams {
	#[id = "gain"]
	gain: FloatParam,
}

#[derive(Debug, Params)]
struct GainParams {
	#[id = "gain"]
	gain: FloatParam,
	#[id = "left"]
	left: FloatParam,
	#[id = "right"]
	right: FloatParam,
}

impl Default for GainMonoParams {
	fn default() -> Self {
		Self {
			gain: gain_param("Gain"),
		}
	}
}

impl Default for GainParams {
	fn default() -> Self {
		Self {
			gain: gain_param("Gain"),
			left: gain_param("Left"),
			right: gain_param("Right"),
		}
	}
}

#[derive(Default)]
struct GainMonoPlugin {
	params: Arc<GainMonoParams>,
}

#[derive(Default)]
struct GainPlugin {
	params: Arc<GainParams>,
}

impl ClapPlugin for GainMonoPlugin {
	const CLAP_DESCRIPTION: Option<&'static str> = Some("Simple mono gain");
	const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::Utility, ClapFeature::Mono];
	const CLAP_ID: &'static str = "insomnia.simple-gain-mono";
	const CLAP_MANUAL_URL: Option<&'static str> = None;
	const CLAP_SUPPORT_URL: Option<&'static str> = Some(env!("CARGO_PKG_HOMEPAGE"));
}

impl ClapPlugin for GainPlugin {
	const CLAP_DESCRIPTION: Option<&'static str> = Some("Simple stereo gain");
	const CLAP_FEATURES: &'static [ClapFeature] = &[
		ClapFeature::Utility,
		ClapFeature::Stereo,
		ClapFeature::Surround,
	];
	const CLAP_ID: &'static str = "insomnia.simple-gain";
	const CLAP_MANUAL_URL: Option<&'static str> = None;
	const CLAP_SUPPORT_URL: Option<&'static str> = Some(env!("CARGO_PKG_HOMEPAGE"));
}

impl Plugin for GainMonoPlugin {
	type BackgroundTask = ();
	type SysExMessage = ();

	const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
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
	}];
	const EMAIL: &'static str = "";
	const NAME: &'static str = "Simple Gain Mono";
	const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
	const VENDOR: &'static str = "Insomnia";
	const VERSION: &'static str = env!("CARGO_PKG_VERSION");

	fn params(&self) -> Arc<dyn Params> {
		self.params.clone()
	}

	fn process(
		&mut self,
		buffer: &mut Buffer,
		_aux: &mut AuxiliaryBuffers,
		_context: &mut impl ProcessContext<Self>,
	) -> ProcessStatus {
		if let [samples, ..] = buffer.as_slice() {
			let gain = db_to_gain(self.params.gain.value());
			components::apply_gain(gain, samples);
		}

		ProcessStatus::Normal
	}
}

impl Plugin for GainPlugin {
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
	const NAME: &'static str = "Simple Gain Stereo";
	const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
	const VENDOR: &'static str = "Insomnia";
	const VERSION: &'static str = env!("CARGO_PKG_VERSION");

	fn params(&self) -> Arc<dyn Params> {
		self.params.clone()
	}

	fn process(
		&mut self,
		buffer: &mut Buffer,
		_aux: &mut AuxiliaryBuffers,
		_context: &mut impl ProcessContext<Self>,
	) -> ProcessStatus {
		if let [left_samples, right_samples, ..] = buffer.as_slice() {
			let gain = db_to_gain(self.params.gain.value());
			let left_gain = gain * db_to_gain(self.params.left.value());
			let right_gain = gain * db_to_gain(self.params.right.value());

			components::apply_gain(left_gain, left_samples);
			components::apply_gain(right_gain, right_samples);
		}

		ProcessStatus::Normal
	}
}
