use crate::wave_trait::WaveTrait;
use crate::wave_trait::AsWave;
use crate::wave::Wave;


pub struct TriangleWave {
    base: Wave,
    phase: f32,
    counter_halt: bool,
    linear_counter: u8,
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
        const WAVEFORM: [u8; 32] = [
            15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0, //
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
        ];

        let mut result = 0u8;
        if self.as_wave().enable && self.as_wave().length_counter > 0 && self.linear_counter > 0 {
            let phase_x32 = (self.phase * 32.0).floor() as usize;
            result = WAVEFORM[phase_x32];
        }
        self.phase = (self.phase + self.as_wave().phase_inc) % 1.0;
        result
    }

    fn on_length_count(&mut self) {
        if !self.counter_halt && self.as_wave().length_counter > 0 {
            self.as_mut_wave().length_counter -= 1;
        }
    }

    fn on_frame(&mut self) {
        todo!()
    }

    fn tick(&mut self, tick: usize) {
    }
}

impl TriangleWave {
    pub fn new() -> Self {
        TriangleWave {
            base: Wave::new("Triangle"),
            phase: 0.0,
            linear_counter: 0,
            counter_halt: false,
        }
    }

    pub fn set_linear_counter(&mut self, linear_counter: u8) {
        self.linear_counter = linear_counter;
        // println!(
        //     "triangle: linear counter {} sec",
        //     self.linear_counter as f32 / 240.0
        // );
    }

    pub fn on_linear_count(&mut self) {
        if !self.counter_halt && self.linear_counter > 0 {
            self.linear_counter -= 1;
            if self.linear_counter == 0 {
                // println!("triangle: linear count zero");
            }
        }
    }

    pub fn set_halt(&mut self, halt: bool) {
        self.counter_halt = halt;
        if self.counter_halt {
            // println!("triangle: counter halted");
        }
}
}