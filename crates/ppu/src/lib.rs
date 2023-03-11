mod registers;
mod renderer;

use core::panic;

use renderer::Renderer;
use registers::{MaskRegister, StatusRegister, AddressRegister, ControlRegister, ScrollRegister};
use cartridge::Mirroring;
use sdl2::render::Texture;

pub const WIDTH: usize = 256;
pub const HEIGHT: usize = 240;

#[derive(Default)]
struct InternalRegister15 {
    coarse_x: u8,
    coarse_y: u8,
    nametable_x: bool,
    nametable_y: bool,
    fine_y: u8
}

pub struct Registers {
    mask: MaskRegister,
    stat: StatusRegister,
    addr: AddressRegister,
    ctrl: ControlRegister,
    scrl: ScrollRegister,
    oam_addr: u8,
    internal_t: InternalRegister15,
    internal_v: InternalRegister15,
    // fine_x: u8
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

    pub fn new_test_vertical() -> Self {
        Ppu::load_cartridge(vec![0; 2048], Mirroring::Vertical)
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
                internal_t: Default::default(),
                internal_v: Default::default(),
                // fine_x: 0,
            },
            data_fifo: 0,
            chr_rom,
            mirroring,
            cycles: 21,
            scanlines: 0,
            fb: Renderer::new()
        }
    }

    pub fn set_renderer_enabled(&mut self, enabled: bool) {
        self.fb.set_enabled(enabled);
    }

    pub fn get_cycles_scanlines(&self) -> (usize, usize) {
        // this is for trace()
        (self.cycles, self.scanlines)
    }

    fn _sprite_0_hit(&self) -> bool {
        let sprite_palettes = self.fb.get_palette(&self.palette_table, 0x10);

        let sprite_tile_id = self.oam_data[1] as usize;
        let sprite_attr = self.oam_data[2];
        // let (scroll_x, scroll_y) = self.get_scroll_offset();
        let sprite_y = self.oam_data[0] as usize;
        let sprite_x = self.oam_data[3] as usize;

        let get_bg_color_id = |x: usize, y: usize| -> usize {
            let x = x % (WIDTH * 2);
            let y = y % (HEIGHT * 2);

            let x_8 = x / 8;
            let y_8 = y / 8;

            // todo support vertical scroll
            let (vram, vram_offset) = match (x < WIDTH, y < HEIGHT) {
                (true, true) => (&self.vram[0..0x400], y_8 * 32 + x_8),
                (true, false) => todo!(),
                (false, true) => (&self.vram[0x400..0x800], y_8 * 32 + x_8 - 32),
                (false, false) => todo!(),
            };

            let tile_id = vram[vram_offset] as usize;
            let chr_rom = &self.chr_rom[self.get_bg_chr_rom_range()];

            let x_lsb3 = x & 0b111;
            let y_lsb3 = y & 0b111;
            let hi = chr_rom[tile_id * 16 + y_lsb3] as usize;
            let lo = chr_rom[tile_id * 16 + y_lsb3 + 8] as usize;
            let mask = 0b1000_0000 >> x_lsb3;
            let color_index = (((lo & mask) << 1) | (hi & mask)) as usize >> (0b111 - x_lsb3);
            let palette = self.fb.get_bg_tile_palette(x_8, y_8, vram, &self.palette_table);
            palette[color_index]
        };

        let bg0_id = self.palette_table[0] as usize;

        let sprite_palette = sprite_palettes[(sprite_attr & 0b11) as usize];
        let sprite_flip_h = sprite_attr & 0b0100_0000 > 0;
        let sprite_flip_v = sprite_attr & 0b1000_0000 > 0;
        let sprite_chr_rom = &self.chr_rom[self.get_sprite_chr_rom_range()];
        let sprite_tile = &sprite_chr_rom[sprite_tile_id*16..=sprite_tile_id*16+15];

        for y in 0..=7 {
            let y = if sprite_flip_h { 7 - y } else { y };
            if self.scanlines != sprite_y + y {
                continue;
            }
            let mut sprite_hi = sprite_tile[y];
            let mut sprite_lo = sprite_tile[y + 8];
            for x in (0..=7).rev() {
                let sprite_color_index = ((sprite_lo & 1) << 1) | (sprite_hi & 1);
                sprite_hi >>= 1;
                sprite_lo >>= 1;

                let x = if sprite_flip_v { 7 - x } else { x };
                if self.cycles <= sprite_x + x {
                    continue;
                }

                let sprite_color_id = sprite_palette[sprite_color_index as usize];
                let bg_color_id = get_bg_color_id(sprite_x + x, sprite_y + y);

                if sprite_color_id != bg0_id && bg_color_id != bg0_id {
                    return true;
                }
            }
        }

        return false;
    }

    fn sprite_0_hit_rough(&self) -> bool {
        let sprite_y = self.oam_data[0] as usize;
        let sprite_x = self.oam_data[3] as usize;
        (sprite_y == self.scanlines as usize)
        && sprite_x <= self.cycles
        && self.reg.mask.contains(MaskRegister::SHOW_SPRITES)
    }


    fn get_scroll_offset(&self) -> (usize, usize) {
        let scroll = self.reg.scrl.get();
        let mut x = scroll.0 as usize;
        let mut y = scroll.1 as usize;

        if self.reg.internal_v.nametable_x {
            x += WIDTH;
        }
        if self.reg.internal_v.nametable_y {
            y += HEIGHT;
        }

        (x, y)
    }

    fn get_sprite_chr_rom_range(&self) -> std::ops::Range<usize>   {
        let sprite_pattern_addr = self.reg.ctrl & ControlRegister::SPRITE_PATTERN_ADDR;
        let offset = 512 * sprite_pattern_addr.bits() as usize;
        offset..offset+0x1000
    }

    fn get_bg_chr_rom_range(&self) -> std::ops::Range<usize>   {
        let bg_pattern_addr = self.reg.ctrl & ControlRegister::BACKROUND_PATTERN_ADDR;
        let offset = 256 * bg_pattern_addr.bits() as usize;
        offset..offset+0x1000
    }

    fn increment_x(&mut self) {
        let v = &mut self.reg.internal_v;
        if v.coarse_x == 31 {
            v.coarse_x = 0;
            v.nametable_x = !v.nametable_x;
        } else {
            v.coarse_x += 1;
        }    
    }

    fn increment_y(&mut self) {
        let v = &mut self.reg.internal_v;
        if v.fine_y < 7 {
            v.fine_y += 1;
        } else {
            v.fine_y = 0;
            if v.coarse_y == 29 {
                v.coarse_y = 0;
                v.nametable_y = !v.nametable_y;           
            } else {
                v.coarse_y += 1;
            }
        }
    }

    fn tick_single(&mut self) -> TickResult {
        let show_sprites = self.reg.mask.contains(MaskRegister::SHOW_SPRITES);
        let show_bg = self.reg.mask.contains(MaskRegister::SHOW_BG);
        if (show_sprites || show_bg) && (0..=239).contains(&self.scanlines) {
            if show_sprites && !self.reg.stat.contains(StatusRegister::SPRITE_0_HIT) && self.sprite_0_hit_rough() {
                self.reg.stat.set(StatusRegister::SPRITE_0_HIT, true);
            }

            if (1..=256).contains(&self.cycles) || (327..).contains(&self.cycles) {
                if self.cycles & 0b111 == 0b111 && self.cycles != 255 {
                    self.increment_x();
                }
                if self.cycles == 256 {
                    self.increment_y();
                }
            } else if self.cycles == 257 {
                // copy x
                let v = &mut self.reg.internal_v;
                let t = &mut self.reg.internal_t;
                v.nametable_x = t.nametable_x;
                v.coarse_x = t.coarse_x;
            }
        } else if self.scanlines == 261 {
            if (1..=256).contains(&self.cycles) || (328..).contains(&self.cycles) {
                if self.cycles & 0b111 == 0b111 && self.cycles != 255 {
                    self.increment_x();
                }
                if self.cycles == 256 {
                    self.increment_y();
                }
            } else if self.cycles == 257 {
                // copy x
                let v = &mut self.reg.internal_v;
                let t = &mut self.reg.internal_t;
                v.nametable_x = t.nametable_x;
                v.coarse_x = t.coarse_x;
            } else if (280..=340).contains(&self.cycles) {
                let v = &mut self.reg.internal_v;
                let t = &mut self.reg.internal_t;
                v.nametable_y = t.nametable_y;
                v.coarse_y = t.coarse_y;
                v.fine_y = t.fine_y;
            }
        }

        self.cycles += 1;

        if self.cycles == 341 {

            self.scanlines += 1;
            self.cycles -= 341;

            // Render background every 8 lines except 0
            let show_bg = self.reg.mask.contains(MaskRegister::SHOW_BG);
            if show_bg && (1..=240).contains(&self.scanlines) && self.scanlines & 0b111 == 0 {
                let row_number = (self.scanlines - 1) / 8;
                let range = self.get_bg_chr_rom_range();
                let chr_rom_slice = &self.chr_rom[range];

                // Draw two screens(rows) for games using PPU scroll
                let vram_slice_a = &self.vram[0..0x400];
                let vram_slice_b = &self.vram[0x400..0x800];
                self.fb.render_bg_row(row_number, 0, 0, chr_rom_slice, &self.palette_table, vram_slice_a);
                match self.mirroring {
                    Mirroring::Invalid => todo!(),
                    Mirroring::Vertical => {
                        self.fb.render_bg_row(row_number, WIDTH, 0, chr_rom_slice, &self.palette_table, vram_slice_b);
                    },
                    Mirroring::Horizontal => {
                        self.fb.render_bg_row(row_number, 0, HEIGHT, chr_rom_slice, &self.palette_table, vram_slice_b);
                    },
                    Mirroring::FourScreen => todo!(),
                }

                // Copy current bg into viewport based on the value of scroll register and base nametable
                let (x, y) = self.get_scroll_offset();
                self.fb.copy_to_viewport(row_number, x, y);
            }

            // Render sprites at the end of visible scanlines
            if self.scanlines == 241 {
                if show_sprites {
                    let range = self.get_sprite_chr_rom_range();
                    self.fb.render_sprites(&self.chr_rom[range], &self.palette_table, &self.oam_data);
                }

                self.reg.stat.set(StatusRegister::VBLANK_STARTED, true);
                // self.reg.stat.set(StatusRegister::SPRITE_0_HIT, false);
                if self.reg.ctrl.enable_generage_nmi() {
                    return TickResult::ShouldInterruptNmiAndUpdateTexture;
                } else {
                    return TickResult::ShouldUpdateTexture;
                }
            }

            // Start over
            if self.scanlines == 262 {
                self.scanlines = 0;
                self.reg.stat.set(StatusRegister::VBLANK_STARTED, false);
                self.reg.stat.set(StatusRegister::SPRITE_0_HIT, false);
                return TickResult::ScanlineReset;
            }
        }

        return TickResult::None;
    }

    pub fn tick(&mut self, cycles: u8) -> Vec<TickResult> {
        (0..cycles).map(|_| self.tick_single()).collect()
    }

    fn is_vblank(&self) -> bool {
        !(0..=239).contains(&self.scanlines)
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
            _ => panic!()
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
        self.oam_data[self.reg.oam_addr as usize] = data;
        self.reg.oam_addr = self.reg.oam_addr.wrapping_add(1);
    }

    pub fn write_scrl(&mut self, data: u8) {
        self.reg.scrl.update(data);
    }

    pub fn write_ctrl(&mut self, data: u8) -> TickResult {
        let before_nmi_status = self.reg.ctrl.contains(ControlRegister::GENERATE_NMI);
        self.reg.ctrl.update(data);
        let after_nmi_status = self.reg.ctrl.contains(ControlRegister::GENERATE_NMI);

        self.reg.internal_t.nametable_x = self.reg.ctrl.contains(ControlRegister::NAMETABLE_X);
        self.reg.internal_t.nametable_y = self.reg.ctrl.contains(ControlRegister::NAMETABLE_Y);

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


#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn test_ppu_vram_writes() {
        let mut ppu = Ppu::new_test_vertical();
        ppu.write_addr(0x23);
        ppu.write_addr(0x05);
        ppu.write_data(0x66);

        assert_eq!(ppu.vram[0x0305], 0x66);
    }

    #[test]
    fn test_ppu_vram_reads() {
        let mut ppu = Ppu::new_test_vertical();
        ppu.write_ctrl(0);
        ppu.vram[0x0305] = 0x66;

        ppu.write_addr(0x23);
        ppu.write_addr(0x05);

        ppu.read_data(false); //load_into_buffer
        assert_eq!(ppu.reg.addr.get(), 0x2306);
        assert_eq!(ppu.read_data(false), 0x66);
    }

    #[test]
    fn test_ppu_vram_reads_cross_page() {
        let mut ppu = Ppu::new_test_vertical();
        ppu.write_ctrl(0);
        ppu.vram[0x01ff] = 0x66;
        ppu.vram[0x0200] = 0x77;

        ppu.write_addr(0x21);
        ppu.write_addr(0xff);

        ppu.read_data(false); //load_into_buffer
        assert_eq!(ppu.read_data(false), 0x66);
        assert_eq!(ppu.read_data(false), 0x77);
    }

    #[test]
    fn test_ppu_vram_reads_step_32() {
        let mut ppu = Ppu::new_test_vertical();
        ppu.write_ctrl(0b100);
        ppu.vram[0x01ff] = 0x66;
        ppu.vram[0x01ff + 32] = 0x77;
        ppu.vram[0x01ff + 64] = 0x88;

        ppu.write_addr(0x21);
        ppu.write_addr(0xff);

        ppu.read_data(false); //load_into_buffer
        assert_eq!(ppu.read_data(false), 0x66);
        assert_eq!(ppu.read_data(false), 0x77);
        assert_eq!(ppu.read_data(false), 0x88);
    }

    // Horizontal: https://wiki.nesdev.com/w/index.php/Mirroring
    //   [0x2000 A ] [0x2400 a ]
    //   [0x2800 B ] [0x2C00 b ]
    // #[test]
    // fn test_vram_horizontal_mirror() {
    //     let mut ppu = Ppu::new_test_vertical();
    //     ppu.write_addr(0x24);
    //     ppu.write_addr(0x05);

    //     ppu.write_data(0x66); //write to a

    //     ppu.write_addr(0x28);
    //     ppu.write_addr(0x05);

    //     ppu.write_data(0x77); //write to B

    //     ppu.write_addr(0x20);
    //     ppu.write_addr(0x05);

    //     ppu.read_data(false); //load into buffer
    //     assert_eq!(ppu.read_data(false), 0x66); //read from A

    //     ppu.write_addr(0x2C);
    //     ppu.write_addr(0x05);

    //     ppu.read_data(false); //load into buffer
    //     assert_eq!(ppu.read_data(false), 0x77); //read from b
    // }

    // Vertical: https://wiki.nesdev.com/w/index.php/Mirroring
    //   [0x2000 A ] [0x2400 B ]
    //   [0x2800 a ] [0x2C00 b ]
    #[test]
    fn test_vram_vertical_mirror() {
        let mut ppu = Ppu::new_test_vertical();

        ppu.write_addr(0x20);
        ppu.write_addr(0x05);

        ppu.write_data(0x66); //write to A

        ppu.write_addr(0x2C);
        ppu.write_addr(0x05);

        ppu.write_data(0x77); //write to b

        ppu.write_addr(0x28);
        ppu.write_addr(0x05);

        ppu.read_data(false); //load into buffer
        assert_eq!(ppu.read_data(false), 0x66); //read from a

        ppu.write_addr(0x24);
        ppu.write_addr(0x05);

        ppu.read_data(false); //load into buffer
        assert_eq!(ppu.read_data(false), 0x77); //read from B
    }

    #[test]
    fn test_read_stat_resets_latch() {
        let mut ppu = Ppu::new_test_vertical();
        ppu.vram[0x0305] = 0x66;

        ppu.write_addr(0x21);
        ppu.write_addr(0x23);
        ppu.write_addr(0x05);

        ppu.read_data(false); //load_into_buffer
        assert_ne!(ppu.read_data(false), 0x66);

        ppu.read_stat(false);

        ppu.write_addr(0x23);
        ppu.write_addr(0x05);

        ppu.read_data(false); //load_into_buffer
        assert_eq!(ppu.read_data(false), 0x66);
    }

    #[test]
    fn test_ppu_vram_mirroring() {
        let mut ppu = Ppu::new_test_vertical();
        ppu.write_ctrl(0);
        ppu.vram[0x0305] = 0x66;

        ppu.write_addr(0x63); //0x6305 -> 0x2305
        ppu.write_addr(0x05);

        ppu.read_data(false); //load into_buffer
        assert_eq!(ppu.read_data(false), 0x66);
        // assert_eq!(ppu.addr.read(), 0x0306)
    }

    #[test]
    fn test_read_stat_resets_vblank() {
        let mut ppu = Ppu::new();
        ppu.reg.stat.set(StatusRegister::VBLANK_STARTED, true);

        let status = ppu.read_stat(false);

        assert_eq!(status >> 7, 1);
        assert_eq!(ppu.reg.stat.bits() >> 7, 0);
    }

    #[test]
    fn test_oam_read_write() {
        let mut ppu = Ppu::new();
        ppu.write_oam_addr(0x10);
        ppu.write_oam_data(0x66);
        ppu.write_oam_data(0x77);

        ppu.write_oam_addr(0x10);
        assert_eq!(ppu.read_oam_data(), 0x66);

        ppu.write_oam_addr(0x11);
        assert_eq!(ppu.read_oam_data(), 0x77);
    }

}