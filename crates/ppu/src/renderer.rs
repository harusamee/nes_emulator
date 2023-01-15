use bitflags::bitflags;

use super::{HEIGHT, WIDTH};

type RGB = (u8, u8, u8);
#[rustfmt::skip]
static SYSTEM_PALLETE: [RGB; 64] = [
   (0x80, 0x80, 0x80), (0x00, 0x3D, 0xA6), (0x00, 0x12, 0xB0), (0x44, 0x00, 0x96), (0xA1, 0x00, 0x5E),
   (0xC7, 0x00, 0x28), (0xBA, 0x06, 0x00), (0x8C, 0x17, 0x00), (0x5C, 0x2F, 0x00), (0x10, 0x45, 0x00),
   (0x05, 0x4A, 0x00), (0x00, 0x47, 0x2E), (0x00, 0x41, 0x66), (0x00, 0x00, 0x00), (0x05, 0x05, 0x05),
   (0x05, 0x05, 0x05), (0xC7, 0xC7, 0xC7), (0x00, 0x77, 0xFF), (0x21, 0x55, 0xFF), (0x82, 0x37, 0xFA),
   (0xEB, 0x2F, 0xB5), (0xFF, 0x29, 0x50), (0xFF, 0x22, 0x00), (0xD6, 0x32, 0x00), (0xC4, 0x62, 0x00),
   (0x35, 0x80, 0x00), (0x05, 0x8F, 0x00), (0x00, 0x8A, 0x55), (0x00, 0x99, 0xCC), (0x21, 0x21, 0x21),
   (0x09, 0x09, 0x09), (0x09, 0x09, 0x09), (0xFF, 0xFF, 0xFF), (0x0F, 0xD7, 0xFF), (0x69, 0xA2, 0xFF),
   (0xD4, 0x80, 0xFF), (0xFF, 0x45, 0xF3), (0xFF, 0x61, 0x8B), (0xFF, 0x88, 0x33), (0xFF, 0x9C, 0x12),
   (0xFA, 0xBC, 0x20), (0x9F, 0xE3, 0x0E), (0x2B, 0xF0, 0x35), (0x0C, 0xF0, 0xA4), (0x05, 0xFB, 0xFF),
   (0x5E, 0x5E, 0x5E), (0x0D, 0x0D, 0x0D), (0x0D, 0x0D, 0x0D), (0xFF, 0xFF, 0xFF), (0xA6, 0xFC, 0xFF),
   (0xB3, 0xEC, 0xFF), (0xDA, 0xAB, 0xEB), (0xFF, 0xA8, 0xF9), (0xFF, 0xAB, 0xB3), (0xFF, 0xD2, 0xB0),
   (0xFF, 0xEF, 0xA6), (0xFF, 0xF7, 0x9C), (0xD7, 0xE8, 0x95), (0xA6, 0xED, 0xAF), (0xA2, 0xF2, 0xDA),
   (0x99, 0xFF, 0xFC), (0xDD, 0xDD, 0xDD), (0x11, 0x11, 0x11), (0x11, 0x11, 0x11)
];

bitflags! {
    struct RenderMode: u8 {
        const IGNORE_BG_COLOR0 = 0b0000_0001;
        const FLIP_HORIZONTAL = 0b0000_0010;
        const FLIP_VERTICAL = 0b0000_0100;
        const BEHIND_BG = 0b0000_1000;
    }
}

enum TargetBuffer {
    Framebuffer,
    Viewport
}

pub struct Renderer {
    // The buffer for background tiles of four nametables
    fb: Vec<u8>,
    // Actual viewport set by the scroll register
    viewport: Vec<u8>,
    draw_leftmost_bg: bool,
    draw_leftmost_sprites: bool,
    enabled: bool
}

impl Renderer {
    pub fn new() -> Self {
        Renderer {
            fb: vec![0u8; WIDTH * HEIGHT * 3 * 4],
            viewport: vec![0u8; WIDTH * HEIGHT * 3],
            draw_leftmost_bg: false,
            draw_leftmost_sprites: false,
            enabled: true
        }
    }

