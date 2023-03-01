use once_cell::sync::Lazy;

pub static TICKS_PER_FRAME: usize = 29830;
pub static TICKS_PER_SECOND: usize = TICKS_PER_FRAME * 60;
pub static SAMPLES_PER_SEC: i32 = 44100;
pub static SAMPLE_PER_TICK: f64 = SAMPLES_PER_SEC as f64 / TICKS_PER_SECOND as f64;

pub static LENGTH_COUNTER_LUT: [u8; 32] = [
    0x0A, 0xFE, 0x14, 0x02, 0x28, 0x04, 0x50, 0x06, 0xA0, 0x08, 0x3C, 0x0A, 0x0E, 0x0C, 0x1A, 0x0E,
    0x0C, 0x10, 0x18, 0x12, 0x30, 0x14, 0x60, 0x16, 0xC0, 0x18, 0x48, 0x1A, 0x10, 0x1C, 0x20, 0x1E,
];

pub static TICK_SAMPLE_TIMING: Lazy<Vec<bool>> = Lazy::new(|| {
    let mut v = vec![false; TICKS_PER_FRAME];
    let mut sum = 0.0f64;
    for i in 0..TICKS_PER_FRAME {
        sum += SAMPLE_PER_TICK;
        if sum > 1.0 {
            v[i] = true;
            sum -= 1.0;
        }
    }
    v
});
