

// as defined in http://www.6502.org/users/obelisk/6502/registers.html

use log::{info, trace};

use crate::cpu::processor_status::{ProcessorStatusFlags, ProcessorStatus};
use crate::cpu::memory_map::MemoryMap;

use super::opcodes::OPCODES_MAP; // 1.3.4

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


  pub memory: MemoryMap
}

// These are the different ways that an instruction can address data
#[derive(Debug)]
pub enum AddressingMode
{
    NoneAddressing,

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
  pub fn new() -> CPU
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

  pub fn log_dump_registers_string(&self) -> String {
    return format!("prgm_ctr: {:#x} | stk_ptr: {:#x} | acc_reg: {:#x} | ind_reg_x: {:#x} | ind_reg_y: {:#x} |
                    cpu_state_flags {}",
     self.program_counter, self.stack_pointer, self.accumulator, self.index_register_x, self.index_register_y, self.processor_status);
  }

  /// Assumes the next part of the program counter is an address
  /// this function gets the address from that address using the specified addressing mode
  fn get_operand_address(&self, mode: &AddressingMode) -> u16
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

      AddressingMode::NoneAddressing => {
        panic!("mode {:?} is not supported", mode);
      }
    }
  }

  /// Resets all the registers and gets the first instruction of the program
  pub fn reset(&mut self)
  {
    self.index_register_x = 0;
    self.index_register_y = 0;
    self.processor_status.reset_flags();

    // Get the start of the program from the program address
    self.program_counter = self.memory.read_mem_u16(0xFFFC);
  }

  /// Loads the program at 0x8000 in the address space.
  /// it then writes the first instruction (0x8000) to 0xFFFC.
  /// 0xFFFC is were the program looks for the fist instruction of the program
  pub fn load_program(&mut self, program: Vec<u8>) {
    self.memory.memory[0x8000 .. (0x8000 + program.len())].copy_from_slice(&program[..]);
    self.memory.write_mem_u16(0xFFFC, 0x8000);
  }

  pub fn load_and_run_program(&mut self, program: Vec<u8>) {
    self.load_program(program);
    self.reset();
    self.run_program();
  }

  pub fn run_program(&mut self) {
    loop {
      let code = self.memory.read_mem_u8(self.program_counter);
      self.program_counter += 1; // consume the read instruction and point to the next

      let opcode = OPCODES_MAP.get(&code).expect(&format!("OpCode {:x} is not recognized", code));
      let program_counter_state = self.program_counter;

      info!("prg_c: {:#x} | {}", self.program_counter, opcode.to_string());

      match code {
          // LDA opcode
          0xA9 | 0xA5  | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 => {
            self.lda(&opcode.addressing_mode);
          } 

          // adc opcode
          0x69 | 0x65 | 0x75 | 0x6D | 0x7D | 0x79 | 0x61 | 0x71 => {
            self.lda(&opcode.addressing_mode);
          }

          0x00 => return,
          _ => todo!()
      }

      // increment the program counter
      if program_counter_state == self.program_counter {
        self.program_counter += (opcode.bytes - 1) as u16;
      }
    }
  }
}

impl CPU {

  /// Helper function that loads field data into register A
  fn add_to_register_a(&mut self, data: u8) {
    let sum = self.accumulator as u16
      + data as u16 
      + (if self.processor_status.has_flag_set(ProcessorStatusFlags::CarryFlag) {
        1
      } else {
        0
      }) as u16;

      let carry = sum > 0xff;

      if carry {
        self.processor_status.set_flag_true(ProcessorStatusFlags::CarryFlag);
      } else {
        self.processor_status.set_flag_false(ProcessorStatusFlags::CarryFlag);
      }

      let result = sum as u8;

      // checks if the sign bit changed
      if(data ^ result) & (result ^ self.accumulator) & 0x80 != 0 {
        self.processor_status.set_flag_true(ProcessorStatusFlags::Overflow);
      } else {
        self.processor_status.set_flag_false(ProcessorStatusFlags::Overflow);
      }

      self.set_register_a(data);
  }

  fn set_register_a(&mut self, data: u8) {
    self.accumulator = data;
    self.processor_status.update_zero_and_negative_flags(self.accumulator);
  }


  /// Add with carry
  /// Adds the contents of a memory location to the accumulator with the carry bit
  /// if it overflows then we set the carry bit
  fn adc(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.memory.read_mem_u8(addr);

    self.add_to_register_a(value);
  }

  /// Loads a value into the a register
  fn lda(&mut self, mode: &AddressingMode) {
    let addr = self.get_operand_address(mode);
    let value = self.memory.read_mem_u8(addr);

    self.set_register_a(value);
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

    #[test]
    fn cpu_lda_from_memory() {
      let mut cpu = CPU::new();
      cpu.memory.write_mem_u8(0x10, 0xF1); // this should set off the negative flag
      cpu.memory.write_mem_u8(0x11, 0x00); // this should set off the zero flag

      // test the negative flag
      cpu.load_and_run_program(vec![0xa5, 0x10, 0x00]);
      assert_eq!(cpu.accumulator, 0xF1);
      assert_eq!(cpu.processor_status.has_flag_set(ProcessorStatusFlags::Negative), true);

      // test the positive flag
      cpu.load_and_run_program(vec![0xa5, 0x11, 0x00]);
      assert_eq!(cpu.accumulator, 0x00);
      assert_eq!(cpu.processor_status.has_flag_set(ProcessorStatusFlags::ZeroFlag), true);
    }

    #[test]
    fn cpu_adc_from_memory() {
      let mut cpu = CPU::new();
      cpu.memory.write_mem_u8(0x10, 0x05); // this should set off the zero flag

      cpu.load_and_run_program(vec![0x65, 0x10, 0x65, 0x10]);
      assert_eq!(cpu.accumulator, 0x10);

      // test the overflow
      cpu.memory.write_mem_u8(0x12, 0xFE);
      cpu.memory.write_mem_u8(0x13, 0x02);

      cpu.load_and_run_program(vec![0x65, 0x12, 0x65, 0x13]);
      assert_eq!(cpu.processor_status.has_flag_set(ProcessorStatusFlags::Overflow), true);
    }
}
