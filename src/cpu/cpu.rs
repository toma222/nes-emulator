

// as defined in http://www.6502.org/users/obelisk/6502/registers.html

use crate::cpu::processor_status::{ProcessorStatusFlags, ProcessorStatus};
use crate::cpu::memory_map::{MemoryMap};

use byteorder::{ByteOrder, LittleEndian, ReadBytesExt}; // 1.3.4

/// Defines the state of a 6502 CPU
/// just a reminder that the CPU will store data little endian <3
pub struct CPU
{
  /// Points to the next program to be executed
  pub program_counter: u16,

  /// Points to a 256 byte stack located between 0x0100 and 0x01FF
  pub stack_pointer: u8,

  /// Used for arethmetic operations
  pub accumulator: u8,

  /// holds counters and offsets for accsessing memory
  pub index_register_x: u8,

  /// same as x
  pub index_register_y: u8,

  /// Holds flags for when operations are done
  /// this u8 is controlled with the processor status flags enum
  pub processor_status: ProcessorStatus,


  memory: MemoryMap
}

// These are the different ways that an instruction can address data
pub enum AddressingMode
{
    /// just an 8 bit constant for the address
    Immediate,
    
    /// 8 bit operand limiting it to the first 256 bytes of memory
    /// the most significant byte is always 0
    ZeroPage,

    /// same as zero page, but it adds whatever is in the x register to the address
    ZeroPageX,

    /// same as zero page, but it adds whatever is in the y register to the address
    ZeroPageY,

    /// Contain the full 16 byte address
    Absolute,

    /// Adds the 16 byte address with the x register
    AbsoluteX,

    /// Adds the 16 byte address with the y register
    AbsoluteY,

    /// its a 16 bit address that identifies the location of the least significant byte
    /// of another 16 bit memory address which is the real target of the instruction
    /// then adds the x register
    IndirectX,

    /// its a 16 bit address that identifies the location of the least significant byte
    /// of another 16 bit memory address which is the real target of the instruction
    /// then adds the y register
    IndirectY,
}

impl CPU
{
  fn new() -> CPU
  {
    return CPU{
      program_counter: 0,
      stack_pointer: 0,
      accumulator: 0,
      index_register_x: 0,
      index_register_y: 0,
      processor_status: ProcessorStatus(ProcessorStatusFlags::Default as u8),
      memory: MemoryMap::new()
    }
  }

  /// Assumes the next part of the program counter is an address
  /// this function gets the address from that address using the specified addressing mode
  fn get_operand_address(&self, mode: AddressingMode) -> u16
  {
    match mode {
        AddressingMode::Immediate => self.program_counter,
        AddressingMode::ZeroPage => self.memory.read_mem_u8(self.program_counter) as u16,
        AddressingMode::ZeroPageX => self.memory.read_mem_u8(self.program_counter).wrapping_add(self.index_register_x) as u16,
        AddressingMode::ZeroPageY => self.memory.read_mem_u8(self.program_counter).wrapping_add(self.index_register_y) as u16,
        AddressingMode::Absolute => self.memory.read_mem_u16(self.program_counter),
        AddressingMode::AbsoluteX => self.memory.read_mem_u16(self.program_counter).wrapping_add(self.index_register_x as u16),
        AddressingMode::AbsoluteY => self.memory.read_mem_u16(self.program_counter).wrapping_add(self.index_register_y as u16),
        AddressingMode::IndirectX => {
          let ptr = self.memory.read_mem_u8(self.program_counter).wrapping_add(self.index_register_x);
          let lo = self.memory.read_mem_u8(ptr as u16);
          let hi = self.memory.read_mem_u8(ptr.wrapping_add(1) as u16);
          (hi as u16) << 8 | (lo as u16)
        }
        AddressingMode::IndirectY => {
          let base = self.memory.read_mem_u8(self.program_counter);

          let lo = self.memory.read_mem_u8(base as u16);
          let hi = self.memory.read_mem_u8((base as u8).wrapping_add(1) as u16);
          let deref_base = (hi as u16) << 8 | (lo as u16);
          let deref = deref_base.wrapping_add(self.index_register_y as u16);
          deref
        }
    }
  }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_new() {
        let cpu = CPU::new();
        assert_eq!(cpu.index_register_x, 0);
    }

    fn cpu_address_immediate() {
      let cpu = CPU::new();
    }
}
