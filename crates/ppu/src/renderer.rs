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

pub struct Renderer {
    fb: Vec<u8>,
}

impl Renderer {
    pub fn new() -> Self {
        Renderer {
            fb: vec![0u8; WIDTH * HEIGHT * 3],
        }
    }

    fn set_pixel(&mut self, x: usize, y: usize, palette: RGB) {
        let (r, g, b) = palette;
        let offset = (y * WIDTH + x) * 3;
        if offset + 2 < self.fb.len() {
            self.fb[offset] = r;
            self.fb[offset + 1] = g;
            self.fb[offset + 2] = b;
        }
    }

    pub fn render_tile(
        &mut self,
        chr_rom: &Vec<u8>,
        bank: usize,
        tile_id: usize,
        offset_x: usize,
        offset_y: usize,
        palette: [RGB; 4],
        is_sprite: bool
    ) {
        let tile_start = bank * 0x1000 + tile_id * 16;
        let tile_end = tile_start + 15;
        let tile = &chr_rom[tile_start..=tile_end];

        for y in 0..=7 {
            let mut hi = tile[y];
            let mut lo = tile[y + 8];

            for x in (0..=7).rev() {
                let color_index = ((lo & 1) << 1) | (hi & 1);
                if is_sprite && color_index == 0 {
                    continue;
                }
                let palette = palette[color_index as usize];
                self.set_pixel(offset_x + x, offset_y + y, palette);

                hi >>= 1;
                lo >>= 1;
            }
        }
    }

    pub fn render_bg_row(
        &mut self,
        row_number: usize,
        offset: usize,
        chr_rom: &Vec<u8>,
        palette_table: &[u8; 32],
        vram: &[u8; 2048],
    ) {
        let tile_start = offset + row_number * WIDTH / 8;
        let tile_end = tile_start + WIDTH / 8;
        let tiles = &vram[tile_start..tile_end];

        let attr_start = offset + 0x3c0;

        let palettes = vec![
            [
                SYSTEM_PALLETE[palette_table[0] as usize],
                SYSTEM_PALLETE[palette_table[1] as usize],
                SYSTEM_PALLETE[palette_table[2] as usize],
                SYSTEM_PALLETE[palette_table[3] as usize],
            ],
            [
                SYSTEM_PALLETE[palette_table[0] as usize],
                SYSTEM_PALLETE[palette_table[5] as usize],
                SYSTEM_PALLETE[palette_table[6] as usize],
                SYSTEM_PALLETE[palette_table[7] as usize],
            ],
            [
                SYSTEM_PALLETE[palette_table[0] as usize],
                SYSTEM_PALLETE[palette_table[9] as usize],
                SYSTEM_PALLETE[palette_table[10] as usize],
                SYSTEM_PALLETE[palette_table[11] as usize],
            ],
            [
                SYSTEM_PALLETE[palette_table[0] as usize],
                SYSTEM_PALLETE[palette_table[13] as usize],
                SYSTEM_PALLETE[palette_table[14] as usize],
                SYSTEM_PALLETE[palette_table[15] as usize],
            ],
        ];

        // println!("{} {:04X}-{:04X} {:?}", row_number, tile_start, tile_end, tiles);

        for (x, tile_id) in tiles.iter().enumerate() {
            let x_4 = x / 4;
            let y_4 = row_number / 4;

            let attr_index = attr_start + y_4 * 8 + x_4;
            let attr_data = vram[attr_index];

            // Which position of attr_data should we use?
            let attr_data_shifts = ((row_number & 0b10) | (x & 0b10 >> 1)) * 2;
            // Get two bits of attr_data as palette id
            let palette_id = ((attr_data >> attr_data_shifts) & 0b11) as usize;
            let palette = palettes[palette_id];

            self.render_tile(
                chr_rom,
                0, // todo
                (*tile_id).into(),
                x * 8,
                row_number * 8,
                palette,
                false
            );
        }
    }

    pub fn render_sprites(
        &mut self,
        offset: u8,
        sprite_8x16: bool,
        chr_rom: &Vec<u8>,
        palette_table: &[u8; 32],
        vram: &[u8; 2048],
        oam_data: &[u8; 256],
    ) {
        let palettes = vec![
            [
                SYSTEM_PALLETE[palette_table[0] as usize],
                SYSTEM_PALLETE[palette_table[0x11] as usize],
                SYSTEM_PALLETE[palette_table[0x12] as usize],
                SYSTEM_PALLETE[palette_table[0x13] as usize],
            ],
            [
                SYSTEM_PALLETE[palette_table[0] as usize],
                SYSTEM_PALLETE[palette_table[0x15] as usize],
                SYSTEM_PALLETE[palette_table[0x16] as usize],
                SYSTEM_PALLETE[palette_table[0x17] as usize],
            ],
            [
                SYSTEM_PALLETE[palette_table[0] as usize],
                SYSTEM_PALLETE[palette_table[0x19] as usize],
                SYSTEM_PALLETE[palette_table[0x1a] as usize],
                SYSTEM_PALLETE[palette_table[0x1b] as usize],
            ],
            [
                SYSTEM_PALLETE[palette_table[0] as usize],
                SYSTEM_PALLETE[palette_table[0x1d] as usize],
                SYSTEM_PALLETE[palette_table[0x1e] as usize],
                SYSTEM_PALLETE[palette_table[0x1f] as usize],
            ],
        ];

        for i in (0..oam_data.len()).step_by(4) {
            match oam_data[i..i+4] {
                [y, tile_id, attr, x] => {
                    let palette = palettes[(attr & 0b11) as usize];
                    self.render_tile(chr_rom, 1, tile_id as usize, x as usize, y as usize, palette, true);
                }
                _ => panic!()
            }
        }
    }

    pub fn get_buffer(&self) -> &Vec<u8> {
        &self.fb
    }
}
