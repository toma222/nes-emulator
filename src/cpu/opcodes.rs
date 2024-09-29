
use std::collections::HashMap;

use crate::cpu::cpu::AddressingMode;
use lazy_static::lazy_static;

use std::fmt::Write;

#[derive(Debug)]
pub struct OpCode
{
    pub code: u8,
    pub mnemonic: &'static str,
    pub bytes: u8,
    pub cycles: u8,
    pub addressing_mode: AddressingMode,
}

// make a way to match the mnemonic in the cpu match statement instead of the bytes
impl OpCode {
    fn new(code: u8, mnemonic: &'static str, bytes: u8, cycles: u8, addressing_mode: AddressingMode) -> OpCode {
        OpCode {
            code,
            mnemonic,
            bytes,
            cycles,
            addressing_mode,
        }
    }

    pub fn to_string(&self) -> String {
        return format!("inst: {}, code: {:#x}, addr: {:?}", self.mnemonic, self.code, self.addressing_mode);
    }
}

lazy_static! {
    static ref CPU_OPCODES: Vec<OpCode> = vec![
        OpCode::new(0xA9, "LDA", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xA5, "LDA", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xB5, "LDA", 2, 4, AddressingMode::ZeroPageX),
        OpCode::new(0xAD, "LDA", 3, 4, AddressingMode::Absolute),
        OpCode::new(0xBD, "LDA", 3, 4, AddressingMode::AbsoluteX), // +1 if page crossed
        OpCode::new(0xB9, "LDA", 3, 4, AddressingMode::AbsoluteY), // +1 if page crossed
        OpCode::new(0xA1, "LDA", 2, 6, AddressingMode::IndirectX),
        OpCode::new(0xB1, "LDA", 2, 5, AddressingMode::IndirectY), // +1 if page crossed
        
        OpCode::new(0xA2, "LDX", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xA6, "LDX", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xB6, "LDX", 2, 4, AddressingMode::ZeroPageX),
        OpCode::new(0xAE, "LDX", 3, 4, AddressingMode::Absolute),
        OpCode::new(0xBE, "LDX", 3, 4, AddressingMode::AbsoluteY), // +1 if page crossed

        OpCode::new(0xA0, "LDY", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xA4, "LDY", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xB4, "LDY", 2, 4, AddressingMode::ZeroPageX),
        OpCode::new(0xAC, "LDY", 3, 4, AddressingMode::Absolute),
        OpCode::new(0xBC, "LDY", 3, 4, AddressingMode::AbsoluteX), // +1 if page crossed

        // adc - add with carry
        OpCode::new(0x69, "ADC", 2, 2, AddressingMode::Immediate),
        OpCode::new(0x65, "ADC", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x75, "ADC", 2, 4, AddressingMode::ZeroPageX),
        OpCode::new(0x6D, "ADC", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x7D, "ADC", 3, 4, AddressingMode::AbsoluteX), // +1 if page crossed
        OpCode::new(0x79, "ADC", 3, 4, AddressingMode::AbsoluteY), // +1 if page crossed
        OpCode::new(0x61, "ADC", 2, 6, AddressingMode::IndirectX),
        OpCode::new(0x71, "ADC", 2, 5, AddressingMode::IndirectY), // +1 if page crossed

        // and - logical and
        OpCode::new(0x29, "AND", 2, 2, AddressingMode::Immediate),
        OpCode::new(0x25, "AND", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x35, "AND", 2, 4, AddressingMode::ZeroPageX),
        OpCode::new(0x2D, "AND", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x3D, "AND", 3, 4, AddressingMode::AbsoluteX), // +1 if page crossed
        OpCode::new(0x39, "AND", 3, 4, AddressingMode::AbsoluteY), // +1 if page crossed
        OpCode::new(0x21, "AND", 2, 6, AddressingMode::IndirectX),
        OpCode::new(0x31, "AND", 2, 5, AddressingMode::IndirectY), // +1 if page crossed

        // asl - arithmetic shift left (<<)
        OpCode::new(0x0A, "ASL", 1, 2, AddressingMode::Accumulator),
        OpCode::new(0x06, "ASL", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0x16, "ASL", 2, 6, AddressingMode::ZeroPageX),
        OpCode::new(0x0E, "ASL", 3, 6, AddressingMode::Absolute),
        OpCode::new(0x1E, "ASL", 3, 7, AddressingMode::AbsoluteX),

        // A lot of branch operations that depend on cpu flags
        OpCode::new(0x90, "BCC", 2, 2, AddressingMode::Relative),
        OpCode::new(0xB0, "BCS", 2, 2, AddressingMode::Relative),
        OpCode::new(0xF0, "BEQ", 2, 2, AddressingMode::Relative),
        OpCode::new(0xD0, "BNE", 2, 2, AddressingMode::Relative),
        OpCode::new(0x10, "BPL", 2, 2, AddressingMode::Relative),
        OpCode::new(0x30, "BMI", 2, 2, AddressingMode::Relative),
        OpCode::new(0x50, "BVC", 2, 2, AddressingMode::Relative),
        OpCode::new(0x70, "BVS", 2, 2, AddressingMode::Relative),

        // clearing flags
        OpCode::new(0x17, "CLC", 1, 2, AddressingMode::NoneAddressing), // carry
        OpCode::new(0xD8, "CLD", 1, 2, AddressingMode::NoneAddressing), // decimal
        OpCode::new(0x58, "CLI", 1, 2, AddressingMode::NoneAddressing), // interrupt
        OpCode::new(0xB8, "CLV", 1, 2, AddressingMode::NoneAddressing), // overflow

        // setting flags
        OpCode::new(0x38, "SEC", 1, 2, AddressingMode::NoneAddressing), // carry
        OpCode::new(0xF8, "SED", 1, 2, AddressingMode::NoneAddressing), // decimal
        OpCode::new(0x78, "SEI", 1, 2, AddressingMode::NoneAddressing), // interrupt

        // Comparing instructions
        OpCode::new(0xC9, "CMP", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xC5, "CMP", 2, 2, AddressingMode::ZeroPage),
        OpCode::new(0xD5, "CMP", 2, 2, AddressingMode::ZeroPageX),
        OpCode::new(0xCD, "CMP", 3, 3, AddressingMode::Absolute),
        OpCode::new(0xDD, "CMP", 3, 3, AddressingMode::AbsoluteX), // +1 if page crossed
        OpCode::new(0xD9, "CMP", 3, 3, AddressingMode::AbsoluteY), // +1 if page crossed
        OpCode::new(0xC1, "CMP", 2, 2, AddressingMode::IndirectX),
        OpCode::new(0xD1, "CMP", 2, 2, AddressingMode::IndirectY), // +1 if page crossed

        // Compare x register
        OpCode::new(0xE0, "CPX", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xE4, "CPX", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xEC, "CPX", 3, 4, AddressingMode::Absolute),

        // Compare y register
        OpCode::new(0xC0, "CPY", 2, 2, AddressingMode::Immediate),
        OpCode::new(0xC4, "CPY", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0xCC, "CMY", 3, 4, AddressingMode::Absolute),

        // bit test
        OpCode::new(0x24, "BIT", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x2C, "BIT", 3, 4, AddressingMode::Absolute),

        OpCode::new(0x00, "BRK", 1, 7, AddressingMode::NoneAddressing),
        OpCode::new(0xAA, "TAX", 1, 2, AddressingMode::NoneAddressing),
    
        // Decrement
        // Decrements 1 from a point in memory
        OpCode::new(0xC6, "DEC", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0xD6, "DEC", 2, 5, AddressingMode::ZeroPageX),
        OpCode::new(0xCE, "DEC", 2, 5, AddressingMode::Absolute),
        OpCode::new(0xDE, "DEC", 2, 5, AddressingMode::AbsoluteX),

        // Decrement 1 from the x register
        OpCode::new(0xCA, "DEX", 1, 2, AddressingMode::NoneAddressing),

        // Decrement 1 from the y register
        OpCode::new(0x88, "DEY", 1, 2, AddressingMode::NoneAddressing),
    
        // Exclusive or
        OpCode::new(0x49, "EOR", 2, 2, AddressingMode::Immediate),
        OpCode::new(0x45, "EOR", 2, 3, AddressingMode::ZeroPage),
        OpCode::new(0x55, "EOR", 2, 4, AddressingMode::ZeroPageX),
        OpCode::new(0x4D, "EOR", 3, 4, AddressingMode::Absolute),
        OpCode::new(0x5D, "EOR", 3, 4, AddressingMode::AbsoluteX), // +1 if page crossed
        OpCode::new(0x59, "EOR", 3, 4, AddressingMode::AbsoluteY), // +1 if page crossed
        OpCode::new(0x41, "EOR", 2, 6, AddressingMode::IndirectX),
        OpCode::new(0x51, "EOR", 2, 5, AddressingMode::IndirectY), // +1 if page crossed
    
        // adds one to a given memory address
        OpCode::new(0xE6, "INC", 2, 5, AddressingMode::ZeroPage),
        OpCode::new(0xF6, "INC", 2, 6, AddressingMode::ZeroPageX),
        OpCode::new(0xEE, "INC", 3, 6, AddressingMode::Absolute),
        OpCode::new(0xFE, "INC", 3, 7, AddressingMode::AbsoluteX),

        // x += 1
        OpCode::new(0xE8, "INX", 1, 2, AddressingMode::NoneAddressing),

        // y += 1
        OpCode::new(0xC8, "INY", 1, 2, AddressingMode::NoneAddressing),

        // jump
        OpCode::new(0x4C, "JMP", 3, 3, AddressingMode::Absolute),
        OpCode::new(0x6C, "JMP", 3, 3, AddressingMode::Indirect),
    
        // jump to subroutine
        OpCode::new(0x20, "JSR", 3, 6, AddressingMode::Absolute),

        // return from subroutine
        OpCode::new(0x60, "RTS", 1, 6, AddressingMode::NoneAddressing),
    ];

    pub static ref OPCODES_MAP: HashMap<u8, &'static OpCode> = {
        let mut map = HashMap::new();
        for code in &*CPU_OPCODES {
            map.insert(code.code, code);
        }
        map
    };
    
}

