

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

// These are the diffrent ways that an instruction can address data
pub enum AddressingMode
{
    /// The address is implied by the instruction
    Implicit,

    /// operates on the accumulator
    Accumulate,

    /// just an 8 bit constant for the address
    Immediate,
    
    /// 8 bit operand limiting it to the first 256 bytes of memory
    /// the most significant byte is always 0
    ZeroPage,

    /// same as zero page, but it adds whatever is in the x register to the address
    ZeroPageX,

    /// same as zero page, but it adds whatever is in the y register to the address
    ZeroPageY,

    /// contain a signed 8 bit relative offset which is added to the program counter.
    Relative,

    /// Contain the full 16 byte address
    Absolute,

    /// Adds the 16 byte address with the x register
    AbsoluteX,

    /// Adds the 16 byte address with the y register
    AbsoluteY,

    /// its a 16 bit address that identifies the location of the least significant byte
    /// of another 16 bit memory address which is the real target of the instruction
    Indirect,

    /// same as indirect but we add the x register
    IndexedIndirect,

    /// same as indirect but we add the y register
    IndirectIndexed,
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

  fn read_mem(&mut self, mode: AddressingMode, loc: u16) -> u16
  {
    match mode {
      AddressingMode::ZeroPage => { self.memory.read_mem_u8(loc) as u16 }
      AddressingMode::ZeroPageX => { self.memory.read_mem_u8(loc + self.index_register_x as u16) as u16 }
      AddressingMode::ZeroPageY => { self.memory.read_mem_u8(loc + self.index_register_y as u16) as u16 }
      AddressingMode::Absolute => { self.memory.read_mem_u16(loc) }
      AddressingMode::AbsoluteX => { self.memory.read_mem_u16(loc + self.index_register_x as u16) }
      AddressingMode::AbsoluteY => { self.memory.read_mem_u16(loc + self.index_register_y as u16) }
      AddressingMode::Indirect => { 
          let mem_parts = [
              self.memory.read_mem_u8(loc),
              self.memory.read_mem_u8(loc+1)
          ];

          let mut mem_parts_ref = &mem_parts[..];
          mem_parts_ref.read_u16::<LittleEndian>().unwrap_or_default()
      }
      AddressingMode::IndexedIndirect => {
          let mem_parts = [
            self.memory.read_mem_u8(loc + self.index_register_x as u16),
            self.memory.read_mem_u8(1 + loc + self.index_register_x as u16)
          ];

          let mut mem_parts_ref = &mem_parts[..];
          mem_parts_ref.read_u16::<LittleEndian>().unwrap_or_default()
      }
      AddressingMode::IndirectIndexed => {
          let mem_parts = [
            self.memory.read_mem_u8(loc + self.index_register_y as u16),
            self.memory.read_mem_u8(1 + loc + self.index_register_y as u16)
          ];

          let mut mem_parts_ref = &mem_parts[..];
          mem_parts_ref.read_u16::<LittleEndian>().unwrap_or_default()
      }
      _ => return 0
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
}
