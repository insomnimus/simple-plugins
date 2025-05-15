// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

mod eq;

use std::sync::Arc;

use components::ComponentMeta;
use nih_plug::{
	prelude::*,
	util::db_to_gain,
};

use self::eq::SundaraEq;

nih_export_clap!(SundaraMonitorsPlugin, SundaraMonitorsPlusPlugin);

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

#[derive(Default, Debug, Params)]
struct SundaraEqParams {}

#[derive(Default)]
struct SundaraMonitorsPlugin {
	eq: SundaraEq,
}

impl ClapPlugin for SundaraMonitorsPlugin {
	const CLAP_DESCRIPTION: Option<&'static str> =
		Some("Monitoring EQ curves for the HIFIMAN Sundara headphones");
	const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::Stereo];
	const CLAP_ID: &'static str = "insomnia.sundara-monitors";
	const CLAP_MANUAL_URL: Option<&'static str> = None;
	const CLAP_SUPPORT_URL: Option<&'static str> = Some(env!("CARGO_PKG_HOMEPAGE"));
}

impl Plugin for SundaraMonitorsPlugin {
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
	const NAME: &'static str = "Sundara Monitors";
	const SAMPLE_ACCURATE_AUTOMATION: bool = true;
	const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
	const VENDOR: &'static str = "Insomnia";
	const VERSION: &'static str = env!("CARGO_PKG_VERSION");

	fn params(&self) -> Arc<dyn Params> {
		Arc::new(SundaraEqParams {})
	}

	fn initialize(
		&mut self,
		_layout: &AudioIOLayout,
		buffer_config: &BufferConfig,
		_context: &mut impl InitContext<Self>,
	) -> bool {
		self.eq = SundaraEq::new(buffer_config.sample_rate);
		true
	}

	fn reset(&mut self) {
		self.eq.reset();
	}

	fn process(
		&mut self,
		buffer: &mut Buffer,
		_aux: &mut AuxiliaryBuffers,
		context: &mut impl ProcessContext<Self>,
	) -> ProcessStatus {
		let [left_samples, right_samples, ..] = buffer.as_slice() else {
			return ProcessStatus::Normal;
		};

		components::apply_stereo(&mut self.eq, left_samples, right_samples);

		context.set_latency_samples(self.eq.latency() as _);
		ProcessStatus::Normal
	}
}

#[derive(Debug, Params)]
struct SundaraMonitorsPlusParams {
	#[id = "gain"]
	gain: FloatParam,
	#[id = "left"]
	left: FloatParam,
	#[id = "right"]
	right: FloatParam,
}

impl Default for SundaraMonitorsPlusParams {
	fn default() -> Self {
		Self {
			gain: gain_param("Gain"),
			left: gain_param("Left"),
			right: gain_param("Right"),
		}
	}
}

#[derive(Default)]
struct SundaraMonitorsPlusPlugin {
	params: Arc<SundaraMonitorsPlusParams>,
	eq: SundaraEq,
}

impl ClapPlugin for SundaraMonitorsPlusPlugin {
	const CLAP_DESCRIPTION: Option<&'static str> =
		Some("Monitoring plugin for the HIFIMAN Sundara headphones");
	const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::Stereo];
	const CLAP_ID: &'static str = "insomnia.sundara-monitors+";
	const CLAP_MANUAL_URL: Option<&'static str> = None;
	const CLAP_SUPPORT_URL: Option<&'static str> = Some(env!("CARGO_PKG_HOMEPAGE"));
}

impl Plugin for SundaraMonitorsPlusPlugin {
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
	const NAME: &'static str = "Sundara Monitors+";
	const SAMPLE_ACCURATE_AUTOMATION: bool = false;
	const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
	const VENDOR: &'static str = "Insomnia";
	const VERSION: &'static str = env!("CARGO_PKG_VERSION");

	fn params(&self) -> Arc<dyn Params> {
		self.params.clone()
	}

	fn initialize(
		&mut self,
		_layout: &AudioIOLayout,
		buffer_config: &BufferConfig,
		_context: &mut impl InitContext<Self>,
	) -> bool {
		self.eq = SundaraEq::new(buffer_config.sample_rate);
		true
	}

	fn reset(&mut self) {
		self.eq.reset();
	}

	fn process(
		&mut self,
		buffer: &mut Buffer,
		_aux: &mut AuxiliaryBuffers,
		context: &mut impl ProcessContext<Self>,
	) -> ProcessStatus {
		let [left_samples, right_samples, ..] = buffer.as_slice() else {
			return ProcessStatus::Normal;
		};

		components::apply_stereo(&mut self.eq, left_samples, right_samples);

		let gain = db_to_gain(self.params.gain.value());
		let left_gain = gain * db_to_gain(self.params.left.value());
		let right_gain = gain * db_to_gain(self.params.right.value());

		components::apply_gain(left_gain, left_samples);
		components::apply_gain(right_gain, right_samples);

		context.set_latency_samples(self.eq.latency() as _);
		ProcessStatus::Normal
	}
}
