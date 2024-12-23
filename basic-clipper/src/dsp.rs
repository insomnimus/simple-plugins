use nih_plug::util::db_to_gain;
use valib::{
	dsp::{
		buffer::AudioBuffer,
		parameter::{
			HasParameters,
			ParamId,
			ParamName,
			RemoteControlled,
		},
		BlockAdapter,
		DSPMeta,
		DSPProcess,
		DSPProcessBlock,
	},
	oversample::{
		Oversample,
		Oversampled,
	},
	simd::{
		AutoF32x2,
		SimdPartialOrd,
	},
	SimdCast,
};

use crate::{
	MAX_BLOCK_SIZE,
	OVERSAMPLE,
};

type Sample = AutoF32x2;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ParamName)]
pub enum DspParam {
	Threshold,
	InputGain,
	OutputGain,
	Oversample,
}

struct DspInner {
	threshold: f32,
	input_gain: f32,
	output_gain: f32,
}

impl HasParameters for DspInner {
	type Name = DspParam;

	fn set_parameter(&mut self, param: Self::Name, val: f32) {
		let val = db_to_gain(val);
		match param {
			DspParam::Threshold => self.threshold = val,
			DspParam::InputGain => self.input_gain = val,
			DspParam::OutputGain => self.output_gain = val,
			_ => (),
		}
	}
}

impl DSPMeta for DspInner {
	type Sample = Sample;

	fn latency(&self) -> usize {
		0
	}
}

impl DSPProcess<1, 1> for DspInner {
	fn process(&mut self, x: [Self::Sample; 1]) -> [Self::Sample; 1] {
		let threshold = Sample::new(self.threshold, self.threshold);
		let input = Sample::new(self.input_gain, self.input_gain);
		let output = Sample::new(self.output_gain, self.output_gain);

		x.map(move |x| (x.cast() * input).simd_clamp(-threshold, threshold) * output)
			.map(|x| x.cast())
	}
}

pub struct Dsp {
	inner: Oversampled<Sample, BlockAdapter<DspInner>>,
}

impl HasParameters for Dsp {
	type Name = DspParam;

	fn set_parameter(&mut self, param: Self::Name, val: f32) {
		match param {
			DspParam::Oversample => {
				let val = if val < 0.5 { 1 } else { OVERSAMPLE };
				self.inner.set_oversampling_amount(val);
			}
			_ => self.inner.inner.set_parameter(param, val),
		}
	}
}

impl DSPMeta for Dsp {
	type Sample = Sample;

	fn set_samplerate(&mut self, sr: f32) {
		self.inner.set_samplerate(sr);
	}

	fn latency(&self) -> usize {
		self.inner.latency()
	}

	fn reset(&mut self) {
		self.inner.reset();
	}
}

impl DSPProcessBlock<1, 1> for Dsp {
	fn process_block(
		&mut self,
		inputs: AudioBuffer<&[Self::Sample], 1>,
		outputs: AudioBuffer<&mut [Self::Sample], 1>,
	) {
		self.inner.process_block(inputs, outputs)
	}
}

pub fn create(orig_samplerate: f32) -> RemoteControlled<Dsp> {
	let dsp_inner = DspInner {
		threshold: 1.0,
		input_gain: 1.0,
		output_gain: 1.0,
	};

	let inner = Oversample::new(OVERSAMPLE, MAX_BLOCK_SIZE)
		.with_dsp(orig_samplerate, BlockAdapter(dsp_inner));

	RemoteControlled::new(orig_samplerate, 1e3, Dsp { inner })
}
