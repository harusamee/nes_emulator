use crate::wave::Wave;
use crate::constants::*;


pub trait AsWave {
    fn as_wave(&self) -> &Wave;
    fn as_mut_wave(&mut self) -> &mut Wave;
}

pub trait WaveTrait: AsWave {
    fn set_enable(&mut self, enable: bool) {
        self.as_mut_wave().enable = enable;
        if !enable {
            self.as_mut_wave().length_counter = 0;
        }
    }

    fn set_tone_freq(&mut self) {
        const CPU_FREQ: f64 = 1.789773 * 1000.0 * 1000.0;
        let in_freq_11bit = self.get_freq_11bit();
        let tone_freq = (CPU_FREQ / (16.0 * (in_freq_11bit as f64 + 1.0))) as f32;
        if self.as_wave().name == "Pulse2" {
            println!("tone freq {} -> {}", in_freq_11bit, tone_freq);
        }
        self.as_mut_wave().tone_freq = tone_freq;
        self.as_mut_wave().phase_inc = tone_freq / SAMPLES_PER_SEC as f32;
    }

    fn get_freq_11bit(&self) -> u16 {
        let hi = self.as_wave().reg_freq_hi as u16 & 0b111;
        let lo = self.as_wave().reg_freq_lo as u16;
        (hi & 0b111) << 8 | lo
    }

    fn set_reg_freq_lo(&mut self, reg_freq_lo: u8) {
        self.as_mut_wave().reg_freq_lo = reg_freq_lo;
    }

    fn set_reg_freq_hi(&mut self, reg_freq_hi: u8) {
        self.as_mut_wave().reg_freq_hi = reg_freq_hi;
    }

    fn set_length_counter(&mut self, length_counter: u8) {
        let length_counter_key = (length_counter & 0b1111_1000) >> 3;
        let length_counter = LENGTH_COUNTER_LUT[length_counter_key as usize];
        self.as_mut_wave().length_counter = length_counter;
    }

    // fn set_frame_counter(&mut self, frame_counter: u8) {
    //     self.as_mut_wave().frame_counter = frame_counter;
    // }

    fn on_length_count(&mut self);
    fn on_frame(&mut self);

    fn get_output(&mut self) -> u8;

    fn tick(&mut self, tick: usize);
}
