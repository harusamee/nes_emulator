use core::panic;
use std::ops::Shl;

use crate::constants::LENGTH_COUNTER_LUT;
use crate::wave::Wave;
use crate::wave_trait::AsWave;
use crate::wave_trait::WaveTrait;

static RATE_INDEX: [u16; 16] = [
    428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72, 54,
];

pub struct Dmc {
    base: Wave,
    // 0x4010
    enable_irq: bool,
    loop_flag: bool,
    rate: u8,
    // 0x4011
    current_output: u8,
    // 0x4012
    sample_address: u16,
    // 0x4013
    sample_length: u16,
    // internal
    rate_counter: u16,
    start_flag: bool,
    sample_buffer_consuming: u8,
    sample_buffer_shifts: u8,
    length_counter_u16: u16,
}

mod private {
    use crate::wave_trait::AsWave;
    impl AsWave for super::Dmc {
        fn as_wave(&self) -> &super::Wave {
            &self.base
        }
        fn as_mut_wave(&mut self) -> &mut super::Wave {
            &mut self.base
        }
    }
}

impl WaveTrait for Dmc {
    fn on_length_count(&mut self) {}

    fn on_frame(&mut self) {
        todo!()
    }

    fn get_output(&mut self) -> u8 {
        self.current_output
    }

    fn set_enable(&mut self, enable: bool) {
        self.as_mut_wave().enable = enable;
        self.start_flag = enable;
    }

    fn tick(&mut self, tick: usize) {
        todo!()
    }
}

impl Dmc {
    pub fn new() -> Self {
        Dmc {
            base: Wave::new("DMC"),
            enable_irq: false,
            loop_flag: false,
            rate: 0,
            current_output: 0,
            sample_address: 0,
            sample_length: 0,
            rate_counter: 511,
            start_flag: false,
            sample_buffer_consuming: 0,
            sample_buffer_shifts: 0,
            length_counter_u16: 0,
        }
    }

    pub fn write_0(&mut self, data: u8) {
        let enable_irq = data & 0b1000_0000 > 0;
        let loop_flag = data & 0b0100_0000 > 0;
        let rate = data & 0b0000_1111;

        self.enable_irq = enable_irq;
        self.loop_flag = loop_flag;
        self.rate = rate;
    }

    pub fn write_1(&mut self, data: u8) {
        let sample_buffer = data & 0b0111_1111;
        self.current_output = sample_buffer;
    }

    pub fn write_2(&mut self, data: u8) {
        self.sample_address = 0xc000u16 + (data as u16).shl(6);
    }

    pub fn write_3(&mut self, data: u8) {
        self.sample_length = (data as u16).shl(4) + 1u16;
    }

    pub fn update_output_value(&mut self, tick: usize) {
        if tick & 0b1 > 0 {
            if self.start_flag {
                self.start_flag = false;
            }
        }

        if tick & 0b1 == 0 {
            self.rate_counter -= 1;
            if self.rate_counter == 0 {
                self.rate_counter = RATE_INDEX[self.rate as usize];

                if self.length_counter_u16 > 0 {
                    let has_bit = self.sample_buffer_consuming & 0b1 > 0;
                    let upper_limit = self.current_output > 0x7d;
                    let lower_limit = self.current_output < 0x02;
                    match (has_bit, upper_limit, lower_limit) {
                        (true, false, _) => {
                            self.current_output += 2;
                        }
                        (false, _, false) => {
                            self.current_output -= 2;
                        }
                        _ => {}
                    }
                    self.sample_buffer_consuming >>= 1;
                }

                self.sample_buffer_shifts += 1;
                // if all bits consumed
                if self.sample_buffer_shifts == 8 {
                    self.sample_buffer_shifts = 0;

                    self.length_counter_u16 -= 1;
                    self.sample_buffer_consuming = self.get_sample(self.sample_address);
                    self.sample_address += 1;
                }
            }
        }
    }

    fn get_sample(&self, address: u16) -> u8 {
        0
    }
}
