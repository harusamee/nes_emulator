use cartridge::Cartridge;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;

#[rustfmt::skip]
static SYSTEM_PALLETE: [(u8,u8,u8); 64] = [
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

static WIDTH: usize = 256;
static HEIGHT: usize = 240;
static SCALE: f32 = 3.0;

fn set_pixel(fb: &mut Vec<u8>, x: usize, y: usize, p: usize) {
    let (r, g, b) = SYSTEM_PALLETE[p];
    let offset = (y * (WIDTH * 2) + x) * 3;
    if offset + 2 < fb.len() {
        fb[offset] = r;
        fb[offset + 1] = g;
        fb[offset + 2] = b;
    }
}

fn render_tile(chr_rom: &Vec<u8>, fb: &mut Vec<u8>, bank: usize, tile_id: usize) {
    let tile_start = bank * 0x1000 + tile_id * 16;
    let tile_end = tile_start + 15;
    let tile = &chr_rom[tile_start..=tile_end];

    let offset_x = (tile_id % 28) * 9;
    let offset_y = (tile_id / 28) * 9;

    for y in 0..=7 {
        let mut hi = tile[y];
        let mut lo = tile[y + 8];

        for x in (0..=7).rev() {
            let color_index = (hi & 1 << 1) | (lo & 1);
            let pallete: usize = match color_index {
                0 => 0x01,
                1 => 0x23,
                2 => 0x27,
                3 => 0x30,
                _ => panic!()
            };

            set_pixel(fb, offset_x + x, offset_y + y, pallete);

            hi >>= 1;
            lo >>= 1;
        }
    }
}

pub fn chr_rom_viewer(args: Vec<String>) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("chr_rom_viewer", (WIDTH as f32 * SCALE) as u32, (HEIGHT as f32 * SCALE) as u32)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    canvas.set_scale(SCALE, SCALE).unwrap();

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, WIDTH as u32, HEIGHT as u32)
        .unwrap();

    let mut fb = vec![0u8; WIDTH * HEIGHT * 3];

    let filename = &args[2];
    let raw = std::fs::read(filename).expect("Could not open file");
    let cartridge = Cartridge::load(&raw).expect("Invalid cartridge");

    println!("chr rom size: 0x{:04X}", cartridge.chr_rom.len());

    for tile_id in 0..=0x100 {
        render_tile(&cartridge.chr_rom, &mut fb, 0, tile_id);
    }

    texture.update(None, &fb, WIDTH * 3).unwrap();
    canvas.copy(&texture, None, None).unwrap();
    canvas.present();

    loop {
        for event in event_pump.poll_iter() {
           match event {
             Event::Quit { .. }
             | Event::KeyDown {
                 keycode: Some(Keycode::Escape),
                 ..
             } => std::process::exit(0),
             _ => { /* do nothing */ }
           }
        }
     }
}