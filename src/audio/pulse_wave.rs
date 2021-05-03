use crate::audio::apu::APU;
use rodio::Source;
use std::time::Duration;

pub struct PulseWave {
    rate: u16,
    duty: f32,
    volume: i8,
    cycles: u32,
}

impl PulseWave {
    pub fn new(rate: u16, duty: f32, volume: i8) -> PulseWave {
        PulseWave {
            rate: rate,
            duty: duty,
            volume: volume,
            cycles: 0,
        }
    }
}

impl Iterator for PulseWave {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        self.cycles += 1;
        let freq = APU::freq_from_rate(self.rate);
        let val: f32 = if self.cycles <= (self.duty * 48000.0 / freq) as u32 {
            (self.volume + 1) as f32 / 16.0
        } else {
            0.0
        };
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
