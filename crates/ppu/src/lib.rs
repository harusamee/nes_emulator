mod registers;
mod renderer;

use std::collections::VecDeque;

use renderer::Renderer;
use registers::{MaskRegister, StatusRegister, AddressRegister, ControlRegister, ScrollRegister};
use cartridge::Mirroring;
use sdl2::render::Texture;

pub const WIDTH: usize = 256;
pub const HEIGHT: usize = 240;

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
    ShouldInterruptNmiAndUpdateTexture,
    ShouldUpdateTexture,    
    ScanlineReset,
}

pub struct Ppu {
    pub chr_rom: Vec<u8>,
    pub palette_table: [u8; 32],
    pub vram: [u8; 2048],
    pub oam_data: [u8; 256],
    pub reg: Registers,
    pub mirroring: Mirroring,
    data_fifo: u8, // temporary buffer for Data Register

    cycles: usize,
    scanlines: usize,
    fb: Renderer
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
            data_fifo: 0,
            chr_rom,
            mirroring,
            cycles: 0,
            scanlines: 0,
            fb: Renderer::new()
        }
    }

    pub fn tick(&mut self, cycles: u8) -> TickResult {
        self.cycles += cycles as usize;

        if self.cycles > 341 {
            self.scanlines += 1;
            self.cycles -= 341;

            // Render background every 8 lines except 0
            let show_bg = self.reg.mask.contains(MaskRegister::SHOW_BG);
            if show_bg && (1..=240).contains(&self.scanlines) && self.scanlines & 0b111 == 0 {
                let row_number = (self.scanlines - 1) / 8;
                let bg_pattern_addr = self.reg.ctrl & ControlRegister::BACKROUND_PATTERN_ADDR;
                let offset = (bg_pattern_addr.bits() >> 1) as usize;
                self.fb.render_bg_row(row_number, offset, &self.chr_rom, &self.palette_table, &self.vram);
            }

            if self.scanlines == 241 {
                let show_sprites = self.reg.mask.contains(MaskRegister::SHOW_SPRITES);
                if show_sprites {
                    let offset = (self.reg.ctrl & ControlRegister::SPRITE_PATTERN_ADDR).bits();
                    let sprite_8x16 = self.reg.ctrl.contains(ControlRegister::SPRITE_SIZE);
                    self.fb.render_sprites(offset, sprite_8x16, &self.chr_rom, &self.palette_table, &self.vram, &self.oam_data);    
                }

                self.reg.stat |= StatusRegister::VBLANK_STARTED;
                if self.reg.ctrl.enable_generage_nmi() {
                    return TickResult::ShouldInterruptNmiAndUpdateTexture;
                } else {
                    return TickResult::ShouldUpdateTexture;
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

    pub fn read_stat(&mut self, trace: bool) -> u8 {
        let result = self.reg.stat.bits();
        if !trace {
            self.reg.stat &= !StatusRegister::VBLANK_STARTED;
            self.reg.scrl.reset_latch();
            self.reg.addr.reset_latch();
        }
        result
    }

    pub fn read_oam_data(&self) -> u8 {
        self.oam_data[self.reg.oam_addr as usize]
    }

    pub fn read_data(&mut self, trace: bool) -> u8 {
        let addr = self.reg.addr.get();
        if !trace {
            self.increment_vram_addr();
        }

        match addr {
            0..=0x1fff => {
                if trace {
                    self.data_fifo
                } else {
                    let result = self.data_fifo;
                    self.data_fifo = self.chr_rom[addr as usize];
                    result
                }
            }
            0x2000..=0x3eff => {
                let mirror_addr = self.get_mirror_addr(addr);
                if trace {
                    self.data_fifo
                } else {
                    let result = self.data_fifo;
                    self.data_fifo = self.vram[mirror_addr as usize];
                    result
                }
            }
            0x3f00..=0x3fff => {
                let mirror_addr = addr & 0b0011_1111_0001_1111;
                let palette_addr = (mirror_addr - 0x3f00) as usize;
                match palette_addr {
                    0x10 | 0x14 | 0x18 | 0x1c => self.palette_table[palette_addr - 0x10],
                    otherwise => self.palette_table[otherwise]
                }
            }
            0x4000..=0xffff => {
                let addr = addr & 0b0011_1111_1111_1111;
                let mirror_addr = self.get_mirror_addr(addr);
                if trace {
                    self.data_fifo
                } else {
                    let result = self.data_fifo;
                    self.data_fifo = self.vram[mirror_addr as usize];
                    result
                }
            }
        }
    }

    pub fn write_mask(&mut self, data: u8) {
        self.reg.mask = MaskRegister::from_bits_truncate(data);

        let bg = self.reg.mask.contains(MaskRegister::SHOW_LEFTMOST_BG);
        let sprite = self.reg.mask.contains(MaskRegister::SHOW_LEFTMOST_SPRITES);
        self.fb.set_draw_leftmost(bg, sprite);
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
            TickResult::ShouldInterruptNmiAndUpdateTexture
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
                let palette_addr = (mirror_addr - 0x3f00) as usize;
                let target_addr = match palette_addr {
                    0x10 | 0x14 | 0x18 | 0x1c => palette_addr - 0x10,
                    otherwise => otherwise
                };
                self.palette_table[target_addr as usize] = data;
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

    pub fn update_sdl_texture(&self, texture: &mut Texture) {
        texture.update(None, &self.fb.get_buffer(), WIDTH * 3).unwrap();
    }
}
