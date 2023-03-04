use crate::wave::Wave;
use crate::wave_trait::AsWave;
use crate::wave_trait::WaveTrait;

pub struct TriangleWave {
    base: Wave,
    counter_halt: bool,
    linear_counter: u8,
    //
    current_output: u8,
    current_level_bit: u8,
    freq_counter: u16,
    linear_counter_internal: u8,
}

mod private {
    use crate::wave_trait::AsWave;
    impl AsWave for super::TriangleWave {
        fn as_wave(&self) -> &super::Wave {
            &self.base
        }
        fn as_mut_wave(&mut self) -> &mut super::Wave {
            &mut self.base
        }
    }
}

impl WaveTrait for TriangleWave {
    fn get_output(&mut self) -> u8 {
        self.current_output
    }

    fn on_length_count(&mut self) {
        if !self.counter_halt && self.as_wave().length_counter > 0 {
            self.as_mut_wave().length_counter -= 1;
        }
        self.update_current_output_with_check();
    }

    fn on_frame(&mut self) {
        todo!()
    }

    fn tick(&mut self, tick: usize) {
        self.freq_counter -= 1;
        if self.freq_counter == 0 {
            self.freq_counter = self.get_freq_11bit();
            if self.current_level_bit == 31 {
                self.current_level_bit = 0;
            } else {
                self.current_level_bit += 1;
            }

            self.update_current_output_with_check();
        }
    }
}

impl TriangleWave {
    pub fn new() -> Self {
        TriangleWave {
            base: Wave::new("Triangle"),
            linear_counter: 0,
            counter_halt: false,
            current_output: 0,
            current_level_bit: 0,
            freq_counter: 0,
            linear_counter_internal: 0,
        }
    }

    fn update_current_output_with_check(&mut self) {
        if self.can_output() {
            self.update_current_output();
          } else {
            self.current_output = 0;
        }

    }

    fn can_output(&self) -> bool {
        if self.base.enable && self.base.length_counter > 0 && self.linear_counter_internal > 0 {
            let freq = self.get_freq_11bit();
            freq >= 0b100
        } else {
            false
        }
    }

    fn update_current_output(&mut self) {
        static WAVEFORM: [u8; 32] = [
            15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 1, //
            1, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
        ];

        self.current_output = WAVEFORM[self.current_level_bit as usize];
    }

    pub fn on_linear_count(&mut self) {
        if !self.counter_halt && self.linear_counter_internal > 0 {
            self.linear_counter_internal -= 1;
            if self.linear_counter_internal == 0 {
                // println!("triangle: linear count zero");
            }
        }
        self.update_current_output_with_check();
    }

    pub fn set_halt(&mut self, halt: bool) {
        self.counter_halt = halt;
        if self.counter_halt {
            // println!("triangle: counter halted");
        }
    }

    pub fn write_0(&mut self, data: u8) {
        let linear_counter = data & 0b0111_1111;
        self.linear_counter = linear_counter;
        self.linear_counter_internal = linear_counter;
        let halt = data & 0b1000_0000 > 0;
        self.set_halt(halt);
    }

    pub fn write_3(&mut self, data: u8) {
        self.set_reg_freq_hi(data);
        self.set_length_counter(data);
        self.set_tone_freq();
        self.linear_counter_internal = self.linear_counter;
    }

}
