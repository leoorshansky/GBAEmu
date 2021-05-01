use rodio::Sample;
use rodio::OutputStreamHandle;
use rodio::source::TakeDuration;
use crate::audio::pulse_wave::PulseWave;
use rodio::source::Amplify;
use std::time::Duration;
use rodio::{OutputStream, Sink};
use rodio::source::{SineWave, Source};
use std::f32::consts::PI;

const SOUND_OFFSET: u64 = 0x40000000;
const REG_SND1SWEEP: u64 = SOUND_OFFSET + 0x60;
const REG_SND1CTR: u64 = SOUND_OFFSET + 0x62;
const REG_SND1FREQ: u64 = SOUND_OFFSET + 0x64;
const REG_SND2CTR: u64 = SOUND_OFFSET + 0x68;
const REG_SND2FREQ: u64 = SOUND_OFFSET + 0x6c;
const REG_SND3SEL: u64 = SOUND_OFFSET + 0x70;
const REG_SND3CNT: u64 = SOUND_OFFSET + 0x72;
const REG_SND3FREQ: u64 = SOUND_OFFSET + 0x74;
const REG_SND4CNT: u64 = SOUND_OFFSET + 0x78;
const REG_SND4FREQ: u64 = SOUND_OFFSET + 0x7c;
const REG_SNDDMGCNT: u64 = SOUND_OFFSET + 0x80;
const REG_SNDDSCNT: u64 = SOUND_OFFSET + 0x82;
const REG_SNDSTAT: u64 = SOUND_OFFSET + 0x84;
const REG_SNDBIAS: u64 = SOUND_OFFSET + 0x88;


pub struct APU<'a> {
    handle: &'a OutputStreamHandle,
    c1: Sink,
    c2: Sink,
    c3: Sink,
    c4: Sink,
    d1: Sink,
    d2: Sink,
}

impl<'a> APU<'a> {
    pub fn new(handle: &'a OutputStreamHandle) -> APU<'a> {
        let c1 = Sink::try_new(handle).unwrap();
        let c2 = Sink::try_new(handle).unwrap();
        let c3 = Sink::try_new(handle).unwrap();
        let c4 = Sink::try_new(handle).unwrap();
        let d1 = Sink::try_new(handle).unwrap();
        let d2 = Sink::try_new(handle).unwrap();
        APU { handle, c1, c2, c3, c4, d1, d2 }
    }

    // todo: channel 3 sample reading, channel 4 noise generation
    // todo: direct sound stuff
    pub fn play_pulse_wave(&self, duration: Duration, amplitude: f32, duty_cycle: f32, 
        frequency: u32, env_step: u8, env_dir: bool, env_init: u8, swp_shift: u8, 
        swp_time: u8, swp_dir: bool) 
    {
        let wave = PulseWave::new(frequency as f32, duty_cycle, env_step, env_dir, env_init, swp_shift, swp_time, swp_dir);
        let wave = wave.take_duration(duration).amplify(amplitude);

        self.c1.append(wave);
    }

    pub fn freq_from_rate(rate: u16) -> f32 {
        (1 << 17) as f32 / (2048 - rate) as f32
    }

    pub fn rate_from_freq(freq: f32) -> u16 {
        (2048.0 - (1 << 17) as f32 / freq) as u16
    }
}



pub fn make_a_sound() {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let apu = APU::new(&stream_handle);
    apu.play_pulse_wave(Duration::from_secs_f32(0.3), 0.2, 0.5, 330, 1, true, 16, 0, 1, true);
    // apu.play_pulse_wave(Duration::from_secs_f32(0.15), 0.2, 0.5, 246, 1, true, 16, 0, 1, true);
    // apu.play_pulse_wave(Duration::from_secs_f32(0.15), 0.2, 0.5, 261, 1, true, 16, 0, 1, true);
    // apu.play_pulse_wave(Duration::from_secs_f32(0.3), 0.2, 0.5, 294, 1, true, 16, 0, 1, true);
    // apu.play_pulse_wave(Duration::from_secs_f32(0.15), 0.2, 0.5, 261, 1, true, 16, 0, 1, true);
    // apu.play_pulse_wave(Duration::from_secs_f32(0.15), 0.2, 0.5, 246, 1, true, 16, 0, 1, true);
    // apu.play_pulse_wave(Duration::from_secs_f32(0.3), 0.2, 0.5, 220, 1, true, 16, 0, 1, true);
    std::thread::sleep(Duration::from_millis(1500));
}

