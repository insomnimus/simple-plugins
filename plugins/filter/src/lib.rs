use std::sync::Arc;

use filters::simper::{
	Coefficients,
	Simper,
	BUTTERWORTH_Q,
};
use nih_plug::prelude::*;

nih_export_clap!(FilterPlugin);

const HPF_OFF_FQ: f32 = 20.0;
const LPF_OFF_FQ: f32 = 20000.0;

fn fq_param(name: &str, default: f32) -> FloatParam {
	FloatParam::new(
		name,
		default,
		FloatRange::Skewed {
			min: 19.9,
			max: 20000.1,
			factor: FloatRange::skew_factor(-2.0),
		},
	)
	.with_string_to_value(formatters::s2v_f32_hz_then_khz())
	// .with_value_to_string(formatters::v2s_f32_hz_then_khz(2))
}

fn q_param(name: &str) -> FloatParam {
	FloatParam::new(
		name,
		BUTTERWORTH_Q as f32,
		FloatRange::Skewed {
			min: 0.1,
			max: 4.8,
			factor: FloatRange::skew_factor(-1.0),
		},
	)
	.with_value_to_string(formatters::v2s_f32_rounded(2))
}

fn format_fq(fq: f32) -> String {
	if fq < 1000.0 {
		format!("{fq:.2} hz")
	} else {
		format!("{:.2} khz", fq / 1000.0)
	}
}

#[derive(Debug, Params)]
struct FilterParam {
	#[id = "fq"]
	fq: FloatParam,
	#[id = "q"]
	q: FloatParam,
}

#[derive(Debug, Params)]
struct FilterParams {
	#[nested(id_prefix = "hp", group = "High-Pass Filter")]
	hp: FilterParam,
	#[nested(id_prefix = "lp", group = "Low-Pass Filter")]
	lp: FilterParam,
}

impl Default for FilterParams {
	fn default() -> Self {
		Self {
			hp: FilterParam {
				q: q_param("HPF Q"),
				fq: fq_param("HPF Frequency", HPF_OFF_FQ).with_value_to_string(Arc::new(|fq| {
					if fq <= HPF_OFF_FQ {
						"Off".to_string()
					} else {
						format_fq(fq)
					}
				})),
			},

			lp: FilterParam {
				q: q_param("LPF Q"),
				fq: fq_param("LPF Frequency", LPF_OFF_FQ).with_value_to_string(Arc::new(|fq| {
					if fq >= LPF_OFF_FQ {
						"Off".to_string()
					} else {
						format_fq(fq)
					}
				})),
			},
		}
	}
}

#[derive(Default)]
struct FilterPlugin {
	params: Arc<FilterParams>,
	sr: f64,
	hps: Vec<Simper>,
	lps: Vec<Simper>,
}

impl FilterPlugin {
	fn update_hp(&mut self) {
		let hc = Coefficients::high_pass(
			self.sr,
			self.params.hp.fq.value() as f64,
			self.params.hp.q.value() as f64,
		);

		for f in &mut self.hps {
			f.set_parameters(hc.clone())
		}
	}

	fn update_lp(&mut self) {
		let lc = Coefficients::low_pass(
			self.sr,
			self.params.lp.fq.value() as f64,
			self.params.lp.q.value() as f64,
		);

		for f in &mut self.lps {
			f.set_parameters(lc.clone())
		}
	}

	fn reset_hp(&mut self) {
		for f in &mut self.hps {
			f.reset();
		}
	}

	fn reset_lp(&mut self) {
		for f in &mut self.lps {
			f.reset();
		}
	}
}

impl ClapPlugin for FilterPlugin {
	const CLAP_DESCRIPTION: Option<&'static str> = Some("Simple high and low-pass filters");
	const CLAP_FEATURES: &'static [ClapFeature] =
		&[ClapFeature::Filter, ClapFeature::Mono, ClapFeature::Stereo];
	const CLAP_ID: &'static str = "insomnia.simple-filter";
	const CLAP_MANUAL_URL: Option<&'static str> = None;
	const CLAP_SUPPORT_URL: Option<&'static str> = Some(env!("CARGO_PKG_HOMEPAGE"));
}

impl Plugin for FilterPlugin {
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
	const NAME: &'static str = "Simple Filter";
	const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
	const VENDOR: &'static str = "Insomnia";
	const VERSION: &'static str = env!("CARGO_PKG_VERSION");

	fn params(&self) -> Arc<dyn Params> {
		self.params.clone()
	}

	fn reset(&mut self) {
		self.reset_hp();
		self.reset_lp();
	}

	fn initialize(
		&mut self,
		layout: &AudioIOLayout,
		buffer_config: &BufferConfig,
		_context: &mut impl InitContext<Self>,
	) -> bool {
		self.sr = buffer_config.sample_rate as _;
		self.hps.clear();
		self.lps.clear();

		let lpf = Simper::low_pass(self.sr, 20e3, BUTTERWORTH_Q);
		let hpf = Simper::high_pass(self.sr, 20.0, BUTTERWORTH_Q);

		for _ in 0..layout.main_input_channels.map_or(0, |n| n.get()) {
			self.hps.push(hpf.clone());
			self.lps.push(lpf.clone());
		}

		true
	}

	fn process(
		&mut self,
		buffer: &mut Buffer,
		_aux: &mut AuxiliaryBuffers,
		_context: &mut impl ProcessContext<Self>,
	) -> ProcessStatus {
		let hp = self.params.hp.fq.value();
		let lp = self.params.lp.fq.value();

		if hp <= HPF_OFF_FQ && lp >= LPF_OFF_FQ {
			// Filters are off
			return ProcessStatus::Normal;
		}

		if lp >= LPF_OFF_FQ {
			// LP is off
			self.update_hp();
			self.reset_lp();

			for (channel, hpf) in buffer.as_slice().iter_mut().zip(&mut self.hps) {
				for sample in channel.iter_mut() {
					*sample = hpf.process(*sample as _) as _;
				}
			}

			return ProcessStatus::Normal;
		}

		if hp <= HPF_OFF_FQ {
			// HP is off
			self.update_lp();
			self.reset_hp();

			for (channel, lpf) in buffer.as_slice().iter_mut().zip(&mut self.lps) {
				for sample in channel.iter_mut() {
					*sample = lpf.process(*sample as _) as _;
				}
			}

			return ProcessStatus::Normal;
		}

		// Both filters are active.
		self.update_hp();
		self.update_lp();

		for (channel, (hpf, lpf)) in buffer
			.as_slice()
			.iter_mut()
			.zip(self.hps.iter_mut().zip(&mut self.lps))
		{
			for sample in channel.iter_mut() {
				*sample = lpf.process(hpf.process(*sample as _)) as _;
			}
		}

		ProcessStatus::Normal
	}
}
