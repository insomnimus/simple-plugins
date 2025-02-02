// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

use std::{
	collections::VecDeque,
	sync::Arc,
};

use components::{
	Component,
	ComponentMeta,
	Simper,
	SimperCoefficients,
	Toggle,
};
use nih_plug::prelude::*;

nih_export_clap!(DepthPlugin);

const MAX_DELAY_MS: f32 = 1.0;
const MAX_DEPTH: f32 = 40.0;
const ALL_PASS_Q: f64 = 0.5;

#[derive(Debug, Params)]
struct DepthParams {
	#[id = "depth"]
	depth: FloatParam,
	#[id = "alt"]
	alt_fq: FloatParam,
}

impl Default for DepthParams {
	fn default() -> Self {
		Self {
			depth: FloatParam::new(
				"Depth",
				0.0,
				FloatRange::Linear {
					min: 0.0,
					max: MAX_DEPTH,
				},
			)
			.with_step_size(1.0),
			alt_fq: FloatParam::new(
				"Alternate - Frequency",
				0.0,
				FloatRange::Skewed {
					min: 19.0,
					max: 20e3,
					factor: FloatRange::skew_factor(-2.0),
				},
			)
			.with_string_to_value(formatters::s2v_f32_hz_then_khz())
			.with_value_to_string(Arc::new(|f| {
				if f <= 20.0 {
					String::from("Off")
				} else if f < 1000.0 {
					format!("{f:.1}hz")
				} else {
					format!("{:.2}khz", f / 1000.0)
				}
			})),
		}
	}
}

fn split_mid_side(input: [f32; 2]) -> (f32, f32) {
	let [l, r] = input;

	// Encoder
	let mid = (l + r) * 0.5;
	let side = l - r;

	(mid, side)
}

fn join_mid_side(mid: f32, side: f32) -> [f32; 2] {
	// Decoder
	let l = mid + side * 0.5;
	let r = mid - side * 0.5;

	[l, r]
}

struct DepthPlugin {
	params: Arc<DepthParams>,
	sample_rate: f32,
	all_pass: Simper<f64>,
	// The mid channel, delayed.
	delay_buf: VecDeque<f32>,
}

impl Default for DepthPlugin {
	fn default() -> Self {
		Self {
			params: Arc::default(),
			delay_buf: VecDeque::new(),
			sample_rate: 44100.0,
			all_pass: Simper::all_pass(44100.0, 20.0, ALL_PASS_Q),
		}
	}
}

impl ClapPlugin for DepthPlugin {
	const CLAP_DESCRIPTION: Option<&'static str> = Some("Stereo depth enhancer");
	const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::Stereo, ClapFeature::AudioEffect];
	const CLAP_ID: &'static str = "insomnia.depth";
	const CLAP_MANUAL_URL: Option<&'static str> = None;
	const CLAP_SUPPORT_URL: Option<&'static str> = Some(env!("CARGO_PKG_HOMEPAGE"));
}

impl Plugin for DepthPlugin {
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
	const NAME: &'static str = "Depth";
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
		self.sample_rate = buffer_config.sample_rate;
		self.all_pass = Simper::all_pass(
			buffer_config.sample_rate as _,
			self.params.alt_fq.value() as _,
			ALL_PASS_Q,
		);
		let max_len = (buffer_config.sample_rate * MAX_DELAY_MS * 1e3) as usize;
		self.delay_buf.clear();
		self.delay_buf
			.reserve(usize::saturating_sub(self.delay_buf.capacity(), max_len));

		true
	}

	fn reset(&mut self) {
		self.delay_buf.clear();
		self.all_pass.reset();
	}

	fn process(
		&mut self,
		buffer: &mut Buffer,
		_aux: &mut AuxiliaryBuffers,
		_context: &mut impl ProcessContext<Self>,
	) -> ProcessStatus {
		let alt_fq = self.params.alt_fq.value();
		let delay_ms = (self.params.depth.value() / MAX_DEPTH) * MAX_DELAY_MS;
		let delay_len = f32::round(self.sample_rate * delay_ms * 1e-3) as usize;

		if delay_len == 0 && alt_fq <= 20.0 {
			self.delay_buf.clear();
			self.all_pass.reset();
			return ProcessStatus::Normal;
		}
		self.all_pass.set_parameters(SimperCoefficients::all_pass(
			self.sample_rate as _,
			alt_fq as _,
			ALL_PASS_Q,
		));

		let [samples_l, samples_r, ..] = buffer.as_slice() else {
			return ProcessStatus::Normal;
		};
		let mut samples = samples_l.iter_mut().zip(samples_r.iter_mut());

		if delay_len == 0 {
			// Only the all-pass filter is active.
			for (l, r) in samples {
				let (mid, side) = split_mid_side([*l, *r]);

				let mid = self.all_pass.process(mid as f64);
				let [new_l, new_r] = join_mid_side(mid as _, side);

				*l = new_l;
				*r = new_r;
			}

			return ProcessStatus::Normal;
		}

		let mut all_pass = Toggle::new(&mut self.all_pass, alt_fq > 20.0, false);

		if self.delay_buf.len() > delay_len {
			// The parameter has changed; truncate the buffer.
			self.delay_buf.truncate(delay_len);
		}
		// At this point, we never have more samples than we can deal with.

		let need_samples = delay_len - self.delay_buf.len();

		// Fill the buffer if it's not yet full.
		for (l, r) in samples.by_ref().take(need_samples) {
			let (mid, side) = split_mid_side([*l, *r]);
			self.delay_buf.push_back(all_pass.process(mid as _) as _);
			let [new_l, new_r] = join_mid_side(0.0, side);
			*l = new_l;
			*r = new_r;
		}

		// Process rest of the samples; the loop only executes if the buffer is full.
		for (l, r) in samples {
			let (mid, side) = split_mid_side([*l, *r]);
			let mid = all_pass.process(mid as _) as f32;
			let delayed_mid = self.delay_buf.pop_front().unwrap();
			let [new_l, new_r] = join_mid_side(delayed_mid, side);

			*l = new_l;
			*r = new_r;
			self.delay_buf.push_back(mid);
		}

		ProcessStatus::Tail(delay_len as _)
	}
}
