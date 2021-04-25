use std::time::Duration;
use rodio::{OutputStream, Sink};
use rodio::source::{SineWave, Source};

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


pub fn make_a_sound() {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    // Add a dummy source of the sake of the example.
    let source = SineWave::new(440).take_duration(Duration::from_secs_f32(3.0)).amplify(0.20);
    sink.append(source);

    // The sound plays in a separate thread. This call will block the current thread until the sink
    // has finished playing all its queued sounds.
    sink.sleep_until_end();
}
