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


struct APU {
    channel1_sinks: Vec<Sink>,
    channel2_sinks: Vec<Sink>,
    channel3_sinks: Vec<Sink>,
    channel4_sinks: Vec<Sink>,
    ds1_sinks: Vec<Sink>,
    ds2_sinks: Vec<Sink>,
}

impl APU {
    fn new() -> APU {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let mut channel1_sinks = Vec::new();
        let mut channel2_sinks = Vec::new();
        let mut channel3_sinks = Vec::new();
        let mut channel4_sinks = Vec::new();
        let mut ds1_sinks = Vec::new();
        let mut ds2_sinks = Vec::new();

        for _i in 0..5 {
            channel1_sinks.push(Sink::try_new(&stream_handle).unwrap());
            channel2_sinks.push(Sink::try_new(&stream_handle).unwrap());
            channel3_sinks.push(Sink::try_new(&stream_handle).unwrap());
            channel4_sinks.push(Sink::try_new(&stream_handle).unwrap());
            ds1_sinks.push(Sink::try_new(&stream_handle).unwrap());
            ds2_sinks.push(Sink::try_new(&stream_handle).unwrap());
        }

        APU {
            channel1_sinks,
            channel2_sinks,
            channel3_sinks,
            channel4_sinks,
            ds1_sinks,
            ds2_sinks,
        }
    }

    fn play_pulse_wave(&self, duration: Duration, amplitude: f32, duty_cycle: f32, frequency: u32) {
        let terms = 5; // constant number of terms of fourier transform
        let mut waves = Self::gen_sine_waves(amplitude, duty_cycle, frequency, terms);

        for i in 0..terms as usize {
            self.channel2_sinks[i].append(waves.remove(0).take_duration(duration));
        }

        self.channel2_sinks[0].sleep_until_end();
    }

    fn gen_sine_waves(amplitude: f32, duty_cycle: f32, frequency: u32, terms: u16) -> Vec<Amplify<SineWave>> {
        let mut waves = Vec::new();

        for n in 1..=terms {
            let freq: u32 = (frequency as f32 * n as f32) as u32;
            let amp: f32 = (2.0 * amplitude / (PI * n as f32)) * (PI * n as f32 * duty_cycle).sin();
            waves.push(SineWave::new(freq).amplify(amp));
        }

        waves
    }

    fn freq_from_rate(rate: u16) -> u32 {
        (1 << 17) / (2048 - rate) as u32
    }
}

pub fn make_a_sound() {
    let apu = APU::new();
    apu.play_pulse_wave(Duration::from_secs_f32(1.2), 1.0, 0.5, 659);
}

