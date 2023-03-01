use core::panic;
use std::collections::HashMap;
use std::thread::{sleep_ms, sleep, current};
use std::time::{Instant, Duration};

use cartridge::Cartridge;
use cpu::Cpu;
use ppu::{HEIGHT, WIDTH};
use joypad::JoypadButton;
use apu::{Apu, init_apu};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use sdl2::EventPump;

use lazy_static::lazy_static;

struct Settings {
    trace: bool,
    wait: bool
}

#[derive(Clone, Copy)]
enum Action {
    Joypad(joypad::JoypadButton),
    ToggleTrace,
    ToggleFrameWait,
    None
}

lazy_static! {
    static ref KEY_MAP: HashMap<Keycode, Action> = HashMap::from([
        (Keycode::Down, Action::Joypad(JoypadButton::DOWN)),
        (Keycode::Up, Action::Joypad(JoypadButton::UP)),
        (Keycode::Right, Action::Joypad(JoypadButton::RIGHT)),
        (Keycode::Left, Action::Joypad(JoypadButton::LEFT)),
        (Keycode::Space, Action::Joypad(JoypadButton::SELECT)),
        (Keycode::Return, Action::Joypad(JoypadButton::START)),
        (Keycode::A, Action::Joypad(JoypadButton::BUTTON_A)),
        (Keycode::S, Action::Joypad(JoypadButton::BUTTON_B)),
        (Keycode::T, Action::ToggleTrace),
        (Keycode::F, Action::ToggleFrameWait)
    ]);
}

fn handle_user_input(cpu: &mut Cpu, event_pump: &mut EventPump) -> Action {
    for event in event_pump.poll_iter() {
        match event {
            Event::Quit { .. }
            | Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => std::process::exit(0),
            Event::KeyDown { keycode, .. } => {
                let keycode = keycode.unwrap_or(Keycode::Ampersand);
                if let Some(action) = KEY_MAP.get(&keycode) {
                    match action {
                        Action::Joypad(key) => {
                            cpu.bus.joypad1.set_button_status(key, true);
                        },
                        _ => {
                            return action.clone();
                        }
                    }
                }
            }
            Event::KeyUp { keycode, .. } => {
                let keycode = keycode.unwrap_or(Keycode::Ampersand);
                if let Some(action) = KEY_MAP.get(&keycode) {
                    match action {
                        Action::Joypad(key) => {
                            cpu.bus.joypad1.set_button_status(key, false);
                        },
                        _ => {}
                    }
                }
            }
            _ => { /* do nothing */ }
        }
    }
    return Action::None;
}

pub fn nes_emulator(args: Vec<String>) {
    const SCALE: usize = 3;

    // Init SDL
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

    // Read cartridge
    let filename = if args.len() >= 2 {
    &args[1]
    } else {
        "pacman.nes"
    };
    let raw = std::fs::read(filename).expect("Could not read the file");
    let cartridge = Cartridge::load(&raw).expect("Invalid cartridge data");
    let fps = match cartridge.video_signal {
        cartridge::VideoSignal::NTSC => 59.94,
        _ => panic!()
    };
    let msec_per_frame = 1000.0 / fps;

    // Associate cartridge to bus
    let mut cpu = Cpu::new();
    cpu.bus.load_cartridge(cartridge);
    let vector = cpu.bus.read16(0xfffc);
    cpu.set_pc(vector);

    // Associate apu to bus
    // Init apu
    let apu = init_apu(&sdl_context);
    apu.audio_device.as_ref().unwrap().resume();
    cpu.bus.associate_apu(apu);


    // For trace
    let mut prev_line = String::new();
    let mut same_count = 0;

    let mut settings = Settings { trace: false, wait: true };

    // Start emulation
    let start_time = Instant::now();
    let mut frame_count = 0u64;
    let mut skip_frame_count = 0u64;
    cpu.run_with_callback(
        &mut settings,
        |cpu, opaque| {
            let settings = opaque.downcast_mut::<Settings>().unwrap();
            let result = handle_user_input(cpu, &mut event_pump);
            match result {
                Action::ToggleTrace => {
                    settings.trace = !settings.trace;
                },
                Action::ToggleFrameWait => {
                    settings.wait = !settings.wait;
                    println!("Wait: {}", settings.wait);
                }
                _ => {},
            }

            if settings.trace {
                let line = cpu.trace();
                print!("{}", line);
                if prev_line == line {
                    same_count += 1;
                    print!(" x {} \r", same_count);
                } else {
                    print!("\r\n");
                    same_count = 0;
                }
                prev_line = line;
            }
        },
        |cpu, opaque| {
            let settings = opaque.downcast_mut::<Settings>().unwrap();

            cpu.bus.ppu.update_sdl_texture(&mut texture);
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();
            frame_count += 1;

            let current_time = Instant::now();
            let elapsed_time_real = current_time - start_time;
            let elapsed_time_nes = Duration::from_millis(frame_count * msec_per_frame as u64);
            let should_wait = elapsed_time_nes > elapsed_time_real;
            if should_wait {
                if settings.wait {
                    let wait_time = elapsed_time_nes - elapsed_time_real;
                    sleep(wait_time);
                }
                skip_frame_count = 0;
            } else {
                skip_frame_count += 1;
            }

            if settings.wait {
                cpu.bus.ppu.set_renderer_enabled(skip_frame_count < 5);
            }

            if settings.wait && skip_frame_count > fps as u64 {
                cpu.bus.ppu.set_renderer_enabled(true);
                skip_frame_count = 0;
            }
        },
    );
}
