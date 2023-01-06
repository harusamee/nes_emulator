mod registers;

use std::collections::VecDeque;

use registers::{MaskRegister, StatusRegister, AddressRegister, ControlRegister, ScrollRegister};
use cartridge::Mirroring;


pub struct Registers {
    mask: MaskRegister,
    stat: StatusRegister,
    addr: AddressRegister,
    ctrl: ControlRegister,
    scrl: ScrollRegister,
    oam_addr: u8,
}

#[derive(PartialEq, Eq)]
pub enum TickResult {
    None,
    ShouldInterruptNmi,
    ScanlineReset,
}

pub struct Ppu {
    pub chr_rom: Vec<u8>,
    pub palette_table: [u8; 32],
    pub vram: [u8; 2048],
    pub oam_data: [u8; 256],
    pub reg: Registers,
    pub mirroring: Mirroring,
    data_fifo: VecDeque<u8>, // temporary buffer for Data Register

    cycles: usize,
    scanlines: usize,
}

impl Ppu {
    pub fn new() -> Self {
        Ppu::load_cartridge(vec![0], Mirroring::Invalid)
    }

    pub fn load_cartridge(chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        Ppu {
            vram: [0; 2048],
            oam_data: [0; 64 * 4],
            palette_table: [0; 32],
            reg: Registers {
                mask: MaskRegister::from_bits_truncate(0),
                stat: StatusRegister::from_bits_truncate(0),
                addr: AddressRegister::new(),
                ctrl: ControlRegister::new(),
                scrl: ScrollRegister::new(),
                oam_addr: 0,
            },
            data_fifo: VecDeque::from([0]),
            chr_rom,
            mirroring,
            cycles: 0,
            scanlines: 0,
        }
    }

    pub fn tick(&mut self, cycles: u8) -> TickResult {
        self.cycles += cycles as usize;

        if self.cycles > 341 {
            self.scanlines += 1;
            self.cycles -= 341;

            if self.scanlines == 240 {
                if self.reg.ctrl.enable_generage_nmi() {
                    self.reg.stat |= StatusRegister::VBLANK_STARTED;
                    return TickResult::ShouldInterruptNmi;
                }
            }

            if self.scanlines >= 262 {
                self.scanlines = 0;
                self.reg.stat &= !StatusRegister::VBLANK_STARTED;
                return TickResult::ScanlineReset;
            }
        }

        return TickResult::None;
    }

    fn is_rendering(&self) -> bool {
        (0..=239).contains(&self.scanlines)
    }

    fn is_vblank(&self) -> bool {
        !self.is_rendering()
    }

    fn increment_vram_addr(&mut self) {
        let amount = self.reg.ctrl.vram_increment_amount();
        self.reg.addr.increment(amount);
    }

    pub fn read_stat(&mut self) -> u8 {
        let result = self.reg.stat.bits();
        self.reg.stat &= !StatusRegister::VBLANK_STARTED;
        result
    }

    pub fn read_oam_data(&self) -> u8 {
        self.oam_data[self.reg.oam_addr as usize]
    }

    pub fn read_data(&mut self) -> u8 {
        let addr = self.reg.addr.get();
        self.increment_vram_addr();

        match addr {
            0..=0x1fff => {
                self.data_fifo.push_back(self.chr_rom[addr as usize]);
                self.data_fifo.pop_front().expect("Invalid operation")
            }
            0x2000..=0x3eff => {
                let mirror_addr = self.get_mirror_addr(addr);
                self.data_fifo.push_back(self.vram[mirror_addr]);
                self.data_fifo.pop_front().expect("Invalid operation")
            }
            0x3f00..=0x3fff => {
                let mirror_addr = addr & 0b0011_1111_0001_1111;
                self.palette_table[(mirror_addr - 0x3f00) as usize]
            }
            0x4000..=0xffff => {
                let addr = addr & 0b0011_1111_1111_1111;
                let mirror_addr = self.get_mirror_addr(addr);
                self.data_fifo.push_back(self.vram[mirror_addr]);
                self.data_fifo.pop_front().expect("Invalid operation")
            }
        }
    }

    pub fn write_mask(&mut self, data: u8) {
        self.reg.mask = MaskRegister::from_bits_truncate(data);
    }

    pub fn write_oam_addr(&mut self, data: u8) {
        self.reg.oam_addr = data;
    }

    pub fn write_oam_data(&mut self, data: u8) {
        if self.is_vblank() {
            self.oam_data[self.reg.oam_addr as usize] = data;
        }
        self.reg.oam_addr = self.reg.oam_addr.wrapping_add(1);
    }

    pub fn write_scrl(&mut self, data: u8) {
        self.reg.scrl.update(data);
    }

    pub fn write_ctrl(&mut self, data: u8) -> TickResult {
        let before_nmi_status = self.reg.ctrl.contains(ControlRegister::GENERATE_NMI);
        self.reg.ctrl.update(data);
        let after_nmi_status = self.reg.ctrl.contains(ControlRegister::GENERATE_NMI);

        if self.is_vblank() && !before_nmi_status && after_nmi_status {
            TickResult::ShouldInterruptNmi
        } else {
            TickResult::None
        }
    }

    pub fn write_addr(&mut self, data: u8) {
        self.reg.addr.update(data);
    }

    pub fn write_data(&mut self, data: u8) {
        let addr = self.reg.addr.get();
        self.increment_vram_addr();

        match addr {
            0x2000..=0x3eff => {
                let mirror_addr = self.get_mirror_addr(addr);
                self.vram[mirror_addr] = data;
            }
            0x3f00..=0x3fff => {
                let mirror_addr = addr & 0b0011_1111_0001_1111;
                self.vram[(mirror_addr - 0x3f00) as usize] = data;
            }
            _ => panic!("Invalid ppu vram write {:04X}", addr),
        }
    }

    fn get_mirror_addr(&self, addr: u16) -> usize {
        // Bitwise AND to make addr in 0x2000-0x2fff
        let addr = addr & 0b0010_1111_1111_1111;
        // To vram vector range
        let vram_addr = (addr - 0x2000) as usize;
        // 0x0000 to 0x03ff -> index = 0
        // 0x0400 to 0x07ff -> index = 1
        // 0x0800 to 0x0cff -> index = 2, etc
        let index = vram_addr / 0x400;

        match (&self.mirroring, index) {
            (Mirroring::Vertical, 0) => vram_addr,
            (Mirroring::Vertical, 1) => vram_addr,
            (Mirroring::Vertical, 2) => vram_addr - 0x800,
            (Mirroring::Vertical, 3) => vram_addr - 0x800,
            (Mirroring::Horizontal, 0) => vram_addr,
            (Mirroring::Horizontal, 1) => vram_addr - 0x400,
            (Mirroring::Horizontal, 2) => vram_addr - 0x400,
            (Mirroring::Horizontal, 3) => vram_addr - 0x800,
            _ => panic!(),
        }
    }
}
