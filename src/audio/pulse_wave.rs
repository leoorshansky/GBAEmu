use std::time::Duration;
use rodio::Source;

pub struct PulseWave {
    freq: f32,
    duty: f32,
    num_sample: usize,
}

impl PulseWave {
    pub fn new(freq: f32, duty: f32) -> PulseWave {
        PulseWave {
            freq: freq,
            duty: duty,
            num_sample: 0,
        }
    }
}

impl Iterator for PulseWave {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        self.num_sample = (self.num_sample + 1) % (48000.0 / self.freq) as usize;

        let val = if self.num_sample <= (self.duty * 48000.0 / self.freq) as usize { 1.0 } else { 0.0 };
        Some(val)
    }
}

impl Source for PulseWave {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        48000
    }

    fn total_duration(&self) -> Option<Duration> {
        None
    }
}