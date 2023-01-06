use std::collections::VecDeque;

use bitflags::bitflags;
use cartridge::Mirroring;

pub struct AddressRegister {
    buf_hi: u8,
    buf_lo: u8,
    buf_sel: bool, // true: hi, false: lo
}

impl AddressRegister {
    pub fn new() -> Self {
        AddressRegister {
            buf_hi: 0,
            buf_lo: 0,
            buf_sel: true,
        }
    }

    pub fn get(&self) -> u16 {
        ((self.buf_hi as u16) << 8) | (self.buf_lo as u16)
    }

    fn set(&mut self, data: u16) {
        self.buf_hi = (data >> 8) as u8;
        self.buf_lo = (data & 0xff) as u8;
    }

    pub fn update(&mut self, data: u8) {
        if self.buf_sel {
            self.buf_hi = data;
        } else {
            self.buf_lo = data;
        }

        if self.get() > 0x3fff {
            self.set(self.get() & 0b0011_1111_1111_1111);
        }
        self.buf_sel = !self.buf_sel;
    }

    pub fn increment(&mut self, amount: u8) {
        let lo = self.buf_lo;
        self.buf_lo = self.buf_lo.wrapping_add(amount);
        if lo > self.buf_lo {
            self.buf_hi = self.buf_hi.wrapping_add(1);
        }
        if self.get() > 0x3fff {
            self.set(self.get() & 0b0011_1111_1111_1111);
        }
    }

    pub fn reset_latch(&mut self) {
        self.buf_sel = true;
    }
}

bitflags! {
   // 7  bit  0
   // ---- ----
   // VPHB SINN
   // |||| ||||
   // |||| ||++- Base nametable address
   // |||| ||    (0 = $2000; 1 = $2400; 2 = $2800; 3 = $2C00)
   // |||| |+--- VRAM address increment per CPU read/write of PPUDATA
   // |||| |     (0: add 1, going across; 1: add 32, going down)
   // |||| +---- Sprite pattern table address for 8x8 sprites
   // ||||       (0: $0000; 1: $1000; ignored in 8x16 mode)
   // |||+------ Background pattern table address (0: $0000; 1: $1000)
   // ||+------- Sprite size (0: 8x8 pixels; 1: 8x16 pixels)
   // |+-------- PPU master/slave select
   // |          (0: read backdrop from EXT pins; 1: output color on EXT pins)
   // +--------- Generate an NMI at the start of the
   //            vertical blanking interval (0: off; 1: on)
   pub struct ControlRegister: u8 {
       const NAMETABLE1              = 0b00000001;
       const NAMETABLE2              = 0b00000010;
       const VRAM_ADD_INCREMENT      = 0b00000100;
       const SPRITE_PATTERN_ADDR     = 0b00001000;
       const BACKROUND_PATTERN_ADDR  = 0b00010000;
       const SPRITE_SIZE             = 0b00100000;
       const MASTER_SLAVE_SELECT     = 0b01000000;
       const GENERATE_NMI            = 0b10000000;
   }
}

impl ControlRegister {
    pub fn new() -> Self {
        ControlRegister::from_bits_truncate(0)
    }

    pub fn vram_increment_amount(&self) -> u8 {
        if !self.contains(ControlRegister::VRAM_ADD_INCREMENT) {
            1
        } else {
            32
        }
    }

    pub fn update(&mut self, data: u8) {
        self.bits = data;
    }

    pub fn enable_generage_nmi(&self) -> bool {
        self.contains(ControlRegister::GENERATE_NMI)
    }
}

bitflags! {
    // 7  bit  0
    // ---- ----
    // VSO. ....
    // |||| ||||
    // |||+-++++- PPU open bus. Returns stale PPU bus contents.
    // ||+------- Sprite overflow. The intent was for this flag to be set
    // ||         whenever more than eight sprites appear on a scanline, but a
    // ||         hardware bug causes the actual behavior to be more complicated
    // ||         and generate false positives as well as false negatives; see
    // ||         PPU sprite evaluation. This flag is set during sprite
    // ||         evaluation and cleared at dot 1 (the second dot) of the
    // ||         pre-render line.
    // |+-------- Sprite 0 Hit.  Set when a nonzero pixel of sprite 0 overlaps
    // |          a nonzero background pixel; cleared at dot 1 of the pre-render
    // |          line.  Used for raster timing.
    // +--------- Vertical blank has started (0: not in vblank; 1: in vblank).
    //            Set at dot 1 of line 241 (the line *after* the post-render
    //            line); cleared after reading $2002 and at dot 1 of the
    //            pre-render line.
    pub struct StatusRegister: u8 {
        const SPRITE_OVERFLOW         = 0b00100000;
        const SPRITE_0_HIT            = 0b01000000;
        const VBLANK_STARTED          = 0b10000000;
    }
}


