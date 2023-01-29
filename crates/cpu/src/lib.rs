mod opcode;
use crate::opcode::*;
mod trace;

use bus::{Bus, PPU_REG_OAM_DMA};

#[cfg(test)]
pub mod tests {
    mod cpu_tests;
}

#[derive(Default, Debug, Copy, Clone)]
struct Flags {
    n: bool,
    v: bool,
    x: bool,
    b: bool,
    d: bool,
    i: bool,
    z: bool,
    c: bool,
}

#[derive(Default)]
pub struct Cpu {
    //ram: Vec<u8>,
    pc: u16,
    sp: u8,
    a: u8,
    x: u8,
    y: u8,
    f: Flags,
    pub bus: Bus,
    total_cycles: usize,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            sp: 0xfd,
            f: Flags {
                n: false,
                v: false,
                x: true,
                b: false,
                d: false,
                i: true,
                z: false,
                c: false,
            },
            total_cycles: 7,
            ..Default::default()
        }
    }

    fn load_and_run(&mut self, program: Vec<u8>) {
        self.bus.write_range(0x600, program);
        self.pc = 0x600 as u16;
        self.run();
    }

    fn update_nz(&mut self, result: u8) {
        self.f.z = result == 0;
        self.f.n = result & 0b1000_0000 > 0;
    }

    fn get_address(&mut self, mode: &AddressingMode) -> u16 {
        let operand_address = self.pc + 1;
        match mode {
            AddressingMode::Immediate => operand_address,
            AddressingMode::ZeroPage => self.bus.read8(operand_address) as u16,
            AddressingMode::ZeroPageX => {
                self.bus.read8(operand_address).wrapping_add(self.x) as u16
            }
            AddressingMode::ZeroPageY => {
                self.bus.read8(operand_address).wrapping_add(self.y) as u16
            }
            AddressingMode::Absolute => self.bus.read16(operand_address),
            AddressingMode::AbsoluteX => {
                self.bus.read16(operand_address).wrapping_add(self.x as u16)
            }
            AddressingMode::AbsoluteY => {
                self.bus.read16(operand_address).wrapping_add(self.y as u16)
            }
            AddressingMode::Indirect => {
                let ptr = self.bus.read16(operand_address);
                if ptr & 0x00FF == 0x00FF {
                    let lo = self.bus.read8(ptr) as u16;
                    let hi = self.bus.read8(ptr & 0xFF00) as u16;
                    hi << 8 | lo
                } else {
                    self.bus.read16(ptr)
                }
            }
            AddressingMode::IndirectX => {
                let ptr = self.bus.read8(operand_address).wrapping_add(self.x);
                let lo = self.bus.read8(ptr as u16) as u16;
                let hi = self.bus.read8(ptr.wrapping_add(1) as u16) as u16;
                hi << 8 | lo
            }
            AddressingMode::IndirectY => {
                let ptr = self.bus.read8(operand_address);
                let lo = self.bus.read8(ptr as u16) as u16;
                let hi = self.bus.read8(ptr.wrapping_add(1) as u16) as u16;
                (hi << 8 | lo).wrapping_add(self.y as u16)
            }
            AddressingMode::Relative => operand_address,
            AddressingMode::Implied => todo!(),
            AddressingMode::Accumulator => todo!(),
        }
    }

    fn page_crossed(&mut self, mode: &AddressingMode) -> bool {
        match mode {
            AddressingMode::AbsoluteX | AddressingMode::AbsoluteY => {
                (self.get_address(mode) ^ self.get_address(&AddressingMode::Absolute)) & 0xff00 > 0
            }
            AddressingMode::IndirectY => {
                let address_lo = (self.get_address(mode) & 0x00ff) as u8;
                self.y > address_lo
            }
            _ => false,
        }
    }

    fn push8(&mut self, data: u8) {
        let stack_address = (self.sp as u16) + 0x100;
        self.bus.write8(stack_address, data);
        self.sp = self.sp.wrapping_sub(1);
    }

    #[must_use]
    fn pop8(&mut self) -> u8 {
        let stack_address = (self.sp as u16) + 0x101;
        let data = self.bus.read8(stack_address);
        self.sp = self.sp.wrapping_add(1);
        data
    }

    fn push16(&mut self, data: u16) {
        let stack_address = (self.sp as u16) + 0x100 - 1;
        self.bus.write16(stack_address, data);
        //println!("push16 {:X} @ {:X} -> {:X}", data, self.sp, self.sp.wrapping_sub(2));
        self.sp = self.sp.wrapping_sub(2);
    }

    #[must_use]
    fn pop16(&mut self) -> u16 {
        let stack_address = (self.sp as u16) + 0x100 + 1;
        let data = self.bus.read16(stack_address);
        //println!("pop16 {:X} @ {:X} -> {:X}", data, self.sp, self.sp.wrapping_add(2));
        self.sp = self.sp.wrapping_add(2);
        data
    }

    fn intr_nmi(&mut self) {
        self.push16(self.pc);

        let mut flags = self.f;
        flags.b = false;
        flags.x = true;
        let flags = u8::from(flags.n) << 7
            | u8::from(flags.v) << 6
            | u8::from(flags.x) << 5
            | u8::from(flags.b) << 4
            | u8::from(flags.d) << 3
            | u8::from(flags.i) << 2
            | u8::from(flags.z) << 1
            | u8::from(flags.c) << 0;
        self.push8(flags);

        self.f.i = false;

        self.bus.tick(2);

        self.pc = self.bus.read16(0xfffa);
    }

    fn sbc_impl(&mut self, data: u8) {
        let (data_plus_carry, overflow1) = data.overflowing_add(u8::from(self.f.c));
        let (result, overflow2) = self.a.overflowing_add(data_plus_carry);

        // http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
        self.f.v = (self.a ^ !data) & (self.a ^ result) & 0x80 > 0;

        self.a = result;
        self.f.c = overflow1 || overflow2;
        self.update_nz(self.a);
    }

    fn adc_impl(&mut self, data: u8) {
        let (data_plus_carry, overflow1) = data.overflowing_add(u8::from(self.f.c));
        let (result, overflow2) = self.a.overflowing_add(data_plus_carry);

        // http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
        self.f.v = !(self.a ^ data) & (self.a ^ result) & 0x80 > 0;

        self.a = result;
        self.f.c = overflow1 || overflow2;
        self.update_nz(self.a);
    }

    fn adc(&mut self, mode: &AddressingMode) {
        let address = self.get_address(mode);
        let data = self.bus.read8(address);

        self.adc_impl(data);
    }

    fn sbc(&mut self, mode: &AddressingMode) {
        let address = self.get_address(mode);
        let data = !self.bus.read8(address);
        self.sbc_impl(data);
    }

    fn plp(&mut self) {
        let data = self.pop8();
        self.f.n = data & 0b1000_0000 > 0;
        self.f.v = data & 0b0100_0000 > 0;
        self.f.x = true;
        // self.f.b = data & 0b0001_0000 > 0;
        self.f.d = data & 0b0000_1000 > 0;
        self.f.i = data & 0b0000_0100 > 0;
        self.f.z = data & 0b0000_0010 > 0;
        self.f.c = data & 0b0000_0001 > 0;
    }

    fn branch(&mut self, mode: &AddressingMode, condition: bool) -> u8 {
        let mut cycle = 0;
        if condition {
            let address = self.get_address(mode);
            let data = self.bus.read8(address) as i8;

            let pc_add2 = self.pc.wrapping_add(2);
            self.pc = self.pc.wrapping_add_signed(data as i16);
            cycle += 1;

            if (self.pc ^ pc_add2) & 0xff00 > 0 {
                cycle += 1;
            }
        }
        cycle
    }

    fn run(&mut self) {
        self.run_with_callback(&mut 0, |_,_| {}, |_,_| {});
    }

    pub fn set_pc(&mut self, pc: u16) {
        self.pc = pc;
    }

    pub fn run_with_callback<F,F2>(&mut self, opaque: &mut dyn std::any::Any, mut callback: F, mut render_callback: F2)
    where
        F: FnMut(&mut Cpu, &mut dyn std::any::Any),
        F2: FnMut(&mut Cpu, &mut dyn std::any::Any),
    {
        loop {
            callback(self, opaque);

            // Read an opcode
            let opcode_u8 = self.bus.read8(self.pc);
            if !OPCODES.contains_key(&opcode_u8) {
                println!("Unknown opcode {:X}", opcode_u8);
            }
            let (opcode, mut cycles, mode, _) = &OPCODES[&opcode_u8];

            // Perform an operation
            match opcode {
                Opcodes::BRK => {
                    self.pc += 1;
                    return;
                }
                Opcodes::ADC => {
                    self.adc(mode);
                    cycles += u8::from(self.page_crossed(mode));
                }
                Opcodes::AND => {
                    let address = self.get_address(mode);
                    let data = self.bus.read8(address);
                    self.a = self.a & data;
                    self.update_nz(self.a);
                    cycles += u8::from(self.page_crossed(mode));
                }
                Opcodes::ASL => {
                    if mode == &AddressingMode::Accumulator {
                        // Sets carry flag if bit7 of old value enabled
                        self.f.c = self.a & 0b1000_0000 > 0;
                        self.a = self.a.wrapping_mul(2);
                        // Sets neg/zero flags with result value
                        self.update_nz(self.a);
                    } else {
                        let address = self.get_address(mode);
                        let data = self.bus.read8(address);
                        self.f.c = data & 0b1000_0000 > 0;
                        let result = data.wrapping_mul(2);
                        self.bus.write8(address, result);
                        self.update_nz(result);
                    }
                }
                Opcodes::BCC => cycles += self.branch(mode, !self.f.c),
                Opcodes::BCS => cycles += self.branch(mode, self.f.c),
                Opcodes::BEQ => cycles += self.branch(mode, self.f.z),
                Opcodes::BMI => cycles += self.branch(mode, self.f.n),
                Opcodes::BNE => cycles += self.branch(mode, !self.f.z),
                Opcodes::BPL => cycles += self.branch(mode, !self.f.n),
                Opcodes::BVC => cycles += self.branch(mode, !self.f.v),
                Opcodes::BVS => cycles += self.branch(mode, self.f.v),
                Opcodes::BIT => {
                    let address = self.get_address(mode);
                    let data = self.bus.read8(address);
                    let result = self.a & data;
                    self.f.z = result == 0;
                    self.f.v = data & 0b0100_0000 > 0;
                    self.f.n = data & 0b1000_0000 > 0;
                }
                Opcodes::CLC => self.f.c = false,
                Opcodes::CLD => self.f.d = false,
                Opcodes::CLI => self.f.i = false,
                Opcodes::CLV => self.f.v = false,
                Opcodes::CMP => {
                    let address = self.get_address(mode);
                    let data = self.bus.read8(address);
                    let result = self.a.wrapping_sub(data);
                    self.update_nz(result);
                    self.f.c = self.a >= data;
                    cycles += u8::from(self.page_crossed(mode));
                }
                Opcodes::CPX => {
                    let address = self.get_address(mode);
                    let data = self.bus.read8(address);
                    let result = self.x.wrapping_sub(data);
                    self.update_nz(result);
                    self.f.c = self.x >= data;
                }
                Opcodes::CPY => {
                    let address = self.get_address(mode);
                    let data = self.bus.read8(address);
                    let result = self.y.wrapping_sub(data);
                    self.update_nz(result);
                    self.f.c = self.y >= data;
                }
                Opcodes::DEC => {
                    let address = self.get_address(mode);
                    let data = self.bus.read8(address);
                    let result = data.wrapping_sub(1);
                    self.bus.write8(address, result);
                    self.update_nz(result);
                }
                Opcodes::DEX => {
                    self.x = self.x.wrapping_sub(1);
                    self.update_nz(self.x);
                }
                Opcodes::DEY => {
                    self.y = self.y.wrapping_sub(1);
                    self.update_nz(self.y);
                }
                Opcodes::EOR => {
                    let address = self.get_address(mode);
                    let data = self.bus.read8(address);
                    self.a = self.a ^ data;
                    self.update_nz(self.a);
                    cycles += u8::from(self.page_crossed(mode));
                }
                Opcodes::INC => {
                    let address = self.get_address(mode);
                    let data = self.bus.read8(address);
                    let result = data.wrapping_add(1);
                    self.update_nz(result);
                    self.bus.write8(address, result);
                }
                Opcodes::INX => {
                    self.x = self.x.wrapping_add(1);
                    self.update_nz(self.x);
                }
                Opcodes::INY => {
                    self.y = self.y.wrapping_add(1);
                    self.update_nz(self.y);
                }
                Opcodes::JMP => {
                    let address = self.get_address(mode);
                    self.pc = address;
                }
                Opcodes::JSR => {
                    let address = self.get_address(mode);
                    let stack_data = self.pc + MODE2BYTES[mode];
                    self.push16(stack_data);
                    self.pc = address;
                }
                Opcodes::LDA => {
                    let address = self.get_address(mode);
                    self.a = self.bus.read8(address);
                    self.update_nz(self.a);
                    cycles += u8::from(self.page_crossed(mode));
                }
                Opcodes::LDX => {
                    let address = self.get_address(mode);
                    self.x = self.bus.read8(address);
                    self.update_nz(self.x);
                    cycles += u8::from(self.page_crossed(mode));
                }
                Opcodes::LDY => {
                    let address = self.get_address(mode);
                    self.y = self.bus.read8(address);
                    self.update_nz(self.y);
                    cycles += u8::from(self.page_crossed(mode));
                }
                Opcodes::LSR => {
                    if mode == &AddressingMode::Accumulator {
                        self.f.c = self.a & 1 == 1;
                        self.a >>= 1;
                        self.update_nz(self.a);
                    } else {
                        let address = self.get_address(mode);
                        let data = self.bus.read8(address);
                        self.f.c = data & 1 == 1;
                        let result = data >> 1;
                        self.bus.write8(address, result);
                        self.update_nz(result);
                    }
                }
                Opcodes::NOP => {
                    cycles += u8::from(self.page_crossed(mode));
                }
                Opcodes::ORA => {
                    let address = self.get_address(mode);
                    let data = self.bus.read8(address);
                    self.a = self.a | data;
                    self.update_nz(self.a);
                    cycles += u8::from(self.page_crossed(mode));
                }
                Opcodes::PHA => self.push8(self.a),
                Opcodes::PHP => {
                    let data = u8::from(self.f.n) << 7
                        | u8::from(self.f.v) << 6
                        | u8::from(self.f.x) << 5
                        | u8::from(true) << 4 // Always pushes 1 as the B flag
                        | u8::from(self.f.d) << 3
                        | u8::from(self.f.i) << 2
                        | u8::from(self.f.z) << 1
                        | u8::from(self.f.c) << 0;
                    self.push8(data);
                }
                Opcodes::PLA => {
                    self.a = self.pop8();
                    self.update_nz(self.a);
                }
                Opcodes::PLP => self.plp(),
                Opcodes::ROL => {
                    if mode == &AddressingMode::Accumulator {
                        let result = self.a << 1 | u8::from(self.f.c);
                        self.f.c = self.a & 0b1000_0000 > 0;
                        self.a = result;
                        self.update_nz(self.a);
                    } else {
                        let address = self.get_address(mode);
                        let data = self.bus.read8(address);
                        let result = data << 1 | u8::from(self.f.c);
                        self.f.c = data & 0b1000_0000 > 0;
                        self.bus.write8(address, result);
                        self.update_nz(result);
                    }
                }
                Opcodes::ROR => {
                    if mode == &AddressingMode::Accumulator {
                        let result = self.a >> 1 | u8::from(self.f.c) << 7;
                        self.f.c = self.a & 0b1 > 0;
                        self.a = result;
                        self.update_nz(self.a);
                    } else {
                        let address = self.get_address(mode);
                        let data = self.bus.read8(address);
                        let result = data >> 1 | u8::from(self.f.c) << 7;
                        self.f.c = data & 0b1 > 0;
                        self.bus.write8(address, result);
                        self.update_nz(result);
                    }
                }
                Opcodes::RTI => {
                    self.plp();
                    self.pc = self.pop16().wrapping_sub(1);
                }
                Opcodes::RTS => {
                    self.pc = self.pop16();
                }
                Opcodes::SBC => {
                    self.sbc(mode);
                    cycles += u8::from(self.page_crossed(mode));
                }
                Opcodes::SEC => self.f.c = true,
                Opcodes::SED => self.f.d = true,
                Opcodes::SEI => self.f.i = true,
                Opcodes::STA => {
                    let address = self.get_address(mode);
                    self.bus.write8(address, self.a);
                    if address == PPU_REG_OAM_DMA {
                        // Suspend 513 + 1 cycles
                        self.bus.tick(64);
                        self.bus.tick(64);
                        self.bus.tick(64);
                        self.bus.tick(64);
                        self.bus.tick(64);
                        self.bus.tick(64);
                        self.bus.tick(64);
                        self.bus.tick(64);
                        cycles += 1;
                        if self.total_cycles & 1 > 0 {
                            cycles += 1;
                        }
                    }
                }
                Opcodes::STX => {
                    let address = self.get_address(mode);
                    self.bus.write8(address, self.x);
                }
                Opcodes::STY => {
                    let address = self.get_address(mode);
                    self.bus.write8(address, self.y);
                }
                Opcodes::TAX => {
                    self.x = self.a;
                    self.update_nz(self.x);
                }
                Opcodes::TAY => {
                    self.y = self.a;
                    self.update_nz(self.y);
                }
                Opcodes::TSX => {
                    self.x = self.sp;
                    self.update_nz(self.x);
                }
                Opcodes::TXA => {
                    self.a = self.x;
                    self.update_nz(self.a);
                }
                Opcodes::TXS => {
                    self.sp = self.x;
                }
                Opcodes::TYA => {
                    self.a = self.y;
                    self.update_nz(self.a);
                }
                //
                // Unofficial opcodes
                //
                Opcodes::AAC => {
                    let address = self.get_address(mode);
                    let data = self.bus.read8(address);
                    self.a = self.a & data;
                    self.update_nz(self.a);
                    self.f.c = self.f.n;
                }
                Opcodes::SAX => {
                    let address = self.get_address(mode);
                    let result = self.a & self.x;
                    // self.update_nz(result);
                    self.bus.write8(address, result);
                }
                Opcodes::ARR => {
                    let address = self.get_address(mode);
                    let data = self.bus.read8(address);
                    self.a = (self.a & data).rotate_right(1);

                    let bit6 = self.a & 0b0100_0000 > 0;
                    let bit5 = self.a & 0b0010_0000 > 0;
                    match (bit6, bit5) {
                        (true, true) => {
                            self.f.c = true;
                            self.f.v = false;
                        }
                        (false, false) => {
                            self.f.c = false;
                            self.f.v = false;
                        }
                        (true, false) => {
                            self.f.c = true;
                            self.f.v = true;
                        }
                        (false, true) => {
                            self.f.c = false;
                            self.f.v = true;
                        }
                        _ => {}
                    };
                }
                Opcodes::ASR => {
                    let address = self.get_address(mode);
                    let data = self.bus.read8(address);
                    self.a = (self.a & data).rotate_right(1);
                    self.f.c = self.a & 0b1000_0000 > 0;
                    self.update_nz(self.a);
                }
                Opcodes::ATX => {
                    let address = self.get_address(mode);
                    let data = self.bus.read8(address);
                    self.a = self.a & data;
                    self.x = self.a;
                    self.update_nz(self.x);
                }
                Opcodes::AXA => todo!(),
                Opcodes::AXS => todo!(),
                Opcodes::DCP => {
                    let address = self.get_address(mode);
                    let data = self.bus.read8(address);
                    // DEC
                    let result = data.wrapping_sub(1);
                    self.bus.write8(address, result);

                    // CMP
                    let (result, overflow) = self.a.overflowing_sub(result);
                    self.update_nz(result);
                    self.f.c = !overflow;
                }
                Opcodes::DOP => {}
                Opcodes::ISB => {
                    let address = self.get_address(mode);
                    let data = self.bus.read8(address);
                    let result = data.wrapping_add(1);
                    self.bus.write8(address, result);

                    self.sbc_impl(!result);
                }
                Opcodes::KIL => {
                    return;
                }
                Opcodes::LAR => {
                    let address = self.get_address(mode);
                    let data = self.bus.read8(address);
                    let result = self.sp & data;
                    self.a = result;
                    self.x = result;
                    self.sp = result;

                    self.update_nz(self.sp);
                    cycles += u8::from(self.page_crossed(mode));
                }
                Opcodes::LAX => {
                    let address = self.get_address(mode);
                    let data = self.bus.read8(address);
                    self.a = data;
                    self.x = data;
                    self.update_nz(self.x);
                    cycles += u8::from(self.page_crossed(mode));
                }
                Opcodes::RLA => {
                    let address = self.get_address(mode);
                    let data = self.bus.read8(address);
                    let result = data << 1 | u8::from(self.f.c);
                    self.f.c = data & 0b1000_0000 > 0;
                    self.bus.write8(address, result);

                    self.a = self.a & result;
                    self.update_nz(self.a);
                }
                Opcodes::RRA => {
                    let address = self.get_address(mode);
                    let data = self.bus.read8(address);
                    let result = data >> 1 | u8::from(self.f.c) << 7;
                    self.bus.write8(address, result);
                    self.f.c = data & 0b1 > 0;

                    self.adc_impl(result);
                }
                Opcodes::SLO => {
                    let address = self.get_address(mode);
                    let data = self.bus.read8(address);
                    let result = data << 1;
                    self.f.c = data & 0b1000_0000 > 0;
                    self.bus.write8(address, result);

                    self.a = self.a | result;
                    self.update_nz(self.a);
                }
                Opcodes::SRE => {
                    let address = self.get_address(mode);
                    let data = self.bus.read8(address);
                    let result = data >> 1;
                    self.f.c = data & 0b1 > 0;
                    self.bus.write8(address, result);

                    self.a = self.a ^ result;
                    self.update_nz(self.a);
                }
                Opcodes::SXA => todo!(),
                Opcodes::SYA => todo!(),
                Opcodes::TOP => {
                    cycles += u8::from(self.page_crossed(mode));
                }
                Opcodes::XAA => todo!(),
                Opcodes::XAS => todo!(),
            }

            // Consume some bytes except jumping
            match opcode {
                Opcodes::JMP | Opcodes::JSR => {}
                // 1 for a opcode and rest for a operand
                _ => self.pc += 1 + MODE2BYTES[mode],
            }

            self.total_cycles += cycles as usize;
            // Tell current instruction's tick to PPU through bus
            // and do callback or generate intr based on `TickResult`
            let tick_results = self.bus.tick(cycles);
            match &tick_results {
                _ if tick_results.contains(&ppu::TickResult::ShouldInterruptNmiAndUpdateTexture) => {
                    render_callback(self, opaque);
                    self.intr_nmi();
                }
                _ if tick_results.contains(&ppu::TickResult::ShouldUpdateTexture) => {
                    render_callback(self, opaque);
                }
                _ => {}
            }
        }
    }
}
