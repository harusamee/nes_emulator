use cartridge::Cartridge;
use ppu::Ppu;

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;
const RAM_EFFECTIVE_BITS: u16 = 0b0111_1111_1111;
const PPU_REG_CTRL: u16 = 0x2000;
const PPU_REG_MASK: u16 = 0x2001;
const PPU_REG_STATUS: u16 = 0x2002;
const PPU_REG_OAM_ADDRESS: u16 = 0x2003;
const PPU_REG_OAM_DATA: u16 = 0x2004;
const PPU_REG_SCROLL: u16 = 0x2005;
const PPU_REG_ADDRESS: u16 = 0x2006;
const PPU_REG_DATA: u16 = 0x2007;
const PPU_REG_OAM_DMA: u16 = 0x4014;
const PPU_REGISTERS_MIRRORS: u16 = 0x2008;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;

pub const PRG_ROM: u16 = 0x8000;
const PRG_ROM_END: u16 = 0xffff;



pub struct Bus {
    work_ram: [u8; 0x800],
    cartridge: Cartridge,
    ppu: Ppu
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            work_ram: [0; 0x800],
            cartridge: Cartridge::new(),
            ppu: Ppu::new()
        }
    }

    pub fn load_cartridge(&mut self, raw: &Vec<u8>) {
        self.cartridge = match Cartridge::load(raw) {
            Ok(cartridge) => cartridge,
            Err(message) => panic!("{}", message),
        };
        self.ppu = Ppu::load_cartridge(
            self.cartridge.chr_rom.clone(),
            self.cartridge.screen_mirroring
        );
    }

    #[must_use]
    pub fn read8(&mut self, mut address: u16) -> u8 {
        match address {
            RAM..=RAM_MIRRORS_END => {
                let address = address & RAM_EFFECTIVE_BITS;
                self.work_ram[address as usize]
            }
            PPU_REG_CTRL | PPU_REG_MASK | PPU_REG_OAM_ADDRESS | PPU_REG_SCROLL | PPU_REG_OAM_DMA => {
                0
                // panic!("Invalid read of {:X}", address);
            }
            PPU_REG_STATUS => 0,
            PPU_REG_OAM_DATA => 0,
            PPU_REG_DATA => self.ppu.read_data_0x2007(),
            PRG_ROM..=PRG_ROM_END => {
                if self.cartridge.loaded {
                    address -= 0x8000;
                    if self.cartridge.prg_rom.len() == 0x4000 && address >= 0x4000 {
                        address -= 0x4000;
                    }
                    self.cartridge.prg_rom[address as usize]    
                } else {
                    panic!("Invalid read of {:X}", address);
                }
            }
            PPU_REGISTERS_MIRRORS..=PPU_REGISTERS_MIRRORS_END => {
                let address = address & 0b0010_0000_0000_0111;
                self.read8(address)
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
            PPU_REG_CTRL => self.ppu.write_ctrl(data),
            PPU_REG_MASK => todo!(),
            PPU_REG_STATUS => panic!("Invalid write of {:X}", address),
            PPU_REG_OAM_ADDRESS => todo!(),
            PPU_REG_OAM_DATA => todo!(),
            PPU_REG_SCROLL => todo!(),
            PPU_REG_ADDRESS => self.ppu.write_addr(data),
            PPU_REG_DATA => self.ppu.write_data(data),
            PPU_REG_OAM_DMA => todo!(),
            PPU_REGISTERS_MIRRORS..=PPU_REGISTERS_MIRRORS_END => {
                let address = address & 0b0010_0000_0000_0111;
                self.write8(address, data);
            }
            PRG_ROM..=PRG_ROM_END => {
                panic!("Invalid write of {:X}", address);
            }
            _ => {} // No-op if out of range
        }
    }

    #[must_use]
    pub fn read16(&mut self, address: u16) -> u16 {
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
