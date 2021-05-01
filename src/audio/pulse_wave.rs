use crate::audio::apu::APU;
use std::time::Duration;
use rodio::Source;

#[derive(Copy, Clone, Debug)]
pub struct PulseWave {
    total_cycles: u64,
    rate: u16,
    duty: f32,
    num_sample: usize,
    env_step_time: u8,
    env_dir: bool,
    env_val: u8,
    swp_shift: u8,
    swp_time: u8,
    swp_dir: bool,
}

impl PulseWave {
    pub fn new(freq: f32, duty: f32, env_step_time: u8, env_dir: bool, env_init: u8, swp_shift: u8, swp_time: u8, swp_dir: bool) -> PulseWave {
        PulseWave {
            total_cycles: 0,
            rate: APU::rate_from_freq(freq),
            duty: duty,
            num_sample: 0,
            env_step_time: env_step_time,
            env_dir: env_dir,
            env_val: env_init,
            swp_shift: swp_shift,
            swp_time: swp_time,
            swp_dir: swp_dir,
        }
    }
}

impl Iterator for PulseWave {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let freq = APU::freq_from_rate(self.rate);
        self.total_cycles = self.total_cycles.wrapping_add(1);
        self.num_sample = (self.num_sample + 1) % (48000.0 / freq) as usize;

        // sweep calculations (374.4 = 7.8 ms * 48kHz)
        let should_sweep = self.total_cycles % (self.swp_time as f64 * 374.4) as u64 == 0 && (self.swp_shift > 0 && self.swp_time > 0);
        if should_sweep {
            let new_rate = self.rate.wrapping_add((if self.swp_dir {65535} else {1} as u16).wrapping_mul(self.rate / (1 << self.swp_shift)));
            if new_rate > 0 {
                if new_rate > 2048 {
                    return None;
                }
                self.rate = new_rate;
                // println!("swept to {}", self.rate);
            }
        }

        // envelope calculations (750 = 15.625 ms * 48kHz)
        let should_bump = self.total_cycles % (self.env_step_time as u64 * 750u64) == 0 && self.total_cycles > 0;
        if should_bump {
            self.env_val = self.env_val.wrapping_add(if self.env_dir {1} else {255});
            self.env_val = std::cmp::min(self.env_val, 16);
            // println!("bump to {}", self.env_val);
        }
        let volume = self.env_val as f32 / 16.0;
        if should_bump {
            // println!("{}", volume);
        }

        // final sound calculations
        let mut val: f32 = if self.num_sample <= (self.duty * 48000.0 / freq) as usize { 1.0 } else { 0.0 };
        val *= volume;
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
