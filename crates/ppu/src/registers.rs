
use bitflags::bitflags;

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
        //println!("ppu addr reg: {:04X}", self.get());
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
        //println!("ppu addr inc: {:04X}", self.get());
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

    // 7  bit  0
    // ---- ----
    // BGRs bMmG
    // |||| ||||
    // |||| |||+- Greyscale (0: normal color, 1: produce a greyscale display)
    // |||| ||+-- 1: Show background in leftmost 8 pixels of screen, 0: Hide
    // |||| |+--- 1: Show sprites in leftmost 8 pixels of screen, 0: Hide
    // |||| +---- 1: Show background
    // |||+------ 1: Show sprites
    // ||+------- Emphasize red (green on PAL/Dendy)
    // |+-------- Emphasize green (red on PAL/Dendy)
    // +--------- Emphasize blue
    pub struct MaskRegister: u8 {
        const GRAYSCALE               = 0b00000001;
        const SHOW_LEFTMOST_BG        = 0b00000010;
        const SHOW_LEFTMOST_SPRITES   = 0b00000100;
        const SHOW_BG                 = 0b00001000;
        const SHOW_SPRITES            = 0b00010000;
        const EMPHASIZE_RED           = 0b00100000;
        const EMPHASIZE_GREEN         = 0b01000000;
        const EMPHASIZE_BLUE          = 0b10000000;
    }

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

pub struct ScrollRegister {
    buf_hi: u8,
    buf_lo: u8,
    buf_sel: bool, // true: hi, false: lo
}

impl ScrollRegister {
    pub fn new() -> Self {
        ScrollRegister {
            buf_hi: 0,
            buf_lo: 0,
            buf_sel: true,
        }
    }

    pub fn get(&self) -> (u8, u8) {
        (self.buf_hi, self.buf_lo)
    }

    pub fn update(&mut self, data: u8) {
        if self.buf_sel {
            self.buf_hi = data;
        } else {
            self.buf_lo = data;
        }

        self.buf_sel = !self.buf_sel;
    }

    pub fn reset_latch(&mut self) {
        self.buf_sel = true;
    }
}