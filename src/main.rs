mod nes_emulator;
mod chr_rom_viewer;
mod nestest;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() >= 2 {
        match args[1].as_str() {
            "chr" => chr_rom_viewer::chr_rom_viewer(args),
            "nestest" => nestest::nestest(args),
            _ => nes_emulator::nes_emulator(args)
        }    
    } else {
        let filename = std::path::Path::new(&args[0]).file_name().unwrap().to_str().unwrap();
        println!("{} *.nes", filename);
        println!("{} chr *.nes", filename);
        println!("{} nestest nestest.nes", filename);
    }
}
