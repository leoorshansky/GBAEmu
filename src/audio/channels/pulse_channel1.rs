use crate::audio::waves::pulse_wave::PulseWave;
use crate::mem::Mem;

const SAMPLE_RATE: u128 = 16_000_000;
const LENGTH_TICK_INTERVAL: u128 = SAMPLE_RATE / 256;
const SWEEP_TICK_INTERVAL: u128 = SAMPLE_RATE / 128;
const ENV_TICK_INTERVAL: u128 = SAMPLE_RATE / 64;

const SOUND_OFFSET: usize = 0x40000000;
const REG_SND1SWEEP: usize = SOUND_OFFSET + 0x60;
const REG_SND1CNT: usize = SOUND_OFFSET + 0x62;
const REG_SND1FREQ: usize = SOUND_OFFSET + 0x64;
const REG_SNDDMGCNT: usize = SOUND_OFFSET + 0x80;
const REG_SNDDSCNT: usize = SOUND_OFFSET + 0x82;

pub struct PulseChannel1 {
    duty: u8,
    length: u8,
    length_en: bool,
    length_count: i128,
    init_vol: i8,
    curr_vol: i8,
    env_inc: bool,
    env_time: u8,
    init_rate: i32,
    curr_rate: i32,
    sweep_shift: u8,
    sweep_time: u8,
    sweep_inc: bool,
    cycles: u128,
    enabled: bool,
}

impl PulseChannel1 {
    pub fn new() -> Self {
        PulseChannel1 {
            duty: 0,
            length: 0,
            length_en: false,
            length_count: 0,
            init_vol: 0,
            curr_vol: 0,
            env_inc: false,
            env_time: 0,
            init_rate: 0,
            curr_rate: 0,
            sweep_shift: 0,
            sweep_time: 0,
            sweep_inc: false,
            cycles: 0,
            enabled: false,
        }
    }

    fn update(&mut self, ram: &Mem) {
        let (dmg_cnt, ds_cnt, cnt, swp, frq) = self.query_mem(ram);

        self.duty = ((cnt << 8) >> 14) as u8;

        let length = ((cnt << 10) >> 10) as u8;
        let length_en = (frq << 1) >> 14 == 1;
        if length != self.length || length_en != self.length_en {
            self.length_count = length as i128;
        }
        self.length = length;
        self.length_en = length_en;
        self.init_vol = ((ds_cnt << 14) >> 14) as i8;
        self.env_inc = (cnt << 4) >> 15 == 1;
        self.env_time = ((cnt << 5) >> 13) as u8;
        self.init_rate = ((frq << 5) >> 5) as i32;
        self.sweep_shift = ((swp << 13) >> 13) as u8;
        self.sweep_time = ((swp << 9) >> 13) as u8;
        self.sweep_inc = (swp << 12) >> 15 == 0;

        let enabled = (dmg_cnt << 7) >> 15 == 1 || (dmg_cnt << 3) >> 15 == 1;
        if enabled != self.enabled {
            self.length_count = length as i128;
            self.enabled = enabled;
        }
    }

    fn query_mem(&self, ram: &Mem) -> (u16, u16, u16, u16, u16) {
        let dmg_cnt = ram.get_halfword(REG_SNDDMGCNT).little_endian();
        let ds_cnt = ram.get_halfword(REG_SNDDSCNT).little_endian();
        let c1_cnt = ram.get_halfword(REG_SND1CNT).little_endian();
        let c1_swp = ram.get_halfword(REG_SND1SWEEP).little_endian();
        let c1_frq = ram.get_halfword(REG_SND1FREQ).little_endian();
        (dmg_cnt, ds_cnt, c1_cnt, c1_swp, c1_frq)
    }

    pub fn next(&mut self, ram: &Mem) -> PulseWave {
        self.update(ram);

        if !self.enabled {
            return PulseWave::new(0, 0.0, 0);
        }

        self.cycles += 1;

        // sound length checks
        if self.length_en && self.cycles % LENGTH_TICK_INTERVAL == 0 {
            self.length_count -= 1;
            if self.length_count <= 0 {
                self.enabled = false;
            }
        }

        // envelope calculations
        if self.env_time != 0 && self.cycles % ENV_TICK_INTERVAL == 0 {
            self.curr_vol += if self.env_inc { 1 } else { -1 };
            if self.curr_vol < 0 {
                self.curr_vol = 0;
            }
            if self.curr_vol > 15 {
                self.curr_vol = 15;
            }
        }

        // sweep calculations
        if self.sweep_shift != 0 && self.sweep_time != 0 && self.cycles % SWEEP_TICK_INTERVAL == 0 {
            let new_rate = self.curr_rate
                + (if self.sweep_inc { 1 } else { -1 })
                    * (self.curr_rate / (1 << self.sweep_shift));
            if new_rate > 0 {
                if new_rate > 2048 {
                    self.enabled = false;
                    return PulseWave::new(0, 0.0, 0);
                }
                self.curr_rate = new_rate;
            }
        }

        PulseWave::new(self.curr_rate as u16, self.duty as f32, self.curr_vol)
    }
}
