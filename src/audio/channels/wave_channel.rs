use crate::audio::apu::APU;
use crate::audio::waves::sample_wave::SampleWave;
use crate::mem::Mem;
use rodio::source::Source;
use rodio::source::TakeDuration;
use std::time::Duration;

const SAMPLE_RATE: u128 = 48000;
const LENGTH_TICK_INTERVAL: u128 = SAMPLE_RATE / 256;

const SOUND_OFFSET: usize = 0x40000000;
const REG_SND3SEL: usize = SOUND_OFFSET + 0x70;
const REG_SND3CNT: usize = SOUND_OFFSET + 0x72;
const REG_SND3FREQ: usize = SOUND_OFFSET + 0x74;
const REG_SNDDMGCNT: usize = SOUND_OFFSET + 0x80;
const REG_SNDDSCNT: usize = SOUND_OFFSET + 0x82;
const REG_SND3BANK0: usize = SOUND_OFFSET + 0x90;
const REG_SND3BANK1: usize = SOUND_OFFSET + 0x98;

pub struct WaveChannel {
    cycles: u128,
    enabled: bool,
    mode: bool,
    bank_sel: u8,
    length: u8,
    length_en: bool,
    length_count: i128,
    vol: u8,
    rate: u16,
    sample_num: u8,
}

impl WaveChannel {
    pub fn new() -> Self {
        WaveChannel {
            cycles: 0,
            enabled: false,
            mode: false,
            bank_sel: 0,
            length: 0,
            length_en: false,
            length_count: 0,
            vol: 0,
            rate: 0,
            sample_num: 0,
        }
    }

    fn update(&mut self, ram: &Mem) {
        let (dmg_cnt, ds_cnt, sel, cnt, frq) = self.query_mem(ram);

        self.mode = (sel << 10) >> 15 == 1;
        self.bank_sel = ((sel << 9) >> 15) as u8;
        self.enabled = (sel << 8) >> 15 == 1;
        self.length = ((cnt << 8) >> 8) as u8;
        self.vol = (cnt >> 13) as u8;
        self.rate = (frq << 5) >> 6;
        self.length_en = (frq << 1) >> 15 == 1;
    }

    fn query_mem(&self, ram: &Mem) -> (u16, u16, u16, u16, u16) {
        let dmg_cnt = ram.get_halfword(REG_SNDDMGCNT).little_endian();
        let ds_cnt = ram.get_halfword(REG_SNDDSCNT).little_endian();
        let c3_sel = ram.get_halfword(REG_SND3SEL).little_endian();
        let c3_cnt = ram.get_halfword(REG_SND3CNT).little_endian();
        let c3_frq = ram.get_halfword(REG_SND3FREQ).little_endian();
        (dmg_cnt, ds_cnt, c3_sel, c3_cnt, c3_frq)
    }

    pub fn next(&mut self, ram: &Mem) -> TakeDuration<SampleWave> {
        self.update(ram);

        if !self.enabled {
            return SampleWave::new(0, 0).take_duration(Duration::from_millis(0));
        }

        self.cycles += 1;

        if self.ready() {
            self.sample_num += 1;
            if (self.mode && self.sample_num >= 64) || (!self.mode && self.sample_num >= 32) {
                self.sample_num = 0;
            }
        }

        // sound length checks
        if self.length_en && self.cycles % LENGTH_TICK_INTERVAL == 0 {
            self.length_count -= 1; // todo: have to find out what to subtract; should be a function of how many times this function is called per second
            if self.length_count <= 0 {
                self.enabled = false;
            }
        }

        let sample = self.get_next_sample(ram);
        SampleWave::new(sample, self.vol).take_duration(Duration::from_secs_f32(
            1.0 / APU::freq_from_rate(self.rate),
        ))
    }

    pub fn ready(&self) -> bool {
        self.cycles % APU::freq_from_rate(self.rate) as u128 == 0
    }

    fn get_next_sample(&self, ram: &Mem) -> u8 {
        let mut double_samples: Vec<u8> = Vec::new();
        if self.mode {
            for i in 0..16 {
                double_samples.push(ram.get_byte(REG_SND3BANK0 + i));
            }
        } else {
            let bank = if self.bank_sel == 0 {
                REG_SND3BANK0
            } else {
                REG_SND3BANK1
            };
            for i in 0..8 {
                double_samples.push(ram.get_byte(bank + i));
            }
        }
        let block = self.sample_num / 2;
        let offset = self.sample_num % 2;
        let sample = double_samples[block as usize];
        if offset == 1 {
            (sample << 4) >> 4
        } else {
            sample >> 4
        }
    }
}
