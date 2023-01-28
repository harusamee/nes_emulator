use cartridge::Cartridge;
use cpu::Cpu;

pub fn nestest(_: Vec<String>) {
    let raw = std::fs::read("nestest.nes").expect("Could not read the file");
    let cartridge = Cartridge::load(&raw).expect("Invalid cartridge data");
    let mut cpu = Cpu::new();
    cpu.bus.load_cartridge(cartridge);
    cpu.set_pc(0xc000);

    cpu.run_with_callback(&mut 0, |cpu, _| {
        let line = cpu.trace();
        println!("{}", line);
    }, |_, _|{});
}