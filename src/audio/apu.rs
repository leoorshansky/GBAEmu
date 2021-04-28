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


// todo: sweep stuff, envelope stuff, channel 3 sample reading, channel 4 noise generation
// todo: direct sound stuff
fn play_pulse_wave(duration: Duration, amplitude: f32, duty_cycle: f32, frequency: u32) {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    sink.append(PulseWave::new(frequency as f32, duty_cycle).amplify(amplitude).take_duration(duration));

    // sink.detach();
    sink.sleep_until_end();
}

fn freq_from_rate(rate: u16) -> u32 {
    (1 << 17) / (2048 - rate) as u32
}

pub fn make_a_sound() {
    // play_pulse_wave(Duration::from_secs_f32(0.5), 1.0, 0.5, 440);
    // play_pulse_wave(Duration::from_secs_f32(0.5), 0.5, 0.5, 440);
    // play_pulse_wave(Duration::from_secs_f32(0.5), 0.25, 0.5, 440);
    play_pulse_wave(Duration::from_secs_f32(0.3), 1.0, 0.5, 330);
    play_pulse_wave(Duration::from_secs_f32(0.2), 1.0, 0.5, 246);
    play_pulse_wave(Duration::from_secs_f32(0.2), 1.0, 0.5, 261);
    play_pulse_wave(Duration::from_secs_f32(0.3), 1.0, 0.5, 294);
    play_pulse_wave(Duration::from_secs_f32(0.2), 1.0, 0.5, 261);
    play_pulse_wave(Duration::from_secs_f32(0.2), 1.0, 0.5, 246);
    play_pulse_wave(Duration::from_secs_f32(0.3), 1.0, 0.5, 220);
}

