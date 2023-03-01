pub struct Wave {
    pub name: String,
    pub enable: bool,
    pub tone_freq: f32,
    pub reg_freq_lo: u8,
    pub reg_freq_hi: u8,
    pub length_counter: u8,
    pub phase_inc: f32,
}

impl Wave {
    pub fn new(name: &str) -> Self {
        Wave {
            enable: false,
            tone_freq: 0.0,
            reg_freq_lo: 0,
            reg_freq_hi: 0,
            length_counter: 0,
            phase_inc: 0.0,
            name: String::from(name),
        }
    }
}
