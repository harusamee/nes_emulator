use apu::{init_null_apu, Apu};
use cartridge::Cartridge;
use joypad::Joypad;
use ppu::{Ppu, TickResult};

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
pub const PPU_REG_OAM_DMA: u16 = 0x4014;
const PPU_REGISTERS_MIRRORS: u16 = 0x2008;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;
const JOYPAD_1: u16 = 0x4016;
// const JOYPAD_2: u16 = 0x4017;
const APU_REG: u16 = 0x4000;
const APU_REG_END: u16 = 0x4015;

pub const PRG_ROM: u16 = 0x8000;
const PRG_ROM_END: u16 = 0xffff;

pub struct Bus {
    work_ram: [u8; 0x800],
    cartridge: Cartridge,
    pub ppu: Ppu,
    pub joypad1: Joypad,
    pub apu: Apu,
    // joypad2: Joypad,
    cycles: usize,
    should_intr_nmi: bool,
}

impl Bus {
    pub fn new() -> Self {
        Bus {
            work_ram: [0; 0x800],
            cartridge: Cartridge::new(),
            ppu: Ppu::new(),
            cycles: 0,
            should_intr_nmi: false,
            joypad1: Joypad::new(),
            // joypad2: Joypad::new(),
            apu: init_null_apu(),
        }
    }

    pub fn load_cartridge(&mut self, cartridge: Cartridge) {
        self.cartridge = cartridge;
        self.ppu = Ppu::load_cartridge(
            self.cartridge.chr_rom.clone(),
            self.cartridge.screen_mirroring,
        );
    }

    pub fn associate_apu(&mut self, apu: Apu) {
        self.apu = apu;
    }

    #[must_use]
    pub fn read8(&mut self, address: u16) -> u8 {
        self.read8_impl(address, false)
    }

    #[must_use]
    pub fn read8_trace(&mut self, address: u16) -> u8 {
        self.read8_impl(address, true)
    }

    pub fn read8_impl(&mut self, address: u16, trace: bool) -> u8 {
        match address {
            RAM..=RAM_MIRRORS_END => {
                let address = address & RAM_EFFECTIVE_BITS;
                self.work_ram[address as usize]
            }
            PPU_REG_CTRL | PPU_REG_MASK | PPU_REG_OAM_ADDRESS | PPU_REG_SCROLL
            | PPU_REG_ADDRESS | PPU_REG_OAM_DMA => {
                0
                // panic!("Invalid read of {:X}", address);
            }
            PPU_REG_STATUS => self.ppu.read_stat(trace),
            PPU_REG_OAM_DATA => self.ppu.read_oam_data(),
            PPU_REG_DATA => self.ppu.read_data(trace),
            PRG_ROM..=PRG_ROM_END => {
                if self.cartridge.loaded {
                    let mut address = address - 0x8000;
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
                self.read8_impl(address, trace)
            }
            JOYPAD_1 => self.joypad1.read(trace),
            //JOYPAD_2 => self.joypad2.read(trace),
            _ => 0u8, // Returns zero if out of range
        }
    }

    pub fn write8(&mut self, address: u16, data: u8) {
        match address {
            RAM..=RAM_MIRRORS_END => {
                let address = address & RAM_EFFECTIVE_BITS;
                self.work_ram[address as usize] = data;
            }
            PPU_REG_CTRL => {
                let tick_result = self.ppu.write_ctrl(data);
                if tick_result == TickResult::ShouldInterruptNmiAndUpdateTexture {
                    self.should_intr_nmi = true;
                }
            }
            PPU_REG_MASK => self.ppu.write_mask(data),
            PPU_REG_STATUS => panic!("Invalid write of {:X}", address),
            PPU_REG_OAM_ADDRESS => self.ppu.write_oam_addr(data),
            PPU_REG_OAM_DATA => self.ppu.write_oam_data(data),
            PPU_REG_SCROLL => self.ppu.write_scrl(data),
            PPU_REG_ADDRESS => self.ppu.write_addr(data),
            PPU_REG_DATA => self.ppu.write_data(data),
            PPU_REG_OAM_DMA => {
                let dma_start = ((data as u16) << 8) as usize;
                let dma_end = dma_start + 0xff;
                let slice = &self.work_ram[dma_start..=dma_end];
                self.ppu.oam_data.copy_from_slice(slice);
            }
            PPU_REGISTERS_MIRRORS..=PPU_REGISTERS_MIRRORS_END => {
                let address = address & 0b0010_0000_0000_0111;
                self.write8(address, data);
            }
            PRG_ROM..=PRG_ROM_END => {
                panic!("Invalid write of {:X}", address);
            }
            APU_REG..=APU_REG_END => self.apu.write_register(address, data),
            JOYPAD_1 => self.joypad1.write(data),
            //JOYPAD_2 => self.joypad2.write(data),
            _ => {} // No-op if out of range
        }
    }

    #[must_use]
    pub fn read16(&mut self, address: u16) -> u16 {
        self.read16_impl(address, false)
    }

    pub fn read16_trace(&mut self, address: u16) -> u16 {
        self.read16_impl(address, true)
    }

    pub fn read16_impl(&mut self, address: u16, trace: bool) -> u16 {
        let lo = self.read8_impl(address, trace) as u16;
        let hi = self.read8_impl(address + 1, trace) as u16;
        hi << 8 | lo
    }

    pub fn write16(&mut self, address: u16, data: u16) {
        let lo: u8 = (data & 0xff) as u8;
        let hi: u8 = (data >> 8) as u8;
        self.write8(address, lo);
        self.write8(address + 1, hi);
    }

    // For writing codes to CPU ram
    pub fn write_range(&mut self, address: u16, data: Vec<u8>) {
        let to = (address as usize) + data.len();
        let range = (address as usize)..to;
        self.work_ram[range].copy_from_slice(&data);
    }

    pub fn tick(&mut self, cycles: u8) -> Vec<TickResult> {
        self.cycles += cycles as usize;
        self.apu.tick(cycles);
        return self.ppu.tick(cycles * 3);
    }
}

impl Default for Bus {
    fn default() -> Self {
        Bus::new()
    }
}
