#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Mirroring {
    Invalid,
    Vertical,
    Horizontal,
    FourScreen,
}

#[derive(Debug)]
pub enum VideoSignal {
    PAL,
    NTSC
}

pub struct Cartridge {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mapper: u8,
    pub screen_mirroring: Mirroring,
    pub loaded: bool,
    pub video_signal: VideoSignal
}

const NES_TAG: [u8; 4] = [b'N', b'E', b'S', 0x1a];

impl Cartridge {
    pub fn new() -> Cartridge {
        Cartridge {
            prg_rom: Vec::from([]),
            chr_rom: Vec::from([]),
            mapper: 0,
            screen_mirroring: Mirroring::Invalid,
            loaded: false,
            video_signal: VideoSignal::NTSC,
        }
    }

    pub fn load(raw: &Vec<u8>) -> Result<Cartridge, &str> {
        if &raw[0..4] != NES_TAG {
            return Err("Header did not NES^Z");
        }

        let has_trainer = raw[6] & 0b0100 > 0;
        let prg_rom_size = 16 * 1024 * (raw[4] as usize);
        let chr_rom_size = 8 * 1024 * (raw[5] as usize);
        let prg_rom_start = 0x10 + if has_trainer { 512 } else { 0 };
        let chr_rom_start = prg_rom_start + prg_rom_size;
        let prg_rom_range = prg_rom_start..(prg_rom_start + prg_rom_size);
        let chr_rom_range = chr_rom_start..(chr_rom_start + chr_rom_size);

        let mapper = ((raw[6] & 0b1111_0000) >> 4) | (raw[7] & 0b1111_0000);

        let screen_mirroring = match raw[6] & 0b0000_1001 {
            0b0000_0000 => Mirroring::Horizontal,
            0b0000_0001 => Mirroring::Vertical,
            0b0000_1000 => Mirroring::FourScreen,
            0b0000_1001 => Mirroring::FourScreen,
            _ => {
                return Err("Invalid screen mirroring type");
            }
        };

        let video_signal = match raw[9] & 1 {
            1 => VideoSignal::PAL,
            _ => VideoSignal::NTSC
        };

        eprintln!("prg_rom: {:?} 0x{:04X}", prg_rom_range, prg_rom_size);
        eprintln!("chr_rom: {:?} 0x{:04X}", chr_rom_range, chr_rom_size);
        eprintln!("video_signal: {:?}", video_signal);

        Ok(Cartridge {
            prg_rom: raw[prg_rom_range].to_vec(),
            chr_rom: raw[chr_rom_range].to_vec(),
            mapper,
            screen_mirroring,
            video_signal,
            loaded: true,
        })
    }
}
