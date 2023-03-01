use once_cell::sync::Lazy;
use ringbuf::{Consumer, Producer, SharedRb};
use sdl2::audio::{AudioCallback, AudioDevice};
use std::{mem::MaybeUninit, sync::Arc};

use crate::constants::*;
use crate::dmc::Dmc;
use crate::noise::Noise;
use crate::pulse_wave::PulseWave;
use crate::triangle_wave::TriangleWave;
use crate::wave_trait::WaveTrait;

pub struct Apu {
    triangle: TriangleWave,
    pulse1: PulseWave,
    pulse2: PulseWave,
    noise: Noise,
    dmc: Dmc,
    tick: usize,
    ringbuf_prod: Producer<f32, Arc<SharedRb<f32, Vec<MaybeUninit<f32>>>>>,
    mode: bool,
    inhib_intr: bool,
    //
    pub audio_device: Option<AudioDevice<ApuSDL>>,
}

impl Apu {
    pub fn new(
        prod: Producer<f32, Arc<SharedRb<f32, Vec<MaybeUninit<f32>>>>>,
        audio_device: Option<AudioDevice<ApuSDL>>,
    ) -> Self {
        Apu {
            triangle: TriangleWave::new(),
            pulse1: PulseWave::new(1),
            pulse2: PulseWave::new(2),
            noise: Noise::new(),
            dmc: Dmc::new(),
            tick: 0,
            ringbuf_prod: prod,
            mode: false,
            inhib_intr: true,
            audio_device,
        }
    }

    pub fn write_register(&mut self, address: u16, data: u8) {
        match address {
            // pulse 1
            0x4000 => self.pulse1.write_0(data),
            0x4001 => self.pulse1.write_1(data),
            0x4002 => self.pulse1.set_reg_freq_lo(data),
            0x4003 => self.pulse1.write_3(data),
            // pulse 2
            0x4004 => self.pulse2.write_0(data),
            0x4005 => self.pulse2.write_1(data),
            0x4006 => self.pulse2.set_reg_freq_lo(data),
            0x4007 => self.pulse2.write_3(data),
            // triangle
            0x4008 => {
                let linear_counter = data & 0b0111_1111;
                self.triangle.set_linear_counter(linear_counter);
                let halt = data & 0b1000_0000 > 0;
                self.triangle.set_halt(halt);
            }
            0x400a => self.triangle.set_reg_freq_lo(data),
            0x400b => {
                self.triangle.set_reg_freq_hi(data);
                self.triangle.set_length_counter(data);
                self.triangle.set_tone_freq();
            }
            // noise
            0x400c => self.noise.write_0(data),
            0x400e => self.noise.write_2(data),
            0x400f => self.noise.write_3(data),
            // dmc
            0x4010 => self.dmc.write_0(data),
            0x4011 => self.dmc.write_1(data),
            0x4012 => self.dmc.write_2(data),
            0x4013 => self.dmc.write_3(data),
            // misc
            0x4015 => {
                self.pulse1.set_enable(data & 0b1 > 0);
                self.pulse2.set_enable(data & 0b10 > 0);
                self.triangle.set_enable(data & 0b100 > 0);
                self.noise.set_enable(data & 0b1000 > 0);
                self.dmc.set_enable(data & 0b1_0000 > 0);
            }
            _ => {}
        };
    }

    // https://www.nesdev.org/wiki/APU_Mixer
    fn get_pulse_output(&mut self) -> f32 {
        static PULSE_LUT: Lazy<[f32; 32]> = Lazy::new(|| {
            let mut lut = [0.0f32; 32];
            for n in 1..lut.len() {
                lut[n] = 95.52 / (8128.0 / n as f32 + 100.0);
            }
            lut
        });
        let pulse1 = self.pulse1.get_output();
        let pulse2 = self.pulse2.get_output();

        PULSE_LUT[(pulse1 + pulse2) as usize]
    }

    fn get_tnd_output(&mut self) -> f32 {
        static TND_LUT: Lazy<[f32; 203]> = Lazy::new(|| {
            let mut lut = [0.0f32; 203];
            for n in 1..lut.len() {
                lut[n] = 163.67 / (24329.0 / n as f32 + 100.0);
            }
            lut
        });
        let triangle = self.triangle.get_output();
        let noise = self.noise.get_output();
        let dmc = self.dmc.get_output();

        TND_LUT[triangle as usize * 3 + noise as usize * 2 + dmc as usize]
    }

    fn tick_single(&mut self) {
        if TICK_SAMPLE_TIMING[self.tick] {
            let pulse = self.get_pulse_output();
            let tnd = self.get_tnd_output();
            match self.ringbuf_prod.push(pulse + tnd) {
                Ok(_) => {}
                Err(_) => {
                    //println!("could not push rb");
                }
            }
        }

        self.pulse1.tick(self.tick);
        self.pulse2.tick(self.tick);
        self.triangle.tick(self.tick);
        self.noise.tick(self.tick);

        self.tick += 1;
        if !self.mode {

            static HALF_FRAME: [usize; 2] = [14913, 29829];
            if HALF_FRAME.contains(&self.tick) {
                self.pulse1.on_length_count();
                self.pulse2.on_length_count();
                self.pulse1.process_sweep();
                self.pulse2.process_sweep();
                self.triangle.on_length_count();
                self.noise.on_length_count();
            }

            static QUARTER_FRAME: [usize; 4] = [7457, 14913, 22371, 29829];
            if QUARTER_FRAME.contains(&self.tick) {
                self.pulse1.process_envelope();
                self.pulse2.process_envelope();
                self.triangle.on_linear_count();
                self.noise.process_envelope();
            }
        } else {
            todo!();
        }

        if self.tick == TICK_SAMPLE_TIMING.len() {
            self.tick = 0;
        }
    }

    pub fn tick_usize(&mut self, tick: usize) {
        for _ in 0..tick {
            self.tick_single();
        }
    }

    pub fn tick(&mut self, tick: u8) {
        self.tick_usize(tick as usize);
    }
}

pub struct ApuSDL {
    pub ringbuf_cons: Consumer<f32, Arc<SharedRb<f32, Vec<MaybeUninit<f32>>>>>,
    callback_count: usize,
}

impl AudioCallback for ApuSDL {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        if self.ringbuf_cons.len() > 0 {
            self.ringbuf_cons.pop_slice(out);
            self.callback_count += 1;
            if self.callback_count % 11 == 0 {
                println!("buffer size {}", self.ringbuf_cons.free_len());
            }
        } else {
            println!(
                "not enough sample: {} < {}",
                self.ringbuf_cons.len(),
                out.len()
            );
        }
    }
}

impl ApuSDL {
    pub fn new(cons: Consumer<f32, Arc<SharedRb<f32, Vec<MaybeUninit<f32>>>>>) -> Self {
        ApuSDL {
            ringbuf_cons: cons,
            callback_count: 0,
        }
    }
}
