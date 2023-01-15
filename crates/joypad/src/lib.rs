use bitflags::bitflags;

bitflags! {
    pub struct JoypadButton: u8 {
        const RIGHT             = 0b10000000;
        const LEFT              = 0b01000000;
        const DOWN              = 0b00100000;
        const UP                = 0b00010000;
        const START             = 0b00001000;
        const SELECT            = 0b00000100;
        const BUTTON_B          = 0b00000010;
        const BUTTON_A          = 0b00000001;
    }
}

pub struct Joypad {
    strobe: bool,
    index: u8,
    pub button: JoypadButton,
}

impl Joypad {
    pub fn new() -> Self {
        Joypad {
            strobe: false,
            index: 0,
            button: JoypadButton::from_bits_truncate(0),
        }
    }

    pub fn read(&mut self, trace: bool) -> u8 {
        if self.index > 7 {
            return 1;
        }

        let result = match self.strobe {
            true => self.button.contains(JoypadButton::BUTTON_A),
            false => {
                let mask = 1 << self.index;
                let result = self.button.bits() & mask > 0;
                if !trace {
                    self.index += 1;
                }

                result
            }
        };
        u8::from(result)
    }

    pub fn write(&mut self, data: u8) {
        self.strobe = data == 1;
        if self.strobe {
            self.index = 0;
        }
    }

    pub fn set_button_status(&mut self, target: &JoypadButton, status: bool) {
        match status {
            true => {
                let bits = self.button.bits() | target.bits();
                self.button = JoypadButton::from_bits_truncate(bits);
            }
            false => {
                let bits = self.button.bits() & !target.bits();
                self.button = JoypadButton::from_bits_truncate(bits);
            }
        }
    }
}
