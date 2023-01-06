use crate::{Cpu, opcode::{OPCODES, MODE2BYTES, AddressingMode, Opcodes}};

impl Cpu {
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

        let operand_address = self.pc + 1;
        let operands = match mode {
            AddressingMode::Accumulator => String::from("A"),
            AddressingMode::Immediate => format!("#${:02X}", self.bus.read8(operand_address)),
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
                    self.bus.read8(operand_address),
                    address as u8,
                    data
                )
            }
            AddressingMode::ZeroPageY => {
                let address = self.get_address(mode);
                let data = self.bus.read8(address);
                format!(
                    "${:02X},Y @ {:02X} = {:02X}",
                    self.bus.read8(operand_address),
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
                let operand_memory = self.bus.read16(operand_address);
                let address = self.get_address(mode);
                format!(
                    "${:04X},X @ {:04X} = {:02X}",
                    operand_memory,
                    address,
                    self.bus.read8(address),
                )
            },
            AddressingMode::AbsoluteY => {
                let operand_memory = self.bus.read16(operand_address);
                let address = self.get_address(mode);
                format!(
                    "${:04X},Y @ {:04X} = {:02X}",
                    operand_memory,
                    address,
                    self.bus.read8(address),
                )
            },
            AddressingMode::Indirect => {
                let address = self.bus.read16(operand_address);
                let deref = self.get_address(mode);
                format!("(${:04X}) = {:04X}", address, deref)
            },
            AddressingMode::IndirectX => {
                let operand_memory = self.bus.read8(operand_address);
                let operand_plus_x = operand_memory.wrapping_add(self.x);
                let address = self.get_address(mode);
                format!(
                    "(${:02X},X) @ {:02X} = {:04X} = {:02X}",
                    operand_memory,
                    operand_plus_x,
                    address,
                    self.bus.read8(address),
                )
            }
            AddressingMode::IndirectY => {
                let operand_memory = self.bus.read8(operand_address);
                let lo = self.bus.read8(operand_memory as u16) as u16;
                let hi = self.bus.read8(operand_memory.wrapping_add(1) as u16) as u16;
                let operand_address_deref = hi << 8 | lo;

                let address = self.get_address(mode);
                format!(
                    "(${:02X}),Y = {:04X} @ {:04X} = {:02X}",
                    operand_memory,
                    operand_address_deref,
                    address,
                    self.bus.read8(address),
                )
            },
            AddressingMode::Relative => {
                let address = self.get_address(mode);
                let data = self.bus.read8(address);
                let mut pc_plus_data = operand_address;
                if data & 0b1000_0000 > 0 {
                    pc_plus_data = pc_plus_data.wrapping_sub(!data as u16);
                } else {
                    pc_plus_data = pc_plus_data.wrapping_add(data as u16).wrapping_add(1);
                }
                format!("${:04X}", pc_plus_data)
            },
            AddressingMode::Implied => String::from(""),
        };

        let opcode = match is_official {
            true => format!(" {:?}", opcode),
            false => format!("*{:?}", opcode)
        };

        format!(
            "{:04X}  {:8} {} {:27} A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X}",
            self.pc, mem_content, opcode, operands, self.a, self.x, self.y, status, self.sp
        )
    }
}