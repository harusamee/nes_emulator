use ringbuf::HeapRb;
use sdl2::{audio::AudioSpecDesired, Sdl};
use std::{ptr::null, time::Duration};

mod constants;
use constants::*;

mod apu;
pub use apu::{Apu, ApuSDL};

mod dmc;
mod noise;
mod pulse_wave;
mod triangle_wave;
mod wave;
mod wave_trait;

pub fn init_apu(sdl_context: &Sdl) -> Apu {
    let audio_subsystem = sdl_context.audio().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(SAMPLES_PER_SEC),
        channels: Some(1),   // mono
        samples: Some(1024), // default sample size
    };

    let rb = HeapRb::<f32>::new((SAMPLES_PER_SEC >> 3) as usize);
    let (prod, cons) = rb.split();

    let device = audio_subsystem
        .open_playback(None, &desired_spec, |_| ApuSDL::new(cons))
        .unwrap();
    Apu::new(prod, Some(device))
}

pub fn init_null_apu() -> Apu {
    let rb = HeapRb::<f32>::new(10);
    let (prod, _) = rb.split();
    Apu::new(prod, None)
}

fn test_apu() {
    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(SAMPLES_PER_SEC),
        channels: Some(1),   // mono
        samples: Some(1024), // default sample size
    };

    let rb = HeapRb::<f32>::new(SAMPLES_PER_SEC as usize);
    let (prod, cons) = rb.split();

    let w = ApuSDL::new(cons);
    let device = audio_subsystem
        .open_playback(None, &desired_spec, |_| w)
        .unwrap();
    let mut apu = Apu::new(prod, Some(device));

    apu.write_register(0x4015, 0x0f);

    apu.write_register(0x4000, 0xdf);
    apu.write_register(0x4002, 0xd5);
    apu.write_register(0x4003, 0x50);

    apu.write_register(0x4004, 0xdf);
    apu.write_register(0x4006, 0xa9);
    apu.write_register(0x4007, 0x50);

    apu.write_register(0x4008, 0x78);
    apu.write_register(0x400a, 0x8e);
    apu.write_register(0x400b, 0x08);

    apu.tick_usize(TICKS_PER_SECOND);

    // Start playback
    apu.audio_device.as_ref().unwrap().resume();

    std::thread::sleep(Duration::from_millis(1000));

    apu.write_register(0x400c, 0xdf);
    apu.write_register(0x400e, 0x8f);
    apu.write_register(0x400f, 0x08);
    apu.tick_usize(TICKS_PER_SECOND);
    std::thread::sleep(Duration::from_millis(1000));

    apu.write_register(0x4000, 0x0f);
    apu.write_register(0x4002, 0xd5);
    apu.write_register(0x4003, 0x08);
    apu.tick_usize(TICKS_PER_SECOND);
    std::thread::sleep(Duration::from_millis(1000));

    apu.write_register(0x4004, 0xdf);
    apu.write_register(0x4005, 0xff);
    apu.write_register(0x4006, 0xa9);
    apu.write_register(0x4007, 0x08);
    apu.tick_usize(TICKS_PER_SECOND);
    std::thread::sleep(Duration::from_millis(2000));
}
