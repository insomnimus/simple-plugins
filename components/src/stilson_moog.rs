/** 24dB/oct slope low-pass filter.

Adapted from [synfx-dsp](https://docs.rs/synfx-dsp/latest/synfx_dsp/fn.process_stilson_moog.html).
**/
#[derive(Debug, Clone)]
pub struct LowPassSM {
	israte: f32,
	// State variables.
	b0: f32,
	b1: f32,
	b2: f32,
	b3: f32,
	delay: [f32; 4],
}

impl LowPassSM {
	pub fn new(sample_rate: f32) -> Self {
		Self {
			israte: 1.0 / sample_rate,
			b0: 0.0,
			b1: 0.0,
			b2: 0.0,
			b3: 0.0,
			delay: [0.0; 4],
		}
	}

	#[inline]
	pub fn process(&mut self, sample: f32, cutoff: f32, resonance: f32) -> f32 {
		let cutoff = cutoff * self.israte;

		let p = cutoff * (1.8 - 0.8 * cutoff);
		let k = 2.0 * (cutoff * core::f32::consts::PI * 0.5).sin() - 1.0;

		let t1 = (1.0 - p) * 1.386249;
		let t2 = 12.0 + t1 * t1;

		let resonance = resonance * (t2 + 6.0 * t1) / (t2 - 6.0 * t1);

		let x = sample - resonance * self.b3;

		// Four cascaded one-pole filters (bilinear transform)
		self.b0 = x * p + self.delay[0] * p - k * self.b0;
		self.b1 = self.b0 * p + self.delay[1] * p - k * self.b1;
		self.b2 = self.b1 * p + self.delay[2] * p - k * self.b2;
		self.b3 = self.b2 * p + self.delay[3] * p - k * self.b3;

		// Clipping band-limited sigmoid
		self.b3 -= (self.b3 * self.b3 * self.b3) * 0.166667;

		self.delay[0] = x;
		self.delay[1] = self.b0;
		self.delay[2] = self.b1;
		self.delay[3] = self.b2;

		self.b3
	}

	pub fn reset(&mut self) {
		self.b0 = 0.0;
		self.b1 = 0.0;
		self.b2 = 0.0;
		self.b3 = 0.0;
		self.delay = [0.0; 4];
	}
}

#[cfg(test)]
mod tests {
	#[inline]
	#[allow(clippy::too_many_arguments)]
	fn process_stilson_moog(
		input: f32,
		freq: f32,
		res: f32,
		israte: f32,
		b0: &mut f32,
		b1: &mut f32,
		b2: &mut f32,
		b3: &mut f32,
		delay: &mut [f32; 4],
	) -> f32 {
		let cutoff = 2.0 * freq * israte;

		let p = cutoff * (1.8 - 0.8 * cutoff);
		let k = 2.0 * (cutoff * std::f32::consts::PI * 0.5).sin() - 1.0;

		let t1 = (1.0 - p) * 1.386249;
		let t2 = 12.0 + t1 * t1;

		let res = res * (t2 + 6.0 * t1) / (t2 - 6.0 * t1);

		let x = input - res * *b3;

		// Four cascaded one-pole filters (bilinear transform)
		*b0 = x * p + delay[0] * p - k * *b0;
		*b1 = *b0 * p + delay[1] * p - k * *b1;
		*b2 = *b1 * p + delay[2] * p - k * *b2;
		*b3 = *b2 * p + delay[3] * p - k * *b3;

		// Clipping band-limited sigmoid
		*b3 -= (*b3 * *b3 * *b3) * 0.166667;

		delay[0] = x;
		delay[1] = *b0;
		delay[2] = *b1;
		delay[3] = *b2;

		*b3
	}

	#[test]
	fn test_same() {
		let fq = 10000.0;
		let sr = 48000.0;
		let res = 0.5;

		let mut f = super::LowPassSM::new(sr);
		let [mut b0, mut b1, mut b2, mut b3] = [0.0; 4];
		let mut delay = [0.0; 4];

		for i in 0..100000 {
			let sample = (i % 1000) as f32 / 1000.0 - 0.5;

			let a = f.process(sample, fq * 2.0, res);
			let b = process_stilson_moog(
				sample, fq, res, f.israte, &mut b0, &mut b1, &mut b2, &mut b3, &mut delay,
			);

			assert_eq!(a, b);
			assert_eq!(f.delay, delay);
		}
	}
}
