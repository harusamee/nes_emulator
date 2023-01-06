use chr_rom_viewer::chr_rom_viewer;

mod nes_emulator;
mod chr_rom_viewer;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    match args[1].as_str() {
        "chr" => chr_rom_viewer(args),
        _ => nes_emulator::nes_emulator(args)
    }
}
