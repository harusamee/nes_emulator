use lazy_static::lazy_static;
use std::collections::HashMap;

#[derive(Debug)]
pub enum Opcodes {
    ADC,
    AND,
    ASL,
    BCC,
    BCS,
    BEQ,
    BIT,
    BMI,
    BNE,
    BPL,
    BRK,
    BVC,
    BVS,
    CLC,
    CLD,
    CLI,
    CLV,
    CMP,
    CPX,
    CPY,
    DEC,
    DEX,
    DEY,
    EOR,
    INC,
    INX,
    INY,
    JMP,
    JSR,
    LDA,
    LDX,
    LDY,
    LSR,
    NOP,
    ORA,
    PHA,
    PHP,
    PLA,
    PLP,
    ROL,
    ROR,
    RTI,
    RTS,
    SBC,
    SEC,
    SED,
    SEI,
    STA,
    STX,
    STY,
    TAX,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,
    // Unofficial opcodes
    AAC,
    SAX,
    ARR,
    ASR,
    ATX,
    AXA,
    AXS,
    DCP,
    DOP,
    ISB,
    KIL,
    LAR,
    LAX,
    RLA,
    RRA,
    SLO,
    SRE,
    SXA,
    SYA,
    TOP,
    XAA,
    XAS,
}

#[derive(PartialEq, Eq, Hash)]
pub enum AddressingMode {
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY,
    Relative,
    Implied,
}

