use crate::audio::channels::pulse_channel1::PulseChannel1;
use crate::audio::channels::pulse_channel2::PulseChannel2;
use crate::mem::Mem;
use rodio::source::Source;
use rodio::OutputStream;
use rodio::OutputStreamHandle;
use rodio::Sink;
use std::time::Duration;

const SOUND_OFFSET: usize = 0x40000000;
const REG_SND3SEL: usize = SOUND_OFFSET + 0x70;
const REG_SND3CNT: usize = SOUND_OFFSET + 0x72;
const REG_SND3FREQ: usize = SOUND_OFFSET + 0x74;
const REG_SND4CNT: usize = SOUND_OFFSET + 0x78;
const REG_SND4FREQ: usize = SOUND_OFFSET + 0x7c;
const REG_SNDDMGCNT: usize = SOUND_OFFSET + 0x80;
const REG_SNDDSCNT: usize = SOUND_OFFSET + 0x82;
const REG_SNDSTAT: usize = SOUND_OFFSET + 0x84;
const REG_SNDBIAS: usize = SOUND_OFFSET + 0x88;

struct Controls {
    enable: bool,
    l_vol: u8,
    r_vol: u8,
    l_c1_en: bool,
    l_c2_en: bool,
    l_c3_en: bool,
    l_c4_en: bool,
    r_c1_en: bool,
    r_c2_en: bool,
    r_c3_en: bool,
    r_c4_en: bool,
    dmg_vol: u8,
}

pub struct APU<'a> {
    handle: &'a OutputStreamHandle,
    c1: Sink,
    c2: Sink,
    c3: Sink,
    c4: Sink,
    da: Sink,
    db: Sink,
    p1: PulseChannel1,
    p2: PulseChannel2,
    controls: Controls,
}

impl<'a> APU<'a> {
    pub fn new(handle: &'a OutputStreamHandle) -> APU<'a> {
        let c1 = Sink::try_new(handle).unwrap();
        let c2 = Sink::try_new(handle).unwrap();
        let c3 = Sink::try_new(handle).unwrap();
        let c4 = Sink::try_new(handle).unwrap();
        let da = Sink::try_new(handle).unwrap();
        let db = Sink::try_new(handle).unwrap();
        let controls = Controls {
            enable: false,
            l_vol: 0,
            r_vol: 0,
            l_c1_en: false,
            l_c2_en: false,
            l_c3_en: false,
            l_c4_en: false,
            r_c1_en: false,
            r_c2_en: false,
            r_c3_en: false,
            r_c4_en: false,
            dmg_vol: 0,
        };
        let p1 = PulseChannel1::new();
        let p2 = PulseChannel2::new();
        APU {
            handle,
            c1,
            c2,
            c3,
            c4,
            da,
            db,
            p1,
            p2,
            controls,
        }
    }

    pub fn step(&mut self, ram: &Mem) {
        self.c1.append(
            self.p1
                .next(ram)
                .take_duration(Duration::from_secs_f64(1.0 / 256.0)),
        );
        self.c2.append(
            self.p2
                .next(ram)
                .take_duration(Duration::from_secs_f64(1.0 / 256.0)),
        );
    }

    fn master_control(&mut self, dmg_cnt: u16, ds_cnt: u16, stat: u16) {
        self.controls.l_vol = ((dmg_cnt << 13) >> 13) as u8;
        self.controls.r_vol = ((dmg_cnt << 9) >> 13) as u8;
        self.controls.l_c1_en = (dmg_cnt << 7) >> 15 == 1;
        self.controls.l_c2_en = (dmg_cnt << 6) >> 15 == 1;
        self.controls.l_c3_en = (dmg_cnt << 5) >> 15 == 1;
        self.controls.l_c4_en = (dmg_cnt << 4) >> 15 == 1;
        self.controls.r_c1_en = (dmg_cnt << 3) >> 15 == 1;
        self.controls.r_c2_en = (dmg_cnt << 2) >> 15 == 1;
        self.controls.r_c3_en = (dmg_cnt << 1) >> 15 == 1;
        self.controls.r_c4_en = dmg_cnt >> 15 == 1;
        self.controls.dmg_vol = ((ds_cnt << 14) >> 14) as u8;
        // self.controls.dsa_conf.vol = (ds_cnt << 13) >> 15 == 1;
        // self.controls.dsb_conf.vol = (ds_cnt << 12) >> 15 == 1;
        // self.controls.dsa_conf.r_en = (ds_cnt << 7) >> 15 == 1;
        // self.controls.dsa_conf.l_en = (ds_cnt << 6) >> 15 == 1;
        // self.controls.dsa_conf.timer = (ds_cnt << 5) >> 15 == 1;
        // self.controls.dsa_conf.fifo_reset = (ds_cnt << 4) >> 15 == 1;
        // self.controls.dsb_conf.r_en = (ds_cnt << 3) >> 15 == 1;
        // self.controls.dsb_conf.l_en = (ds_cnt << 2) >> 15 == 1;
        // self.controls.dsb_conf.timer = (ds_cnt << 1) >> 15 == 1;
        // self.controls.dsb_conf.fifo_reset = ds_cnt >> 15 == 1;
        self.controls.enable = (stat << 8) >> 15 == 1;
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
    // for _ in 0..1000 {
    //     apu.play_pulse_wave1(Duration::from_secs_f32(0.3), 0.2, 0.5, 330, 1, true, 16, 0, 1, true);
    // }
    // std::thread::sleep(Duration::from_millis(1500));
}
