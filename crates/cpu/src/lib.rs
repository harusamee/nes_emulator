mod opcode;
use bus::Bus;

use crate::opcode::*;

#[cfg(test)]
pub mod tests {
    mod cpu_tests;
}

#[derive(Default, Debug)]
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
            ..Default::default()
        }
    }

    pub fn trace(&mut self) -> String {
        let opcode_u8 = self.bus.read8(self.pc);
        if !OPCODES.contains_key(&opcode_u8) {
            println!("Unknown opcode {:X}", opcode_u8);
        }
        let (opcode, _, mode, is_official) = &OPCODES[&opcode_u8];

        let status: u8 = u8::from(self.f.n) << 7
            | u8::from(self.f.v) << 6
            | u8::from(self.f.x) << 5
            | u8::from(self.f.b) << 4
            | u8::from(self.f.d) << 3
            | u8::from(self.f.i) << 2
            | u8::from(self.f.z) << 1
            | u8::from(self.f.c) << 0;

        let mem_content = (0..(1 + MODE2BYTES[mode]))
            .map(|i| format!("{:02X}", self.bus.read8(self.pc + i)))
            .collect::<Vec<String>>()
            .join(" ");

        self.pc += 1;
        let operands = match mode {
            AddressingMode::Accumulator => String::from("A"),
            AddressingMode::Immediate => format!("#${:02X}", self.bus.read8(self.pc)),
            AddressingMode::ZeroPage => {
                let address = self.get_address(mode);
                let data = self.bus.read8(address);
                format!("${:02X} = {:02X}", address, data)
            },
            AddressingMode::ZeroPageX => {
                let address = self.get_address(mode);
                let data = self.bus.read8(address);
                format!(
                    "${:02X},X @ {:02X} = {:02X}",
                    self.bus.read8(self.pc),
                    address as u8,
                    data
                )
            }
            AddressingMode::ZeroPageY => {
                let address = self.get_address(mode);
                let data = self.bus.read8(address);
                format!(
                    "${:02X},Y @ {:02X} = {:02X}",
                    self.bus.read8(self.pc),
                    address as u8,
                    data
                )
            }
            AddressingMode::Absolute => {
                let address = self.get_address(mode);
                match opcode {
                    Opcodes::JSR | Opcodes::JMP => {
                        format!("${:04X}", address)
                    }
                    _ => {
                        format!(
                            "${:04X} = {:02X}",
                            address,
                            self.bus.read8(address),
                        )
                    }
                }
            }
            AddressingMode::AbsoluteX => {
                let operand_address = self.bus.read16(self.pc);
                let address = self.get_address(mode);
                format!(
                    "${:04X},X @ {:04X} = {:02X}",
                    operand_address,
                    address,
                    self.bus.read8(address),
                )
            },
            AddressingMode::AbsoluteY => {
                let operand_address = self.bus.read16(self.pc);
                let address = self.get_address(mode);
                format!(
                    "${:04X},Y @ {:04X} = {:02X}",
                    operand_address,
                    address,
                    self.bus.read8(address),
                )
            },
            AddressingMode::Indirect => {
                let address = self.bus.read16(self.pc);
                let deref = self.get_address(mode);
                format!("(${:04X}) = {:04X}", address, deref)
            },
            AddressingMode::IndirectX => {
                let operand_address = self.bus.read8(self.pc);
                let operand_plus_x = operand_address.wrapping_add(self.x);
                let address = self.get_address(mode);
                format!(
                    "(${:02X},X) @ {:02X} = {:04X} = {:02X}",
                    operand_address,
                    operand_plus_x,
                    address,
                    self.bus.read8(address),
                )
            }
            AddressingMode::IndirectY => {
                let operand_address = self.bus.read8(self.pc);
                let lo = self.bus.read8(operand_address as u16) as u16;
                let hi = self.bus.read8(operand_address.wrapping_add(1) as u16) as u16;
                let operand_address_deref = hi << 8 | lo;

                let address = self.get_address(mode);
                format!(
                    "(${:02X}),Y = {:04X} @ {:04X} = {:02X}",
                    operand_address,
                    operand_address_deref,
                    address,
                    self.bus.read8(address),
                )
            },
            AddressingMode::Relative => {
                let address = self.get_address(mode);
                let data = self.bus.read8(address);
                let mut pc_plus_data = self.pc;
                if data & 0b1000_0000 > 0 {
                    pc_plus_data = pc_plus_data.wrapping_sub(!data as u16);
                } else {
                    pc_plus_data = pc_plus_data.wrapping_add(data as u16).wrapping_add(1);
                }
                format!("${:04X}", pc_plus_data)
            },
            AddressingMode::Implied => String::from(""),
        };
        self.pc -= 1;


        let opcode = match is_official {
            true => format!(" {:?}", opcode),
            false => format!("*{:?}", opcode)
        };

        format!(
            "{:04X}  {:8} {} {:27} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
            self.pc, mem_content, opcode, operands, self.a, self.x, self.y, status, self.sp
        )
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

    fn get_address(&self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.pc,
            AddressingMode::ZeroPage => self.bus.read8(self.pc) as u16,
            AddressingMode::ZeroPageX => self.bus.read8(self.pc).wrapping_add(self.x) as u16,
            AddressingMode::ZeroPageY => self.bus.read8(self.pc).wrapping_add(self.y) as u16,
            AddressingMode::Absolute => self.bus.read16(self.pc),
            AddressingMode::AbsoluteX => self.bus.read16(self.pc).wrapping_add(self.x as u16),
            AddressingMode::AbsoluteY => self.bus.read16(self.pc).wrapping_add(self.y as u16),
            AddressingMode::Indirect => {
                let ptr = self.bus.read16(self.pc);
                if ptr & 0x00FF == 0x00FF {
                    let lo = self.bus.read8(ptr) as u16;
                    let hi = self.bus.read8(ptr & 0xFF00) as u16;
                    hi << 8 | lo
                } else {
                    self.bus.read16(ptr)
                }
            }
            AddressingMode::IndirectX => {
                let ptr = self.bus.read8(self.pc).wrapping_add(self.x);
                let lo = self.bus.read8(ptr as u16) as u16;
                let hi = self.bus.read8(ptr.wrapping_add(1) as u16) as u16;
                hi << 8 | lo
            }
            AddressingMode::IndirectY => {
                let ptr = self.bus.read8(self.pc);
                let lo = self.bus.read8(ptr as u16) as u16;
                let hi = self.bus.read8(ptr.wrapping_add(1) as u16) as u16;
                (hi << 8 | lo).wrapping_add(self.y as u16)
            }
            AddressingMode::Relative => self.pc,
            AddressingMode::Implied => todo!(),
            AddressingMode::Accumulator => todo!(),
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

    fn branch(&mut self, mode: &AddressingMode, condition: bool) {
        let address = self.get_address(mode);
        let data = self.bus.read8(address) as i8;
        if condition {
            self.pc = self.pc.wrapping_add_signed(data as i16);
        }
    }

    fn run(&mut self) {
        self.run_with_callback(|_| {});
    }

    pub fn set_pc(&mut self, pc: u16) {
        self.pc = pc;
    }

    pub fn run_with_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut Cpu),
    {
        loop {
            callback(self);

            // Read an opcode
            let opcode_u8 = self.bus.read8(self.pc);
            if !OPCODES.contains_key(&opcode_u8) {
                println!("Unknown opcode {:X}", opcode_u8);
            }
            let (opcode, _, mode, _) = &OPCODES[&opcode_u8];
            // Consume one byte
            self.pc += 1;

            // Perform an operation
            match opcode {
                Opcodes::BRK => return,
                Opcodes::ADC => self.adc(mode),
                Opcodes::AND => {
                    let address = self.get_address(mode);
                    let data = self.bus.read8(address);
                    self.a = self.a & data;
                    self.update_nz(self.a);
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
                Opcodes::BCC => self.branch(mode, !self.f.c),
                Opcodes::BCS => self.branch(mode, self.f.c),
                Opcodes::BEQ => self.branch(mode, self.f.z),
                Opcodes::BMI => self.branch(mode, self.f.n),
                Opcodes::BNE => self.branch(mode, !self.f.z),
                Opcodes::BPL => self.branch(mode, !self.f.n),
                Opcodes::BVC => self.branch(mode, !self.f.v),
                Opcodes::BVS => self.branch(mode, self.f.v),
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
                    let stack_data = self.pc + MODE2BYTES[mode] - 1;
                    self.push16(stack_data);
                    self.pc = address;
                }
                Opcodes::LDA => {
                    let address = self.get_address(mode);
                    self.a = self.bus.read8(address);
                    self.update_nz(self.a);
                }
                Opcodes::LDX => {
                    let address = self.get_address(mode);
                    self.x = self.bus.read8(address);
                    self.update_nz(self.x);
                }
                Opcodes::LDY => {
                    let address = self.get_address(mode);
                    self.y = self.bus.read8(address);
                    self.update_nz(self.y);
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
                Opcodes::NOP => {}
                Opcodes::ORA => {
                    let address = self.get_address(mode);
                    let data = self.bus.read8(address);
                    self.a = self.a | data;
                    self.update_nz(self.a);
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
                    self.pc = self.pop16();
                }
                Opcodes::RTS => {
                    let address = self.pop16();
                    self.pc = address.wrapping_add(1);
                }
                Opcodes::SBC => self.sbc(mode),
                Opcodes::SEC => self.f.c = true,
                Opcodes::SED => self.f.d = true,
                Opcodes::SEI => self.f.i = true,
                Opcodes::STA => {
                    let address = self.get_address(mode);
                    self.bus.write8(address, self.a);
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
                },
                Opcodes::SAX => {
                    let address = self.get_address(mode);
                    let result = self.a & self.x;
                    // self.update_nz(result);
                    self.bus.write8(address, result);
                },
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
                },
                Opcodes::ASR => {
                    let address = self.get_address(mode);
                    let data = self.bus.read8(address);
                    self.a = (self.a & data).rotate_right(1);
                    self.f.c = self.a & 0b1000_0000 > 0;
                    self.update_nz(self.a);
                },
                Opcodes::ATX => {
                    let address = self.get_address(mode);
                    let data = self.bus.read8(address);
                    self.a = self.a & data;
                    self.x = self.a;
                    self.update_nz(self.x);
                },
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
                },
                Opcodes::DOP => {},
                Opcodes::ISB => {
                    let address = self.get_address(mode);
                    let data = self.bus.read8(address);
                    let result = data.wrapping_add(1);
                    self.bus.write8(address, result);

                    self.sbc_impl(!result);
                },
                Opcodes::KIL => { return; },
                Opcodes::LAR => {
                    let address = self.get_address(mode);
                    let data = self.bus.read8(address);
                    let result = self.sp & data;
                    self.a = result;
                    self.x = result;
                    self.sp = result;

                    self.update_nz(self.sp);
                },
                Opcodes::LAX => {
                    let address = self.get_address(mode);
                    let data = self.bus.read8(address);
                    self.a = data;
                    self.x = data;
                    self.update_nz(self.x);
                },
                Opcodes::RLA => {
                    let address = self.get_address(mode);
                    let data = self.bus.read8(address);
                    let result = data << 1 | u8::from(self.f.c);
                    self.f.c = data & 0b1000_0000 > 0;
                    self.bus.write8(address, result);

                    self.a = self.a & result;
                    self.update_nz(self.a);
                },
                Opcodes::RRA => {
                    let address = self.get_address(mode);
                    let data = self.bus.read8(address);
                    let result = data >> 1 | u8::from(self.f.c) << 7;
                    self.bus.write8(address, result);
                    self.f.c = data & 0b1 > 0;

                    self.adc_impl(result);
                },
                Opcodes::SLO => {
                    let address = self.get_address(mode);
                    let data = self.bus.read8(address);
                    let result = data << 1;
                    self.f.c = data & 0b1000_0000 > 0;
                    self.bus.write8(address, result);

                    self.a = self.a | result;
                    self.update_nz(self.a);
                },
                Opcodes::SRE => {
                    let address = self.get_address(mode);
                    let data = self.bus.read8(address);
                    let result = data >> 1;
                    self.f.c = data & 0b1 > 0;
                    self.bus.write8(address, result);

                    self.a = self.a ^ result;
                    self.update_nz(self.a);
                },
                Opcodes::SXA => todo!(),
                Opcodes::SYA => todo!(),
                Opcodes::TOP => {},
                Opcodes::XAA => todo!(),
                Opcodes::XAS => todo!(),
            }

            // Consume some bytes except jumping
            match opcode {
                Opcodes::JMP => {}
                Opcodes::JSR => {}
                _ => self.pc += MODE2BYTES[mode],
            }
        }
    }
}
