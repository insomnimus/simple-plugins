mod dsp;

use std::sync::Arc;

use nih_plug::prelude::*;
use valib::{
	contrib::nih_plug::{
		process_buffer_simd,
		BindToParameter,
	},
	dsp::{
		parameter::{
			RemoteControl,
			RemoteControlled,
		},
		DSPMeta,
	},
};

use self::dsp::{
	Dsp,
	DspParam,
};

nih_export_clap!(ClipperPlugin);

const OVERSAMPLE: usize = 8;
const MAX_BLOCK_SIZE: usize = 512;

#[derive(Debug, Params)]
struct ClipperParams {
	#[id = "threshold"]
	threshold: FloatParam,
	#[id = "input-gain"]
	input_gain: FloatParam,
	#[id = "output-gain"]
	output_gain: FloatParam,
	#[id = "oversample"]
	oversample: BoolParam,
}

impl ClipperParams {
	fn new(remote: &RemoteControl<DspParam>) -> Arc<Self> {
		let p = |name: &str, default: f32, min: f32, max: f32| {
			FloatParam::new(
				name,
				default,
				FloatRange::Linear {
					min,
					max,
					// factor: FloatRange::gain_skew_factor(min, max),
				},
			)
			.with_unit(" dB")
			.with_step_size(0.1)
			// .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
			// .with_string_to_value(formatters::s2v_f32_gain_to_db())
		};

		Arc::new(Self {
			threshold: p("Threshold", 0.0, -45.0, 15.0)
				.bind_to_parameter(remote, DspParam::Threshold),
			input_gain: p("Input Gain", 0.0, -30.0, 30.0)
				.bind_to_parameter(remote, DspParam::InputGain),
			output_gain: p("Output Gain", 0.0, -30.0, 30.0)
				.bind_to_parameter(remote, DspParam::OutputGain),
			oversample: BoolParam::new("Oversample", false)
				.bind_to_parameter(remote, DspParam::Oversample),
		})
	}
}

struct ClipperPlugin {
	dsp: RemoteControlled<Dsp>,
	params: Arc<ClipperParams>,
}

impl Default for ClipperPlugin {
	fn default() -> Self {
		let dsp = dsp::create(44100.0);
		let params = ClipperParams::new(&dsp.proxy);
		Self { dsp, params }
	}
}

impl Plugin for ClipperPlugin {
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
	const NAME: &'static str = "Basic Clipper";
	const URL: &'static str = "https://github.com/insomnimus/basic-clipper";
	const VENDOR: &'static str = "Insomnia";
	const VERSION: &'static str = env!("CARGO_PKG_VERSION");

	fn params(&self) -> Arc<dyn Params> {
		self.params.clone()
	}

	fn initialize(
		&mut self,
		_audio_io_layout: &AudioIOLayout,
		buffer_config: &BufferConfig,
		_context: &mut impl InitContext<Self>,
	) -> bool {
		self.dsp.set_samplerate(buffer_config.sample_rate);
		true
	}

	fn reset(&mut self) {
		self.dsp.reset();
	}

	fn process(
		&mut self,
		buffer: &mut Buffer,
		_aux: &mut AuxiliaryBuffers,
		_context: &mut impl ProcessContext<Self>,
	) -> ProcessStatus {
		_context.set_latency_samples(self.dsp.latency() as _);
		process_buffer_simd::<_, _, MAX_BLOCK_SIZE>(&mut self.dsp, buffer);
		ProcessStatus::Normal
	}
}

impl ClapPlugin for ClipperPlugin {
	const CLAP_DESCRIPTION: Option<&'static str> = Some("Basic hard clipper");
	const CLAP_FEATURES: &'static [ClapFeature] = &[
		ClapFeature::AudioEffect,
		ClapFeature::Distortion,
		ClapFeature::Mono,
		ClapFeature::Stereo,
	];
	const CLAP_ID: &'static str = "insomnia.basic-clipper";
	const CLAP_MANUAL_URL: Option<&'static str> = None;
	const CLAP_SUPPORT_URL: Option<&'static str> = None;
}
