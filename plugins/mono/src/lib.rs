// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

mod simd;

use std::sync::Arc;

use nih_plug::prelude::*;

nih_export_clap!(MonoPlugin);

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Enum, Default)]
enum Mode {
	#[default]
	Mix,
	Left,
	Right,
}

#[derive(Debug, Params)]
struct MonoParams {
	#[id = "mode"]
	mode: EnumParam<Mode>,
}

impl Default for MonoParams {
	fn default() -> Self {
		Self {
			mode: EnumParam::new("Mode", Mode::Mix),
		}
	}
}

#[derive(Default)]
struct MonoPlugin {
	params: Arc<MonoParams>,
}

impl ClapPlugin for MonoPlugin {
	const CLAP_DESCRIPTION: Option<&'static str> = Some("Stereo to mono utility");
	const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::Utility, ClapFeature::Mono];
	const CLAP_ID: &'static str = "insomnia.mono";
	const CLAP_MANUAL_URL: Option<&'static str> = None;
	const CLAP_SUPPORT_URL: Option<&'static str> = Some(env!("CARGO_PKG_HOMEPAGE"));
}

impl Plugin for MonoPlugin {
	type BackgroundTask = ();
	type SysExMessage = ();

	const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
		main_input_channels: Some(new_nonzero_u32(2)),
		main_output_channels: Some(new_nonzero_u32(1)),
		aux_input_ports: &[],
		aux_output_ports: &[],
		names: PortNames {
			layout: Some("Stereo to mono"),
			main_input: Some("Input"),
			main_output: Some("Output"),
			aux_inputs: &[],
			aux_outputs: &[],
		},
	}];
	const EMAIL: &'static str = "";
	const NAME: &'static str = "Mono";
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
		let [ch1, ch2, ..] = buffer.as_slice() else {
			return ProcessStatus::Normal;
		};

		match self.params.mode.value() {
			Mode::Mix => simd::mix_runtime_select(ch1, ch2),
			Mode::Left => (),
			Mode::Right => ch1.copy_from_slice(ch2),
		}

		ProcessStatus::Normal
	}
}
