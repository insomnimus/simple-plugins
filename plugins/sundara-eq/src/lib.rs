// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

use std::sync::Arc;

use components::{
	f64x2,
	Cascade,
	Component,
	ComponentMeta,
	SimdFloat,
	Simper,
};
use nih_plug::{
	prelude::*,
	util::db_to_gain,
};

nih_export_clap!(SundaraEqPlugin);

#[derive(Default, Debug, Params)]
struct SundaraEqParams {}

struct SundaraEqPlugin {
	filters: Cascade<Simper<f64x2>, 3>,
	preamp: f64x2,
}

impl Default for SundaraEqPlugin {
	fn default() -> Self {
		let filters = Cascade(core::array::from_fn(|_| {
			Simper::bell(
				f64x2::splat(44100.0),
				f64x2::splat(5000.0),
				Simper::BUTTERWORTH_Q,
				f64x2::splat(0.0),
			)
		}));

		let preamp = f64x2::splat(db_to_gain(-4.0) as _);

		Self { filters, preamp }
	}
}

impl ClapPlugin for SundaraEqPlugin {
	const CLAP_DESCRIPTION: Option<&'static str> =
		Some("Monitoring EQ curves for the HIFIMAN Sundara headphones");
	const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::Stereo];
	const CLAP_ID: &'static str = "insomnia.sundara-eq";
	const CLAP_MANUAL_URL: Option<&'static str> = None;
	const CLAP_SUPPORT_URL: Option<&'static str> = Some(env!("CARGO_PKG_HOMEPAGE"));
}

impl Plugin for SundaraEqPlugin {
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
	const NAME: &'static str = "Sundara Eq";
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
		let sr = f64x2::splat(buffer_config.sample_rate as _);
		let filters: [(fn(f64x2, f64x2, f64x2, f64x2) -> Simper<f64x2>, _, _, _); 3] = [
			(Simper::low_shelf, 50.0, 1.0, 3.5),
			(Simper::bell, 2112.0, 1.5, 2.5),
			(Simper::bell, 6300.0, 4.0, -3.0),
		];

		self.filters =
			Cascade(filters.map(|(f, fq, q, gain)| {
				f(sr, f64x2::splat(fq), f64x2::splat(q), f64x2::splat(gain))
			}));

		true
	}

	fn reset(&mut self) {
		self.filters.reset();
	}

	fn process(
		&mut self,
		buffer: &mut Buffer,
		_aux: &mut AuxiliaryBuffers,
		_context: &mut impl ProcessContext<Self>,
	) -> ProcessStatus {
		let [input_l, input_r] = buffer.as_slice() else {
			return ProcessStatus::Normal;
		};

		for (l, r) in input_l.iter_mut().zip(input_r.iter_mut()) {
			let sample = f64x2::new([*l as _, *r as _]);
			let [new_l, new_r] = self.filters.process(sample * self.preamp).value();
			*l = new_l as _;
			*r = new_r as _;
		}

		ProcessStatus::Normal
	}
}
