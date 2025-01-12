// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

mod eq;

use std::sync::Arc;

use components::{
	f64x2,
	ComponentMeta,
	DcBlocker,
	Toggle,
	Tube,
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
	#[id = "drive"]
	drive: FloatParam,
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
			drive: FloatParam::new("Drive", 0.0, FloatRange::Linear { min: 0.0, max: 1.0 })
				.with_step_size(0.01)
				.with_value_to_string(formatters::v2s_f32_percentage(0))
				.with_string_to_value(formatters::s2v_f32_percentage()),
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
					min: -30.0,
					max: 30.0,
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
	dc_mono: DcBlocker<f64>,
	dc_stereo: DcBlocker<f64x2>,
	drive_mono: Toggle<Tube<f64>>,
	drive_stereo: Toggle<Tube<f64x2>>,
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
				dc_mono: DcBlocker::new(44100.0),
				dc_stereo: DcBlocker::new(44100.0),
				drive_mono: Toggle::new(Tube::new(44100.0), false, true),
				drive_stereo: Toggle::new(Tube::new(44100.0), false, true),
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
		self.state.dc_mono.reset();
		self.state.dc_stereo.reset();

		self.state.drive_mono.reset();
		self.state.drive_stereo.reset();

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

		let drive = self.params.drive.value() as f64;

		*self.state = State {
			dc_mono: DcBlocker::new(self.sr),
			dc_stereo: DcBlocker::new(self.sr),
			drive_mono: Toggle::new(Tube::new(self.sr).with_amount(drive), drive > 0.0, true),
			drive_stereo: Toggle::new(Tube::new(self.sr).with_amount(drive), drive > 0.0, true),
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
		let drive = self.params.drive.value() as f64;

		let hp_active = if buffer.channels() == 1 {
			self.state.drive_mono.set_amount(drive);
			self.state.drive_mono.toggle(drive > 0.0);

			self.state.eq_mono.update_parameters(&self.params.eq);

			self.state.eq_mono.hp_active()
		} else {
			self.state.drive_stereo.set_amount(drive);
			self.state.drive_stereo.toggle(drive > 0.0);

			self.state.eq_stereo.update_parameters(&self.params.eq);

			self.state.eq_stereo.hp_active()
		};

		let input_gain = db_to_gain(self.params.input_gain.value());
		let output_gain = db_to_gain(self.params.output_gain.value());

		components::apply_gain_mono_stereo(input_gain, buffer.as_slice());

		let latency = components::apply_mono_stereo(
			(
				&mut self.state.drive_mono,
				&mut self.state.eq_mono,
				Toggle::new(&mut self.state.dc_mono, !hp_active, false),
			),
			(
				&mut self.state.drive_stereo,
				&mut self.state.eq_stereo,
				Toggle::new(&mut self.state.dc_stereo, !hp_active, false),
			),
			buffer.as_slice(),
		);

		components::apply_gain_mono_stereo(output_gain, buffer.as_slice());

		context.set_latency_samples(latency as _);
		ProcessStatus::Normal
	}
}
