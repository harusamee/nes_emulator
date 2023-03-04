use crate::wave::Wave;
use crate::wave_trait::AsWave;
use crate::wave_trait::WaveTrait;

pub struct PulseWave {
    base: Wave,
    phase: f32,
    // 0x4000 or 0x4004
    duty: u8,
    envelope_counter_halt: bool,
    contant_volume: bool,
    volume: u8,
    // 0x4001 or 0x4005
    sweep_shifts: u8,
    sweep_period: u8,
    sweep_enable: bool,
    sweep_negate: bool,
    // internal flag/counter
    freq_counter: u16,
    current_duty_bit: u8,
    current_output: u8,
    envelope_start_flag: bool,
    decay_counter: u8,
    envelope_counter: u8,
    sweep_start_flag: bool,
    sweep_counter: u8,
    current_volume: u8,
}

mod private {
    use crate::wave_trait::AsWave;

    impl AsWave for super::PulseWave {
        fn as_wave(&self) -> &super::Wave {
            &self.base
        }
        fn as_mut_wave(&mut self) -> &mut super::Wave {
            &mut self.base
        }
    }
}

impl WaveTrait for PulseWave {
    fn on_length_count(&mut self) {
        if !self.envelope_counter_halt && self.as_wave().length_counter > 0 {
            self.as_mut_wave().length_counter -= 1;
            if self.as_wave().length_counter == 0 {
                // println!("pulse: length count zero");
            }
        }
    }
    fn on_frame(&mut self) {
        todo!()
    }

    fn get_output(&mut self) -> u8 {
        self.current_output
    }

    fn tick(&mut self, tick: usize) {
        if tick & 0b1 > 0 {
            self.freq_counter -= 1;
            if self.freq_counter == 0 {
                self.freq_counter = self.get_freq_11bit();
                if self.current_duty_bit == 0 {
                    self.current_duty_bit = 7;
                } else {
                    self.current_duty_bit -= 1;
                }
                self.update_current_output();
            }
        }
    }
}

impl PulseWave {
    pub fn new(no: u8) -> Self {
        PulseWave {
            base: Wave::new(&format!("Pulse{}", no)),
            phase: 0.0,
            duty: 0,
            envelope_counter_halt: false,
            contant_volume: false,
            volume: 0,
            envelope_start_flag: false,
            decay_counter: 1,
            envelope_counter: 0,
            sweep_start_flag: false,
            sweep_enable: false,
            sweep_period: 0u8,
            sweep_negate: false,
            sweep_shifts: 0u8,
            sweep_counter: 1,
            freq_counter: 1,
            current_duty_bit: 0,
            current_output: 0,
            current_volume: 0
        }
    }

    fn can_output(&self) -> bool {
        if self.as_wave().enable && self.as_wave().length_counter > 0 {
            let freq = self.get_freq_11bit();
            if freq > 0b111 {
                let freq = if self.sweep_enable {
                    freq + (freq >> self.sweep_shifts)
                } else {
                    freq
                };
                return freq < 0b111_1111_1111;
            }
        }
        false
    }

    fn update_current_output(&mut self) {
        static DUTY_TO_WAVEFORM: [[u8; 8]; 4] = [
            [0, 0, 0, 0, 0, 0, 0, 1],
            [0, 0, 0, 0, 0, 0, 1, 1],
            [0, 0, 0, 0, 1, 1, 1, 1],
            [1, 1, 1, 1, 1, 1, 0, 0],
        ];

        let waveform = DUTY_TO_WAVEFORM[self.duty as usize];
        let result = waveform[self.current_duty_bit as usize];
        self.current_output = result * self.current_volume;
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

        if self.contant_volume {
            self.current_volume = self.volume;
        } else {
            self.current_volume = self.decay_counter;
        }

        if self.can_output() {
            self.update_current_output();
        } else {
            self.current_output = 0;
        }
    }

    pub fn process_sweep(&mut self) {
        if self.sweep_counter == 0 {
            self.sweep_counter = self.sweep_period;
            if self.sweep_enable && self.sweep_shifts > 0 {
                let mut freq = self.get_freq_11bit();
                let sweep_amount = freq >> self.sweep_shifts;
                freq = if self.sweep_negate {
                    freq.saturating_sub(sweep_amount)
                } else {
                    freq.saturating_add(sweep_amount)
                };

                self.as_mut_wave().reg_freq_lo = (freq & 0b1111_1111) as u8;
                self.as_mut_wave().reg_freq_hi = ((freq & 0b0111_0000_0000) >> 8) as u8;
                self.set_tone_freq();
            }
        } else {
            self.sweep_counter -= 1;
        }
        if self.sweep_start_flag {
            self.sweep_start_flag = false;
            self.sweep_counter = self.sweep_period;
        }

        if self.can_output() {
            self.update_current_output();
        } else {
            self.current_output = 0;
        }
    }

    pub fn write_0(&mut self, data: u8) {
        let duty = (data & 0b1100_0000) >> 6;
        let halt = data & 0b0010_0000 > 0;
        let const_vol = data & 0b0001_0000 > 0;
        let envelope = data & 0b0000_1111;

        self.duty = duty;
        self.envelope_counter_halt = halt;
        self.contant_volume = const_vol;
        self.volume = envelope;
        if self.envelope_counter_halt {
            // println!("{}: counter halted", &self.base.name);
        }
        if const_vol {
            self.current_volume = self.volume;
        } else {
            self.current_volume = self.decay_counter;
        }
    }

    pub fn write_1(&mut self, data: u8) {
        let enable = data & 0b1000_0000 > 0;
        let period = (data & 0b0111_0000) >> 4;
        let negate = data & 0b0000_1000 > 0;
        let shifts = data & 0b0000_0111;

        self.sweep_enable = enable;
        self.sweep_period = period;
        self.sweep_negate = negate;
        self.sweep_shifts = shifts;
        self.sweep_start_flag = true;
    }

    pub fn write_3(&mut self, data: u8) {
        self.set_reg_freq_hi(data);
        self.set_length_counter(data);
        // println!(
        //     "{}: length count {}",
        //     &self.base.name,
        //     self.base.length_counter as f32 / 120.0
        // );

        self.set_tone_freq();
        self.envelope_start_flag = true;
        self.current_duty_bit = 0;
    }
}
