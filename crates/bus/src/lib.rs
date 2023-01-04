use cartridge::Cartridge;

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;
const RAM_EFFECTIVE_BITS: u16 = 0b0111_1111_1111;
const PPU_REGISTERS: u16 = 0x2000;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;
const PPU_REGISTERS_EFFECTIVE_BITS: u16 = 0b0010_0000_0000_0111;
pub const PRG_ROM: u16 = 0x8000;
const PRG_ROM_END: u16 = 0xffff;



pub struct Bus {
    work_ram: [u8; 0x800],
    cartridge: Cartridge
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            work_ram: [0; 0x800],
            cartridge: Cartridge::new(),
        }
    }

    pub fn load_cartridge(&mut self, raw: &Vec<u8>) {
        self.cartridge = match Cartridge::load(raw) {
            Ok(cartridge) => cartridge,
            Err(message) => panic!("{}", message),
        }
    }

    #[must_use]
    pub fn read8(&self, mut address: u16) -> u8 {
        match (address, self.cartridge.loaded) {
            (RAM..=RAM_MIRRORS_END, _) => {
                let address = address & RAM_EFFECTIVE_BITS;
                self.work_ram[address as usize]
            }
            (PPU_REGISTERS..=PPU_REGISTERS_MIRRORS_END, _) => {
                let _address = address & PPU_REGISTERS_EFFECTIVE_BITS;
                todo!();
            }
            (PRG_ROM..=PRG_ROM_END, true) => {
                address -= 0x8000;
                if self.cartridge.prg_rom.len() == 0x4000 && address >= 0x4000 {
                    address -= 0x4000;
                }
                self.cartridge.prg_rom[address as usize]
            }
            (PRG_ROM..=PRG_ROM_END, false) => {
                panic!("Invalid read of {:X}", address);
            }
            _ => 0u8 // Returns zero if out of range
        }
    }

    pub fn write8(&mut self, address: u16, data: u8) {
        match address {
            RAM..=RAM_MIRRORS_END => {
                let address = address & RAM_EFFECTIVE_BITS;
                self.work_ram[address as usize] = data;
            }
            PPU_REGISTERS..=PPU_REGISTERS_MIRRORS_END => {
                let _address = address & PPU_REGISTERS_EFFECTIVE_BITS;
                todo!();
            }
            PRG_ROM..=PRG_ROM_END => {
                panic!("Invalid write of {:X}", address);
            }
            _ => {} // No-op if out of range
        }
    }

    #[must_use]
    pub fn read16(&self, address: u16) -> u16 {
        let lo = self.read8(address) as u16;
        let hi = self.read8(address + 1) as u16;
        hi << 8 | lo
    }

    pub fn write16(&mut self, address: u16, data: u16) {
        let lo: u8 = (data & 0xff) as u8;
        let hi: u8 = (data >> 8) as u8;
        self.write8(address, lo);
        self.write8(address + 1, hi);
    }

    pub fn write_range(&mut self, address: u16, data: Vec<u8>) {
        let to = (address as usize) + data.len();
        let range = (address as usize)..to;
        self.work_ram[range].copy_from_slice(&data);
    }
}

impl Default for Bus {
    fn default() -> Self {
        Bus::new()        
    }
}
