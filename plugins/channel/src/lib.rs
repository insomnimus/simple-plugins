// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

mod eq;

use std::sync::Arc;

use components::{
	f64x2,
	ComponentMeta,
};
use nih_plug::{
	prelude::*,
	util::db_to_gain,
};

use self::eq::{
	Eq,
	EqParams,
};

nih_export_clap!(ChannelPlugin);

#[derive(Params)]
struct ChannelParams {
	#[nested(group = "Eq")]
	eq: EqParams,
	#[id = "in_gn"]
	input_gain: FloatParam,
	#[id = "ou_gn"]
	output_gain: FloatParam,
}

impl Default for ChannelParams {
	fn default() -> Self {
		Self {
			eq: EqParams::default(),
			input_gain: FloatParam::new(
				"Input Gain",
				0.0,
				FloatRange::Linear {
					min: -50.0,
					max: 50.0,
				},
			)
			.with_unit("dB")
			.with_step_size(0.05),
			output_gain: FloatParam::new(
				"Output Gain",
				0.0,
				FloatRange::Linear {
					min: -50.0,
					max: 50.0,
				},
			)
			.with_unit("dB")
			.with_step_size(0.05),
		}
	}
}

struct ChannelPlugin {
	params: Arc<ChannelParams>,

	sr: f64,
	sr_f64x2: f64x2,
	state: Box<State>,
}

struct State {
	eq_mono: Eq<f64>,
	eq_stereo: Eq<f64x2>,
}

impl Default for ChannelPlugin {
	fn default() -> Self {
		let params = ChannelParams::default();

		Self {
			sr: 44100.0,
			sr_f64x2: f64x2::splat(44100.0),

			state: Box::new(State {
				eq_mono: Eq::new(44100.0, &params.eq),
				eq_stereo: Eq::new(f64x2::splat(44100.0), &params.eq),
			}),

			params: Arc::new(params),
		}
	}
}

impl ClapPlugin for ChannelPlugin {
	const CLAP_DESCRIPTION: Option<&'static str> = Some("Simple channel strip");
	const CLAP_FEATURES: &'static [ClapFeature] =
		&[ClapFeature::Filter, ClapFeature::Mono, ClapFeature::Stereo];
	const CLAP_ID: &'static str = "insomnia.simple-channel";
	const CLAP_MANUAL_URL: Option<&'static str> = None;
	const CLAP_SUPPORT_URL: Option<&'static str> = Some(env!("CARGO_PKG_HOMEPAGE"));
}

impl Plugin for ChannelPlugin {
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
	const NAME: &'static str = "Simple Channel Strip";
	const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
	const VENDOR: &'static str = "Insomnia";
	const VERSION: &'static str = env!("CARGO_PKG_VERSION");

	fn params(&self) -> Arc<dyn Params> {
		self.params.clone()
	}

	fn reset(&mut self) {
		self.state.eq_mono.reset();
		self.state.eq_stereo.reset();
	}

	fn initialize(
		&mut self,
		_layout: &AudioIOLayout,
		buffer_config: &BufferConfig,
		_context: &mut impl InitContext<Self>,
	) -> bool {
		self.sr = buffer_config.sample_rate as _;
		self.sr_f64x2 = f64x2::splat(buffer_config.sample_rate as _);

		*self.state = State {
			eq_mono: Eq::new(self.sr, &self.params.eq),
			eq_stereo: Eq::new(self.sr_f64x2, &self.params.eq),
		};

		true
	}

	fn process(
		&mut self,
		buffer: &mut Buffer,
		_aux: &mut AuxiliaryBuffers,
		context: &mut impl ProcessContext<Self>,
	) -> ProcessStatus {
		if buffer.channels() == 1 {
			self.state.eq_mono.update_parameters(&self.params.eq);
			context.set_latency_samples(self.state.eq_mono.latency() as _);
		} else {
			self.state.eq_stereo.update_parameters(&self.params.eq);
			context.set_latency_samples(self.state.eq_mono.latency() as _);
		}

		let input_gain = db_to_gain(self.params.input_gain.value());
		let output_gain = db_to_gain(self.params.output_gain.value());

		if input_gain != 1.0 {
			for channel in buffer.as_slice().iter_mut() {
				components::apply_gain(input_gain, channel);
			}
		}

		components::apply_mono_stereo(
			&mut self.state.eq_mono,
			&mut self.state.eq_stereo,
			buffer.as_slice(),
		);

		if output_gain != 1.0 {
			for channel in buffer.as_slice().iter_mut() {
				components::apply_gain(output_gain, channel);
			}
		}

		ProcessStatus::Normal
	}
}
