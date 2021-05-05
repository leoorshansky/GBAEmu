use rodio::Source;
use std::time::Duration;

pub struct SampleWave {
    cycles: u32,
    volume: u8,
    sample: u8,
}

impl SampleWave {
    pub fn new(sample: u8, volume: u8) -> Self {
        SampleWave {
            cycles: 0,
            volume,
            sample,
        }
    }

    fn get_vol(&self) -> f32 {
        match self.volume {
            0 => 0.0,
            1 => 1.0,
            8 => 0.75,
            2 => 0.5,
            3 => 0.25,
            _ => 0.0,
        }
    }
}

impl Iterator for SampleWave {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        self.cycles += 1;
        Some(self.get_vol() * self.sample as f32 / 16.0)
    }
}

impl Source for SampleWave {
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
