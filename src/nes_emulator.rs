use std::collections::HashMap;

use cpu::Cpu;
use ppu::{HEIGHT, WIDTH};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::EventPump;

use lazy_static::lazy_static;

const SCALE: usize = 3;

lazy_static! {
    static ref KEY_MAP: HashMap<Keycode, joypad::JoypadButton> = HashMap::from([
        (Keycode::Down, joypad::JoypadButton::DOWN),
        (Keycode::Up, joypad::JoypadButton::UP),
        (Keycode::Right, joypad::JoypadButton::RIGHT),
        (Keycode::Left, joypad::JoypadButton::LEFT),~
        (Keycode::Space, joypad::JoypadButton::SELECT),
        (Keycode::Return, joypad::JoypadButton::START),
        (Keycode::A, joypad::JoypadButton::BUTTON_A),
        (Keycode::S, joypad::JoypadButton::BUTTON_B)
    ]);
}

fn handle_user_input(cpu: &mut Cpu, event_pump: &mut EventPump) {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => std::process::exit(0),
            Event::KeyDown { keycode, .. } => {
                let keycode = keycode.unwrap_or(Keycode::Ampersand);
                if let Some(key) = KEY_MAP.get(&keycode) {
                    cpu.bus.joypad1.set_button_status(key, true);
                }
            }
            Event::KeyUp { keycode, .. } => {
                let keycode = keycode.unwrap_or(Keycode::Ampersand);
                if let Some(key) = KEY_MAP.get(&keycode) {
                    cpu.bus.joypad1.set_button_status(key, false);
                }
            }
            _ => { /* do nothing */ }
        }
    }
}

pub fn nes_emulator(args: Vec<String>) {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window(
            "nes_emulator",
            (WIDTH * SCALE) as u32,
            (HEIGHT * SCALE) as u32,
        )
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();
    canvas.set_scale(SCALE as f32, SCALE as f32).unwrap();

    let creator = canvas.texture_creator();
    let mut texture = creator
        .create_texture_target(PixelFormatEnum::RGB24, WIDTH as u32, HEIGHT as u32)
        .unwrap();

    let mut cpu = Cpu::new();

    let filename = if args.len() >= 2 {
        &args[1]
    } else {
        "pacman.nes"
    };
    let raw = std::fs::read(filename).expect("Could not read the file");
    cpu.bus.load_cartridge(&raw);
    let vector = cpu.bus.read16(0xfffc);
    cpu.set_pc(vector);

    canvas.present();

    let mut prev_line = String::new();
    let mut same_count = 0;
    cpu.run_with_callback(
        move |cpu| {
            // let line = cpu.trace();
            // print!("{}", line);
            // if prev_line == line {
            //     same_count += 1;
            //     print!(" x {} \r", same_count);
            // } else {
            //     print!("\r\n");
            //     same_count = 0;
            // }
            // prev_line = line;
            handle_user_input(cpu, &mut event_pump);
        },
        |cpu| {
            cpu.bus.ppu.update_sdl_texture(&mut texture);
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();
        },
    );
}
