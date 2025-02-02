// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright 2024-2025 Taylan Gökkaya

use std::{
	collections::VecDeque,
	sync::Arc,
};

use nih_plug::prelude::*;

nih_export_clap!(DepthPlugin);

const MAX_DELAY_MS: f32 = 1.0;
const MAX_DEPTH: f32 = 40.0;

#[derive(Debug, Params)]
struct DepthParams {
	#[id = "depth"]
	depth: FloatParam,
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

#[derive(Default)]
struct DepthPlugin {
	params: Arc<DepthParams>,
	sample_rate: f32,
	// The mid channel, delayed.
	delay_buf: VecDeque<f32>,
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
		let max_len = (buffer_config.sample_rate * MAX_DELAY_MS * 1e3) as usize;
		self.delay_buf.clear();
		self.delay_buf
			.reserve(usize::saturating_sub(self.delay_buf.capacity(), max_len));

		true
	}

	fn reset(&mut self) {
		self.delay_buf.clear();
	}

	fn process(
		&mut self,
		buffer: &mut Buffer,
		_aux: &mut AuxiliaryBuffers,
		_context: &mut impl ProcessContext<Self>,
	) -> ProcessStatus {
		let delay_ms = (self.params.depth.value() / MAX_DEPTH) * MAX_DELAY_MS;
		let delay_len = f32::round(self.sample_rate * delay_ms * 1e-3) as usize;

		if delay_len == 0 {
			self.delay_buf.clear();
			return ProcessStatus::Normal;
		}

		let [samples_l, samples_r, ..] = buffer.as_slice() else {
			return ProcessStatus::Normal;
		};
		let mut samples = samples_l.iter_mut().zip(samples_r.iter_mut());

		if self.delay_buf.len() > delay_len {
			// The parameter has changed; truncate the buffer.
			self.delay_buf.truncate(delay_len);
		}
		// At this point, we never have more samples than we can deal with.

		let need_samples = delay_len - self.delay_buf.len();

		// Fill the buffer if it's not yet full.
		for (l, r) in samples.by_ref().take(need_samples) {
			let (mid, side) = split_mid_side([*l, *r]);
			self.delay_buf.push_back(mid);
			let [new_l, new_r] = join_mid_side(0.0, side);
			*l = new_l;
			*r = new_r;
		}

		// Process rest of the samples; the loop only executes if the buffer is full.
		for (l, r) in samples {
			let (mid, side) = split_mid_side([*l, *r]);
			let delayed_mid = self.delay_buf.pop_front().unwrap();
			let [new_l, new_r] = join_mid_side(delayed_mid, side);
			*l = new_l;
			*r = new_r;
			self.delay_buf.push_back(mid);
		}

		ProcessStatus::Tail(delay_len as _)
	}
}