type Cycle = u8;
type IsOfficial = bool;
type OpcodeValue = (Opcodes, Cycle, AddressingMode, IsOfficial);
lazy_static! {
    pub static ref OPCODES: HashMap<u8, OpcodeValue> = HashMap::from([
        (0x69, (Opcodes::ADC, 2, AddressingMode::Immediate, true)),
        (0x65, (Opcodes::ADC, 3, AddressingMode::ZeroPage, true)),
        (0x75, (Opcodes::ADC, 4, AddressingMode::ZeroPageX, true)),
        (0x6D, (Opcodes::ADC, 4, AddressingMode::Absolute, true)),
        (0x7D, (Opcodes::ADC, 4, AddressingMode::AbsoluteX, true)),
        (0x79, (Opcodes::ADC, 4, AddressingMode::AbsoluteY, true)),
        (0x61, (Opcodes::ADC, 6, AddressingMode::IndirectX, true)),
        (0x71, (Opcodes::ADC, 5, AddressingMode::IndirectY, true)),

        (0x29, (Opcodes::AND, 2, AddressingMode::Immediate, true)),
        (0x25, (Opcodes::AND, 3, AddressingMode::ZeroPage, true)),
        (0x35, (Opcodes::AND, 4, AddressingMode::ZeroPageX, true)),
        (0x2D, (Opcodes::AND, 4, AddressingMode::Absolute, true)),
        (0x3D, (Opcodes::AND, 4, AddressingMode::AbsoluteX, true)),
        (0x39, (Opcodes::AND, 4, AddressingMode::AbsoluteY, true)),
        (0x21, (Opcodes::AND, 6, AddressingMode::IndirectX, true)),
        (0x31, (Opcodes::AND, 5, AddressingMode::IndirectY, true)),

        (0x0a, (Opcodes::ASL, 2, AddressingMode::Accumulator, true)),
        (0x06, (Opcodes::ASL, 5, AddressingMode::ZeroPage, true)),
        (0x16, (Opcodes::ASL, 6, AddressingMode::ZeroPageX, true)),
        (0x0e, (Opcodes::ASL, 6, AddressingMode::Absolute, true)),
        (0x1e, (Opcodes::ASL, 7, AddressingMode::AbsoluteX, true)),

        // Branches
        (0x90, (Opcodes::BCC, 2, AddressingMode::Relative, true)),
        (0xb0, (Opcodes::BCS, 2, AddressingMode::Relative, true)),
        (0xf0, (Opcodes::BEQ, 2, AddressingMode::Relative, true)),
        (0x30, (Opcodes::BMI, 2, AddressingMode::Relative, true)),
        (0xd0, (Opcodes::BNE, 2, AddressingMode::Relative, true)),
        (0x10, (Opcodes::BPL, 2, AddressingMode::Relative, true)),
        (0x50, (Opcodes::BVC, 2, AddressingMode::Relative, true)),
        (0x70, (Opcodes::BVS, 2, AddressingMode::Relative, true)),

        (0x24, (Opcodes::BIT, 3, AddressingMode::ZeroPage, true)),
        (0x2c, (Opcodes::BIT, 4, AddressingMode::Absolute, true)),

        (0x00, (Opcodes::BRK, 7, AddressingMode::Implied, true)),

        // Clears
        (0x18, (Opcodes::CLC, 2, AddressingMode::Implied, true)),
        (0xd8, (Opcodes::CLD, 2, AddressingMode::Implied, true)),
        (0x58, (Opcodes::CLI, 2, AddressingMode::Implied, true)),
        (0xb8, (Opcodes::CLV, 2, AddressingMode::Implied, true)),

        (0xc9, (Opcodes::CMP, 2, AddressingMode::Immediate, true)),
        (0xc5, (Opcodes::CMP, 3, AddressingMode::ZeroPage, true)),
        (0xd5, (Opcodes::CMP, 4, AddressingMode::ZeroPageX, true)),
        (0xcd, (Opcodes::CMP, 4, AddressingMode::Absolute, true)),
        (0xdd, (Opcodes::CMP, 4, AddressingMode::AbsoluteX, true)),
        (0xd9, (Opcodes::CMP, 4, AddressingMode::AbsoluteY, true)),
        (0xc1, (Opcodes::CMP, 6, AddressingMode::IndirectX, true)),
        (0xd1, (Opcodes::CMP, 5, AddressingMode::IndirectY, true)),

        (0xe0, (Opcodes::CPX, 2, AddressingMode::Immediate, true)),
        (0xe4, (Opcodes::CPX, 3, AddressingMode::ZeroPage, true)),
        (0xec, (Opcodes::CPX, 4, AddressingMode::Absolute, true)),

        (0xc0, (Opcodes::CPY, 2, AddressingMode::Immediate, true)),
        (0xc4, (Opcodes::CPY, 3, AddressingMode::ZeroPage, true)),
        (0xcc, (Opcodes::CPY, 4, AddressingMode::Absolute, true)),

        (0xc6, (Opcodes::DEC, 5, AddressingMode::ZeroPage, true)),
        (0xd6, (Opcodes::DEC, 6, AddressingMode::ZeroPageX, true)),
        (0xce, (Opcodes::DEC, 6, AddressingMode::Absolute, true)),
        (0xde, (Opcodes::DEC, 7, AddressingMode::AbsoluteX, true)),

        (0xca, (Opcodes::DEX, 2, AddressingMode::Implied, true)),
        (0x88, (Opcodes::DEY, 2, AddressingMode::Implied, true)),

        (0x49, (Opcodes::EOR, 2, AddressingMode::Immediate, true)),
        (0x45, (Opcodes::EOR, 3, AddressingMode::ZeroPage, true)),
        (0x55, (Opcodes::EOR, 4, AddressingMode::ZeroPageX, true)),
        (0x4d, (Opcodes::EOR, 4, AddressingMode::Absolute, true)),
        (0x5d, (Opcodes::EOR, 4, AddressingMode::AbsoluteX, true)),
        (0x59, (Opcodes::EOR, 4, AddressingMode::AbsoluteY, true)),
        (0x41, (Opcodes::EOR, 6, AddressingMode::IndirectX, true)),
        (0x51, (Opcodes::EOR, 5, AddressingMode::IndirectY, true)),

        // Increments
        (0xe6, (Opcodes::INC, 5, AddressingMode::ZeroPage, true)),
        (0xf6, (Opcodes::INC, 6, AddressingMode::ZeroPageX, true)),
        (0xee, (Opcodes::INC, 6, AddressingMode::Absolute, true)),
        (0xfe, (Opcodes::INC, 7, AddressingMode::AbsoluteX, true)),
        (0xe8, (Opcodes::INX, 2, AddressingMode::Implied, true)),
        (0xc8, (Opcodes::INY, 2, AddressingMode::Implied, true)),

        // Jumps
        (0x4c, (Opcodes::JMP, 3, AddressingMode::Absolute, true)),
        (0x6c, (Opcodes::JMP, 5, AddressingMode::Indirect, true)),
        (0x20, (Opcodes::JSR, 6, AddressingMode::Absolute, true)),

        (0xA9, (Opcodes::LDA, 2, AddressingMode::Immediate, true)),
        (0xA5, (Opcodes::LDA, 3, AddressingMode::ZeroPage, true)),
        (0xB5, (Opcodes::LDA, 4, AddressingMode::ZeroPageX, true)),
        (0xAD, (Opcodes::LDA, 4, AddressingMode::Absolute, true)),
        (0xBD, (Opcodes::LDA, 4, AddressingMode::AbsoluteX, true)),
        (0xB9, (Opcodes::LDA, 4, AddressingMode::AbsoluteY, true)),
        (0xA1, (Opcodes::LDA, 6, AddressingMode::IndirectX, true)),
        (0xB1, (Opcodes::LDA, 5, AddressingMode::IndirectY, true)),

        (0xA2, (Opcodes::LDX, 2, AddressingMode::Immediate, true)),
        (0xA6, (Opcodes::LDX, 3, AddressingMode::ZeroPage, true)),
        (0xB6, (Opcodes::LDX, 4, AddressingMode::ZeroPageY, true)),
        (0xAE, (Opcodes::LDX, 4, AddressingMode::Absolute, true)),
        (0xBE, (Opcodes::LDX, 4, AddressingMode::AbsoluteY, true)),

        (0xA0, (Opcodes::LDY, 2, AddressingMode::Immediate, true)),
        (0xA4, (Opcodes::LDY, 3, AddressingMode::ZeroPage, true)),
        (0xB4, (Opcodes::LDY, 4, AddressingMode::ZeroPageX, true)),
        (0xAC, (Opcodes::LDY, 4, AddressingMode::Absolute, true)),
        (0xBC, (Opcodes::LDY, 4, AddressingMode::AbsoluteX, true)),

        (0x4a, (Opcodes::LSR, 2, AddressingMode::Accumulator, true)),
        (0x46, (Opcodes::LSR, 5, AddressingMode::ZeroPage, true)),
        (0x56, (Opcodes::LSR, 6, AddressingMode::ZeroPageX, true)),
        (0x4e, (Opcodes::LSR, 6, AddressingMode::Absolute, true)),
        (0x5e, (Opcodes::LSR, 7, AddressingMode::AbsoluteX, true)),

        (0xea, (Opcodes::NOP, 2, AddressingMode::Implied, true)),

        (0x09, (Opcodes::ORA, 2, AddressingMode::Immediate, true)),
        (0x05, (Opcodes::ORA, 3, AddressingMode::ZeroPage, true)),
        (0x15, (Opcodes::ORA, 4, AddressingMode::ZeroPageX, true)),
        (0x0d, (Opcodes::ORA, 4, AddressingMode::Absolute, true)),
        (0x1d, (Opcodes::ORA, 4, AddressingMode::AbsoluteX, true)),
        (0x19, (Opcodes::ORA, 4, AddressingMode::AbsoluteY, true)),
        (0x01, (Opcodes::ORA, 6, AddressingMode::IndirectX, true)),
        (0x11, (Opcodes::ORA, 5, AddressingMode::IndirectY, true)),

        // Stacks
        (0x48, (Opcodes::PHA, 3, AddressingMode::Implied, true)),
        (0x08, (Opcodes::PHP, 3, AddressingMode::Implied, true)),
        (0x68, (Opcodes::PLA, 4, AddressingMode::Implied, true)),
        (0x28, (Opcodes::PLP, 4, AddressingMode::Implied, true)),

        (0x2a, (Opcodes::ROL, 2, AddressingMode::Accumulator, true)),
        (0x26, (Opcodes::ROL, 5, AddressingMode::ZeroPage, true)),
        (0x36, (Opcodes::ROL, 6, AddressingMode::ZeroPageX, true)),
        (0x2e, (Opcodes::ROL, 6, AddressingMode::Absolute, true)),
        (0x3e, (Opcodes::ROL, 7, AddressingMode::AbsoluteX, true)),

        (0x6a, (Opcodes::ROR, 2, AddressingMode::Accumulator, true)),
        (0x66, (Opcodes::ROR, 3, AddressingMode::ZeroPage, true)),
        (0x76, (Opcodes::ROR, 4, AddressingMode::ZeroPageX, true)),
        (0x6e, (Opcodes::ROR, 4, AddressingMode::Absolute, true)),
        (0x7e, (Opcodes::ROR, 4, AddressingMode::AbsoluteX, true)),

        (0x60, (Opcodes::RTS, 6, AddressingMode::Implied, true)),
        (0x40, (Opcodes::RTI, 6, AddressingMode::Implied, true)),

        (0xe9, (Opcodes::SBC, 2, AddressingMode::Immediate, true)),
        (0xe5, (Opcodes::SBC, 3, AddressingMode::ZeroPage, true)),
        (0xf5, (Opcodes::SBC, 4, AddressingMode::ZeroPageX, true)),
        (0xed, (Opcodes::SBC, 4, AddressingMode::Absolute, true)),
        (0xfd, (Opcodes::SBC, 4, AddressingMode::AbsoluteX, true)),
        (0xf9, (Opcodes::SBC, 4, AddressingMode::AbsoluteY, true)),
        (0xe1, (Opcodes::SBC, 6, AddressingMode::IndirectX, true)),
        (0xf1, (Opcodes::SBC, 5, AddressingMode::IndirectY, true)),

        (0x38, (Opcodes::SEC, 2, AddressingMode::Implied, true)),

        (0xf8, (Opcodes::SED, 2, AddressingMode::Implied, true)),

        (0x78, (Opcodes::SEI, 2, AddressingMode::Implied, true)),

        (0x85, (Opcodes::STA, 3, AddressingMode::ZeroPage, true)),
        (0x95, (Opcodes::STA, 4, AddressingMode::ZeroPageX, true)),
        (0x8D, (Opcodes::STA, 4, AddressingMode::Absolute, true)),
        (0x9D, (Opcodes::STA, 5, AddressingMode::AbsoluteX, true)),
        (0x99, (Opcodes::STA, 5, AddressingMode::AbsoluteY, true)),
        (0x81, (Opcodes::STA, 6, AddressingMode::IndirectX, true)),
        (0x91, (Opcodes::STA, 6, AddressingMode::IndirectY, true)),

        (0x86, (Opcodes::STX, 3, AddressingMode::ZeroPage, true)),
        (0x96, (Opcodes::STX, 4, AddressingMode::ZeroPageY, true)),
        (0x8e, (Opcodes::STX, 4, AddressingMode::Absolute, true)),

        (0x84, (Opcodes::STY, 3, AddressingMode::ZeroPage, true)),
        (0x94, (Opcodes::STY, 4, AddressingMode::ZeroPageX, true)),
        (0x8c, (Opcodes::STY, 4, AddressingMode::Absolute, true)),

        (0xaa, (Opcodes::TAX, 2, AddressingMode::Implied, true)),
        (0xa8, (Opcodes::TAY, 2, AddressingMode::Implied, true)),
        (0xba, (Opcodes::TSX, 2, AddressingMode::Implied, true)),
        (0x8a, (Opcodes::TXA, 2, AddressingMode::Implied, true)),
        (0x9a, (Opcodes::TXS, 2, AddressingMode::Implied, true)),
        (0x98, (Opcodes::TYA, 2, AddressingMode::Implied, true)),

        //
        // Unofficial opcodes
        //

        (0x0b, (Opcodes::AAC, 2, AddressingMode::Immediate, false)),
        (0x2b, (Opcodes::AAC, 2, AddressingMode::Immediate, false)),

        (0x87, (Opcodes::SAX, 3, AddressingMode::ZeroPage, false)),
        (0x97, (Opcodes::SAX, 4, AddressingMode::ZeroPageY, false)),
        (0x8f, (Opcodes::SAX, 4, AddressingMode::Absolute, false)),
        (0x83, (Opcodes::SAX, 6, AddressingMode::IndirectX, false)),

        (0x6b, (Opcodes::ARR, 2, AddressingMode::Immediate, false)),
        (0x4b, (Opcodes::ASR, 2, AddressingMode::Immediate, false)),

        (0xab, (Opcodes::ATX, 2, AddressingMode::Immediate, false)),

        (0x9f, (Opcodes::AXA, 5, AddressingMode::AbsoluteY, false)),
        (0x6b, (Opcodes::AXA, 6, AddressingMode::IndirectY, false)),

        (0xcb, (Opcodes::AXS, 2, AddressingMode::Immediate, false)),

        (0xc7, (Opcodes::DCP, 5, AddressingMode::ZeroPage, false)),
        (0xd7, (Opcodes::DCP, 6, AddressingMode::ZeroPageX, false)),
        (0xcf, (Opcodes::DCP, 6, AddressingMode::Absolute, false)),
        (0xdf, (Opcodes::DCP, 7, AddressingMode::AbsoluteX, false)),
        (0xdb, (Opcodes::DCP, 7, AddressingMode::AbsoluteY, false)),
        (0xc3, (Opcodes::DCP, 8, AddressingMode::IndirectX, false)),
        (0xd3, (Opcodes::DCP, 8, AddressingMode::IndirectY, false)),

        (0x04, (Opcodes::NOP, 3, AddressingMode::ZeroPage, false)),
        (0x14, (Opcodes::NOP, 4, AddressingMode::ZeroPageX, false)),
        (0x34, (Opcodes::NOP, 4, AddressingMode::ZeroPageX, false)),
        (0x44, (Opcodes::NOP, 3, AddressingMode::ZeroPage, false)),
        (0x54, (Opcodes::NOP, 4, AddressingMode::ZeroPageX, false)),
        (0x64, (Opcodes::NOP, 3, AddressingMode::ZeroPage, false)),
        (0x74, (Opcodes::NOP, 4, AddressingMode::ZeroPageX, false)),
        (0x80, (Opcodes::NOP, 2, AddressingMode::Immediate, false)),
        (0x82, (Opcodes::NOP, 2, AddressingMode::Immediate, false)),
        (0x89, (Opcodes::NOP, 2, AddressingMode::Immediate, false)),
        (0xc2, (Opcodes::NOP, 2, AddressingMode::Immediate, false)),
        (0xd4, (Opcodes::NOP, 4, AddressingMode::ZeroPageX, false)),
        (0xe2, (Opcodes::NOP, 2, AddressingMode::Immediate, false)),
        (0xf4, (Opcodes::NOP, 4, AddressingMode::ZeroPageX, false)),

        (0xe7, (Opcodes::ISB, 5, AddressingMode::ZeroPage, false)),
        (0xf7, (Opcodes::ISB, 6, AddressingMode::ZeroPageX, false)),
        (0xef, (Opcodes::ISB, 6, AddressingMode::Absolute, false)),
        (0xff, (Opcodes::ISB, 7, AddressingMode::AbsoluteX, false)),
        (0xfb, (Opcodes::ISB, 7, AddressingMode::AbsoluteY, false)),
        (0xe3, (Opcodes::ISB, 8, AddressingMode::IndirectX, false)),
        (0xf3, (Opcodes::ISB, 8, AddressingMode::IndirectY, false)),

        (0x02, (Opcodes::KIL, 0, AddressingMode::Implied, false)),
        (0x12, (Opcodes::KIL, 0, AddressingMode::Implied, false)),
        (0x22, (Opcodes::KIL, 0, AddressingMode::Implied, false)),
        (0x32, (Opcodes::KIL, 0, AddressingMode::Implied, false)),
        (0x42, (Opcodes::KIL, 0, AddressingMode::Implied, false)),
        (0x52, (Opcodes::KIL, 0, AddressingMode::Implied, false)),
        (0x62, (Opcodes::KIL, 0, AddressingMode::Implied, false)),
        (0x72, (Opcodes::KIL, 0, AddressingMode::Implied, false)),
        (0x92, (Opcodes::KIL, 0, AddressingMode::Implied, false)),
        (0xb2, (Opcodes::KIL, 0, AddressingMode::Implied, false)),
        (0xd2, (Opcodes::KIL, 0, AddressingMode::Implied, false)),
        (0xf2, (Opcodes::KIL, 0, AddressingMode::Implied, false)),

        (0xbb, (Opcodes::LAR, 4, AddressingMode::AbsoluteY, false)),

        (0xa7, (Opcodes::LAX, 3, AddressingMode::ZeroPage, false)),
        (0xb7, (Opcodes::LAX, 4, AddressingMode::ZeroPageY, false)),
        (0xaf, (Opcodes::LAX, 4, AddressingMode::Absolute, false)),
        (0xbf, (Opcodes::LAX, 4, AddressingMode::AbsoluteY, false)),
        (0xa3, (Opcodes::LAX, 6, AddressingMode::IndirectX, false)),
        (0xb3, (Opcodes::LAX, 5, AddressingMode::IndirectY, false)),

        (0x1a, (Opcodes::NOP, 2, AddressingMode::Implied, false)),
        (0x3a, (Opcodes::NOP, 2, AddressingMode::Implied, false)),
        (0x5a, (Opcodes::NOP, 2, AddressingMode::Implied, false)),
        (0x7a, (Opcodes::NOP, 2, AddressingMode::Implied, false)),
        (0xda, (Opcodes::NOP, 2, AddressingMode::Implied, false)),
        (0xfa, (Opcodes::NOP, 2, AddressingMode::Implied, false)),

        (0x27, (Opcodes::RLA, 5, AddressingMode::ZeroPage, false)),
        (0x37, (Opcodes::RLA, 6, AddressingMode::ZeroPageX, false)),
        (0x2f, (Opcodes::RLA, 6, AddressingMode::Absolute, false)),
        (0x3f, (Opcodes::RLA, 7, AddressingMode::AbsoluteX, false)),
        (0x3b, (Opcodes::RLA, 7, AddressingMode::AbsoluteY, false)),
        (0x23, (Opcodes::RLA, 8, AddressingMode::IndirectX, false)),
        (0x33, (Opcodes::RLA, 8, AddressingMode::IndirectY, false)),

        (0x67, (Opcodes::RRA, 5, AddressingMode::ZeroPage, false)),
        (0x77, (Opcodes::RRA, 6, AddressingMode::ZeroPageX, false)),
        (0x6f, (Opcodes::RRA, 6, AddressingMode::Absolute, false)),
        (0x7f, (Opcodes::RRA, 7, AddressingMode::AbsoluteX, false)),
        (0x7b, (Opcodes::RRA, 7, AddressingMode::AbsoluteY, false)),
        (0x63, (Opcodes::RRA, 8, AddressingMode::IndirectX, false)),
        (0x73, (Opcodes::RRA, 8, AddressingMode::IndirectY, false)),

        (0xeb, (Opcodes::SBC, 2, AddressingMode::Immediate, false)),

        (0x07, (Opcodes::SLO, 5, AddressingMode::ZeroPage, false)),
        (0x17, (Opcodes::SLO, 6, AddressingMode::ZeroPageX, false)),
        (0x0f, (Opcodes::SLO, 6, AddressingMode::Absolute, false)),
        (0x1f, (Opcodes::SLO, 7, AddressingMode::AbsoluteX, false)),
        (0x1b, (Opcodes::SLO, 7, AddressingMode::AbsoluteY, false)),
        (0x03, (Opcodes::SLO, 8, AddressingMode::IndirectX, false)),
        (0x13, (Opcodes::SLO, 8, AddressingMode::IndirectY, false)),

        (0x47, (Opcodes::SRE, 5, AddressingMode::ZeroPage, false)),
        (0x57, (Opcodes::SRE, 6, AddressingMode::ZeroPageX, false)),
        (0x4f, (Opcodes::SRE, 6, AddressingMode::Absolute, false)),
        (0x5f, (Opcodes::SRE, 7, AddressingMode::AbsoluteX, false)),
        (0x5b, (Opcodes::SRE, 7, AddressingMode::AbsoluteY, false)),
        (0x43, (Opcodes::SRE, 8, AddressingMode::IndirectX, false)),
        (0x53, (Opcodes::SRE, 8, AddressingMode::IndirectY, false)),

        (0x9e, (Opcodes::SXA, 4, AddressingMode::AbsoluteY, false)),

        (0x9c, (Opcodes::SYA, 4, AddressingMode::AbsoluteY, false)),

        (0x0c, (Opcodes::NOP, 4, AddressingMode::Absolute, false)),
        (0x1c, (Opcodes::NOP, 4, AddressingMode::AbsoluteX, false)),
        (0x3c, (Opcodes::NOP, 4, AddressingMode::AbsoluteX, false)),
        (0x5c, (Opcodes::NOP, 4, AddressingMode::AbsoluteX, false)),
        (0x7c, (Opcodes::NOP, 4, AddressingMode::AbsoluteX, false)),
        (0xdc, (Opcodes::NOP, 4, AddressingMode::AbsoluteX, false)),
        (0xfc, (Opcodes::NOP, 4, AddressingMode::AbsoluteX, false)),

        (0x8b, (Opcodes::XAA, 2, AddressingMode::Immediate, false)),

        (0x8b, (Opcodes::XAS, 5, AddressingMode::AbsoluteY, false)),
    ]);

    pub static ref MODE2BYTES: HashMap<AddressingMode, u16> = HashMap::from([
        (AddressingMode::Accumulator, 0),
        (AddressingMode::Immediate, 1),
        (AddressingMode::ZeroPage, 1),
        (AddressingMode::ZeroPageX, 1),
        (AddressingMode::ZeroPageY, 1),
        (AddressingMode::Absolute, 2),
        (AddressingMode::AbsoluteX, 2),
        (AddressingMode::AbsoluteY, 2),
        (AddressingMode::Indirect, 2),
        (AddressingMode::IndirectX, 1),
        (AddressingMode::IndirectY, 1),
        (AddressingMode::Relative, 1),
        (AddressingMode::Implied, 0),
    ]);
}