pub struct Registers {
    addr: AddressRegister,
    ctrl: ControlRegister,
    stat: StatusRegister
}

#[derive(PartialEq, Eq)]
pub enum TickResult {
    Noop,
    ShouldInterruptNmi,
    ScanlineReset
}

pub struct Ppu {
    pub chr_rom: Vec<u8>,
    pub palette_table: [u8; 32],
    pub vram: [u8; 2048],
    pub oam_data: [u8; 256],
    pub reg: Registers,
    pub mirroring: Mirroring,
    fifo: VecDeque<u8>,

    cycles: usize,
    scanlines: usize,
}

impl Ppu {
    pub fn new() -> Self {
        Ppu {
            vram: [0; 2048],
            oam_data: [0; 64 * 4],
            palette_table: [0; 32],
            reg: Registers {
                addr: AddressRegister::new(),
                ctrl: ControlRegister::new(),
                stat: StatusRegister::from_bits_truncate(0)
            },
            fifo: VecDeque::from([0]),
            chr_rom: [0; 0x2000].to_vec(),
            mirroring: Mirroring::Invalid,
            cycles: 0,
            scanlines: 0,
        }
    }

    pub fn load_cartridge(chr_rom: Vec<u8>, mirroring: Mirroring) -> Self {
        Ppu {
            vram: [0; 2048],
            oam_data: [0; 64 * 4],
            palette_table: [0; 32],
            reg: Registers {
                addr: AddressRegister::new(),
                ctrl: ControlRegister::new(),
                stat: StatusRegister::from_bits_truncate(0),
            },
            fifo: VecDeque::from([0]),
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

            if self.scanlines == 241 {
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

        return TickResult::Noop;
    }

    fn increment_vram_addr(&mut self) {
        let amount = self.reg.ctrl.vram_increment_amount();
        self.reg.addr.increment(amount);
    }

    pub fn read_data(&mut self) -> u8 {
        let addr = self.reg.addr.get();
        self.increment_vram_addr();

        match addr {
            0..=0x1fff => {
                self.fifo.push_back(self.chr_rom[addr as usize]);
                self.fifo.pop_front().expect("Invalid operation")
            },
            0x2000..=0x3eff => {
                let mirror_addr = self.get_mirror_addr(addr);
                self.fifo.push_back(self.vram[mirror_addr]);
                self.fifo.pop_front().expect("Invalid operation")
            },
            0x3f00..=0x3fff => self.palette_table[(addr - 0x3f00) as usize],
            0x4000..=0xffff => {
                let addr = addr & 0b0011_1111_1111_1111;
                let mirror_addr = self.get_mirror_addr(addr);
                self.fifo.push_back(self.vram[mirror_addr]);
                self.fifo.pop_front().expect("Invalid operation")
            }
        }
    }

    pub fn read_stat(&mut self) -> u8 {
        let result = self.reg.stat.bits();
        self.reg.stat &= !StatusRegister::VBLANK_STARTED;
        result
    }

    pub fn write_ctrl(&mut self, data: u8) -> TickResult {
        let before_nmi_status = self.reg.ctrl.contains(ControlRegister::GENERATE_NMI);
        self.reg.ctrl.update(data);
        let after_nmi_status = self.reg.ctrl.contains(ControlRegister::GENERATE_NMI);

        if !before_nmi_status && after_nmi_status {
            TickResult::ShouldInterruptNmi
        } else {
            TickResult::Noop
        }
    }

    pub fn write_addr(&mut self, data: u8) {
        let addr = self.reg.addr.get();
        self.increment_vram_addr();

        match addr {
            0x2000..=0x2fff => {
                let mirror_addr = self.get_mirror_addr(addr);
                self.vram[mirror_addr] = data;
            },
            _ => panic!()
        }
    }

    pub fn write_data(&mut self, data: u8) {
        self.reg.addr.update(data);
        self.increment_vram_addr();
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
            _ => panic!()
        }
    }
}