    pub fn set_draw_leftmost(&mut self, bg: bool, sprite: bool) {
        self.draw_leftmost_bg = bg;
        self.draw_leftmost_sprites = sprite;
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn set_pixel(&mut self, target: &TargetBuffer, x: usize, y: usize, rgb: RGB) {
        let (r, g, b) = rgb;
        let (target, pitch) = match target {
            TargetBuffer::Framebuffer => (&mut self.fb, WIDTH * 2 * 3),
            TargetBuffer::Viewport => (&mut self.viewport, WIDTH * 3),
        };
        let offset = y * pitch + x * 3;

        if offset + 2 < target.len() {
            target[offset] = r;
            target[offset + 1] = g;
            target[offset + 2] = b;
        }
    }

    fn render_tile(
        &mut self,
        target: &TargetBuffer,
        chr_rom: &[u8],
        tile_id: usize,
        offset_x: usize,
        offset_y: usize,
        color_id_list: [usize; 4],
        bg0_id: usize,
        mode: RenderMode,
    ) {
        let is_bg = !mode.contains(RenderMode::IGNORE_BG_COLOR0);

        if is_bg && (offset_x >= WIDTH * 2 || offset_y >= HEIGHT * 2) {
            return;
        }

        let tile_start = tile_id * 16;
        let tile_end = tile_start + 15;
        let tile = &chr_rom[tile_start..=tile_end];
        
        for y in 0..=7 {
            let mut hi = tile[y];
            let mut lo = tile[y + 8];

            for x in (0..=7).rev() {
                let color_index = ((lo & 1) << 1) | (hi & 1);
                let color_id = color_id_list[color_index as usize];

                hi >>= 1;
                lo >>= 1;

                if is_bg {
                    if !self.draw_leftmost_bg && offset_x == 0 {
                        continue;
                    }
                } else {
                    // Sprite
                    if color_id == bg0_id {
                        continue;
                    }
                    if mode.contains(RenderMode::BEHIND_BG) {
                        continue;
                    }
                    if !self.draw_leftmost_sprites && offset_x == 0 {
                        continue;
                    }
                }

                let x = offset_x + if mode.contains(RenderMode::FLIP_HORIZONTAL) { 7 - x } else { x };
                let y = offset_y + if mode.contains(RenderMode::FLIP_VERTICAL) { 7 - y } else { y };
                let rgb = SYSTEM_PALLETE[color_id as usize];
                self.set_pixel(target, x, y, rgb);
            }
        }
    }

    fn get_palette(&self, palette_table: &[u8; 32], offset: usize) -> Vec<[usize; 4]> {
        vec![
            [
                palette_table[0] as usize,
                palette_table[offset + 1] as usize,
                palette_table[offset + 2] as usize,
                palette_table[offset + 3] as usize,
            ],
            [
                palette_table[0] as usize,
                palette_table[offset + 5] as usize,
                palette_table[offset + 6] as usize,
                palette_table[offset + 7] as usize,
            ],
            [
                palette_table[0] as usize,
                palette_table[offset + 9] as usize,
                palette_table[offset + 10] as usize,
                palette_table[offset + 11] as usize,
            ],
            [
                palette_table[0] as usize,
                palette_table[offset + 13] as usize,
                palette_table[offset + 14] as usize,
                palette_table[offset + 15] as usize,
            ],
        ]
    }

    pub fn render_bg_row(
        &mut self,
        row_number: usize,
        offset_x: usize,
        offset_y: usize,
        chr_rom: &[u8],
        palette_table: &[u8; 32],
        vram: &[u8],
    ) {
        if !self.enabled {
            return;
        }

        let palettes = self.get_palette(palette_table, 0);

        let tile_start = row_number * WIDTH / 8;
        let tile_end = tile_start + WIDTH / 8;
        let tiles = &vram[tile_start..tile_end];

        let attr_start = 0x3c0;

        for (x, tile_id) in tiles.iter().enumerate() {
            let x_4 = x / 4;
            let y_4 = row_number / 4;

            let attr_index = attr_start + y_4 * 8 + x_4;
            let attr_data = vram[attr_index];

            // Which position of attr_data should we use?
            let attr_data_shifts = ((row_number & 0b10) | ((x & 0b10) >> 1)) * 2;
            // Get two bits of attr_data as palette id
            let palette_id = ((attr_data >> attr_data_shifts) & 0b11) as usize;
            let palette = palettes[palette_id];

            self.render_tile(
                &TargetBuffer::Framebuffer,
                chr_rom,
                (*tile_id).into(),
                offset_x + x * 8,
                offset_y + row_number * 8,
                palette,
                palette_table[0] as usize,
                RenderMode::from_bits_truncate(0),
            );
        }
    }

    pub fn render_sprites(
        &mut self,
        sprite_8x16: bool,
        chr_rom: &[u8],
        palette_table: &[u8; 32],
        oam_data: &[u8; 256],
    ) {
        if !self.enabled {
            return;
        }

        let palettes = self.get_palette(palette_table, 0x10);

        for i in (0..oam_data.len()).step_by(4) {
            match oam_data[i..i + 4] {
                [y, tile_id, attr, x] => {
                    let palette = palettes[(attr & 0b11) as usize];

                    let mut mode = RenderMode::IGNORE_BG_COLOR0;
                    let priority = attr & 0b0010_0000 > 0;
                    let flip_h = attr & 0b0100_0000 > 0;
                    let flip_v = attr & 0b1000_0000 > 0;
                    mode.set(RenderMode::FLIP_HORIZONTAL, flip_h);
                    mode.set(RenderMode::FLIP_VERTICAL, flip_v);
                    mode.set(RenderMode::BEHIND_BG, priority);

                    self.render_tile(
                        &TargetBuffer::Viewport,
                        chr_rom,
                        tile_id as usize,
                        x as usize,
                        y as usize,
                        palette,
                        palette_table[0] as usize,
                        mode,
                    );
                }
                _ => panic!(),
            }
        }
    }

    fn copy_to_viewport_impl(&mut self, row_number: usize, fb_offset_xy: (usize,usize), vp_offset_xy: (usize,usize), max_width: usize) {
        const PITCH_FB: usize = WIDTH * 3 * 2;
        const PITCH_VP: usize = WIDTH * 3;

        let (fb_offset_x, fb_offset_y) = fb_offset_xy;
        let (vp_offset_x, vp_offset_y) = vp_offset_xy;

        let fb_offset_base = PITCH_FB * (fb_offset_y + row_number * 8) + fb_offset_x * 3;
        let vp_offset_base = PITCH_VP * (vp_offset_y + row_number * 8) + vp_offset_x * 3;
        // todo support horizontal scroll
        let need_copy_twice = fb_offset_x > WIDTH;
        let mut width = match need_copy_twice {
            true => WIDTH * 2 - fb_offset_x,
            false => WIDTH,
        };

        if width > max_width {
            width = max_width;
        }

        for i in 0..=7 {
            let fb_offset = fb_offset_base + (i * PITCH_FB);
            let vp_offset = vp_offset_base + (i * PITCH_VP);

            self.viewport[vp_offset..vp_offset+width*3].copy_from_slice(
                &self.fb[fb_offset..fb_offset+width*3]);
        }

        if need_copy_twice {
            let max_width = fb_offset_x - WIDTH;
            self.copy_to_viewport_impl(row_number, (0, fb_offset_y), (width, 0), max_width);
        }
    }

    pub fn copy_to_viewport(&mut self, row_number: usize, offset_x: usize, offset_y: usize) {
        self.copy_to_viewport_impl(row_number, (offset_x, offset_y), (0, 0), WIDTH);
    }

    pub fn get_buffer(&self) -> &Vec<u8> {
        &self.viewport
    }
}
