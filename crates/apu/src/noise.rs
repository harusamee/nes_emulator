use crate::wave::Wave;
use crate::wave_trait::AsWave;
use crate::wave_trait::WaveTrait;

static NOISE_PERIOD_LUT: [u16; 16] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
];

pub struct Noise {
    base: Wave,
    // 0x400c
    envelope_counter_halt: bool,
    contant_volume: bool,
    volume: u8,
    // 0x400e
    mode: bool,
    period: u8,
    // internal flag/counter
    envelope_start_flag: bool,
    decay_counter: u8,
    envelope_counter: u8,
    period_counter: u16,
    random_value: u16,
}

mod private {
    use crate::wave_trait::AsWave;
    impl AsWave for super::Noise {
        fn as_wave(&self) -> &super::Wave {
            &self.base
        }
        fn as_mut_wave(&mut self) -> &mut super::Wave {
            &mut self.base
        }
    }
}

impl WaveTrait for Noise {
    fn on_length_count(&mut self) {
        if !self.envelope_counter_halt && self.as_wave().length_counter > 0 {
            self.as_mut_wave().length_counter -= 1;
            if self.as_wave().length_counter == 0 {
                // println!("noise: length count zero");
            }
        }
    }

    fn on_frame(&mut self) {
        todo!()
    }

    fn get_output(&mut self) -> u8 {
        if self.as_wave().enable && self.as_wave().length_counter > 0 {
            let volume = if self.contant_volume {
                self.volume
            } else {
                self.decay_counter
            };
    
            if self.random_value & 0x4000 == 0 {
                return volume;
            }
        }
        0
    }

    fn tick(&mut self, tick: usize) {
        self.update_random_value(tick);
    }
}

impl Noise {
    pub fn new() -> Self {
        Noise {
            base: Wave::new("Noise"),
            envelope_counter_halt: false,
            contant_volume: false,
            volume: 0,
            envelope_start_flag: false,
            decay_counter: 1,
            envelope_counter: 0,
            mode: false,
            period: 0,
            period_counter: 1,
            random_value: 1,
        }
    }

    pub fn process_envelope(&mut self) {
        if self.envelope_start_flag {
            self.envelope_start_flag = false;
            self.decay_counter = 15;
            self.envelope_counter = self.volume;
        } else {
            if self.envelope_counter == 0 {
                self.envelope_counter = self.volume;
                if self.decay_counter > 0 {
                    self.decay_counter -= 1;
                } else if self.envelope_counter_halt {
                    self.decay_counter = 15;
                }
            } else {
                self.envelope_counter -= 1;
            }
        }
    }

    pub fn write_0(&mut self, data: u8) {
        let halt = data & 0b0010_0000 > 0;
        let const_vol = data & 0b0001_0000 > 0;
        let envelope = data & 0b0000_1111;

        self.envelope_counter_halt = halt;
        self.contant_volume = const_vol;
        self.volume = envelope;
        if self.envelope_counter_halt {
            println!("{}: counter halted", &self.base.name);
        }
    }

    pub fn write_2(&mut self, data: u8) {
        let mode = data & 0b1000_0000 > 0;
        let period = data & 0b0000_1111;
        self.mode = mode;
        self.period = period;
    }

    pub fn write_3(&mut self, data: u8) {
        self.set_length_counter(data);
        // println!(
        //     "{}: length count {}",
        //     &self.base.name,
        //     self.base.length_counter as f32 / 120.0
        // );

        self.envelope_start_flag = true;
    }

    pub fn update_random_value(&mut self, tick: usize) {
        if tick & 0b1 == 1 {
            self.period_counter -= 1;
            if self.period_counter == 0 {
                self.period_counter = NOISE_PERIOD_LUT[self.period as usize];
                let r = self.random_value;
                self.random_value = if self.mode {
                    r.wrapping_shl(1) | ((r.wrapping_shr(14) ^ r.wrapping_shr(8)) & 0b1)
                } else {
                    r.wrapping_shl(1) | ((r.wrapping_shr(14) ^ r.wrapping_shr(13)) & 0b1)
                }
            }
        }
    }
}
