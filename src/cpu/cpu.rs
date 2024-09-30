// as defined in http://www.6502.org/users/obelisk/6502/registers.html

use log::{trace, warn};

// use crate::cpu::memory_map::MemoryMap;
use crate::cpu::bus::Bus;
use crate::cpu::memory::Mem;
use crate::cpu::processor_status::{ProcessorStatus, ProcessorStatusFlags};

use super::opcodes::OPCODES_MAP; // 1.3.4

/// This defines the memory
/// and has some implementations for managing that memory
/// This holds the memory. All of it <3.
/// The first 256 byte page of memory (0x0000 - 0x00FF) is Zero Page
/// The second page (0x0100-0x01FF) is the system stack
/// The other reserved parts of the memory map is 0xFFFA to 0xFFFF
/// that part has to be programed with the interrupt handler (0xFFFA/B)
/// the power reset location and the BRK/interrupt request handler
/*
pub struct MemoryMap {
    pub memory: [u8; 0xFFFF],
}

impl MemoryMap {
    pub fn new() -> MemoryMap {
        MemoryMap {
            memory: [0; 0xFFFF],
        }
    }
}

impl Mem for MemoryMap {
    fn read_mem_u8(&self, loc: u16) -> u8 {
        return self.memory[loc as usize];
    }

    fn write_mem_u8(&mut self, loc: u16, data: u8) {
        self.memory[loc as usize] = data;
    }
}
*/

/// Defines the state of a 6502 CPU
/// just a reminder that the CPU will store data little endian <3
pub struct CPU {
    /// Points to the next program to be executed
    pub program_counter: u16,

    /// Points to a 256 byte stack located between 0x0100 and 0x01FF
    /// Starts at 0x01FF and decreases down
    pub stack_pointer: u16,

    /// This is the offset that we SUBTRACT ( the stack moves DOWN )
    /// from the stack pointer in order to find were to place the next byte of memory
    stack_base: u8,

    /// Used for arethmetic operations
    pub accumulator: u8,

    /// holds counters and offsets for accsessing memory
    pub index_register_x: u8,

    /// same as x
    pub index_register_y: u8,

    /// Holds flags for when operations are done
    /// this u8 is controlled with the processor status flags enum
    pub processor_status: ProcessorStatus,

    pub bus: Bus,
}

/// Forward all memory operations to the bus
impl Mem for CPU {
    fn read_mem_u8(&self, addr: u16) -> u8 {
        self.bus.read_mem_u8(addr)
    }

    fn write_mem_u8(&mut self, addr: u16, data: u8) {
        self.bus.write_mem_u8(addr, data)
    }
}

// These are the different ways that an instruction can address data
#[derive(Debug)]
pub enum AddressingMode {
    NoneAddressing,

    /// The operation operated on the accumulator register
    Accumulator,

    /// Relitave contains a signed 8 bit address that we increment the program counter with
    Relative,

    /// just an 8 bit constant as your parameter
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

    /// Only the jmp instruction uses this
    /// it points to a least sig byte that is used to identify a 16 bit address
    Indirect,

    /// its a 16 bit address that identifies the location of the least significant byte
    /// of another 16 bit memory address which is the real target of the instruction
    /// then adds the x register
    IndirectX,

    /// its a 16 bit address that identifies the location of the least significant byte
    /// of another 16 bit memory address which is the real target of the instruction
    /// then adds the y register
    IndirectY,
}

impl CPU {
    pub fn new(bus: Bus) -> CPU {
        return CPU {
            program_counter: 0,
            stack_pointer: 0x01FF,
            stack_base: 0,
            accumulator: 0,
            index_register_x: 0,
            index_register_y: 0,
            processor_status: ProcessorStatus(ProcessorStatusFlags::Default as u8),
            bus,
            // memory: MemoryMap::new(),
        };
    }

    pub fn log_dump_registers_string(&self) -> String {
        return format!("prgm_ctr: {:#x} | stk_ptr: {:#x} | acc_reg: {:#x} | ind_reg_x: {:#x} | ind_reg_y: {:#x} |
                    cpu_state_flags: {}",
                    self.program_counter, self.stack_pointer, self.accumulator, self.index_register_x, self.index_register_y,
                    self.processor_status);
    }

    /// Assumes the next part of the program counter is an address
    /// this function gets the address from that address using the specified addressing mode
    fn get_operand_address(&self, mode: &AddressingMode) -> u16 {
        match mode {
            AddressingMode::Immediate => self.program_counter,
            AddressingMode::Relative => self.program_counter,
            AddressingMode::ZeroPage => self.read_mem_u8(self.program_counter) as u16,
            AddressingMode::ZeroPageX => {
                self.read_mem_u8(self.program_counter)
                    .wrapping_add(self.index_register_x) as u16
            }
            AddressingMode::ZeroPageY => {
                self.read_mem_u8(self.program_counter)
                    .wrapping_add(self.index_register_y) as u16
            }
            AddressingMode::Absolute => self.read_mem_u16(self.program_counter),
            AddressingMode::AbsoluteX => self
                .read_mem_u16(self.program_counter)
                .wrapping_add(self.index_register_x as u16),
            AddressingMode::AbsoluteY => self
                .read_mem_u16(self.program_counter)
                .wrapping_add(self.index_register_y as u16),
            AddressingMode::Indirect => {
                let ptr = self.read_mem_u8(self.program_counter);
                let lo = self.read_mem_u8(ptr as u16);
                let hi = self.read_mem_u8(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
            }
            AddressingMode::IndirectX => {
                let base = self.read_mem_u8(self.program_counter);
                let ptr: u8 = (base as u8).wrapping_add(self.index_register_x);
                let lo = self.read_mem_u8(ptr as u16);
                let hi = self.read_mem_u8(ptr.wrapping_add(1) as u16);
                (hi as u16) << 8 | (lo as u16)
            }
            AddressingMode::IndirectY => {
                let base = self.read_mem_u8(self.program_counter);
                let lo = self.read_mem_u8(base as u16);
                let hi = self.read_mem_u8((base as u8).wrapping_add(1) as u16);
                let deref_base = (hi as u16) << 8 | (lo as u16);
                let deref = deref_base.wrapping_add(self.index_register_y as u16);
                deref
            }

            AddressingMode::NoneAddressing | AddressingMode::Accumulator => {
                panic!("mode {:?} is not supported", mode);
            }
        }
    }

    /// Resets all the registers and gets the first instruction of the program
    pub fn reset(&mut self) {
        self.index_register_x = 0;
        self.index_register_y = 0;
        self.processor_status.reset_flags();

        // Get the start of the program from the program address
        self.program_counter = self.read_mem_u16(0xFFFC);
    }

    /// Loads the program at 0x8000 in the address space.
    /// it then writes the first instruction (0x8000) to 0xFFFC.
    /// 0xFFFC is were the program looks for the fist instruction of the program
    pub fn load_program(&mut self, program: Vec<u8>) {
        for i in 0..(program.len() as u16) {
            self.write_mem_u8(0x0600 + i, program[i as usize]);
        }

        // self.write_mem_u16(0xFFFC, 0x0600);
    }

    // The tests use this function
    pub fn load_and_run_program(&mut self, program: Vec<u8>) {
        self.load_program(program);
        self.reset();
        self.program_counter = 0x0600;
        self.run();
    }

    // The tests use this function
    pub fn run(&mut self) {
        self.run_with_callback(|_| {});
    }

    pub fn run_with_callback<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut CPU),
    {
        loop {
            callback(self);

            let code = self.read_mem_u8(self.program_counter);
            self.program_counter += 1; // consume the read instruction and point to the next

            let opcode = OPCODES_MAP.get(&code).expect(&format!(
                "OpCode {:x} is not recognized. dumping CPU\n {}",
                code,
                self.log_dump_registers_string()
            ));
            let program_counter_state = self.program_counter;

            trace!(
                "prg_c: {:#x} | {}",
                self.program_counter - 1,
                opcode.to_string()
            );

            match code {
                /* ------ LOAD INSTRUCTIONS ------ */
                // LDA opcode
                0xA9 | 0xA5 | 0xB5 | 0xAD | 0xBD | 0xB9 | 0xA1 | 0xB1 => {
                    self.lda(&opcode.addressing_mode);
                }

                // ldx
                0xA2 | 0xA6 | 0xB6 | 0xAE | 0xBE => {
                    self.ldx(&opcode.addressing_mode);
                }

                // ldy
                0xA0 | 0xA4 | 0xB4 | 0xAC | 0xBC => {
                    self.ldy(&opcode.addressing_mode);
                }

                /* ------ ADDING + SUBTRACTING (WITH CARRY) INSTRUCTIONS ------ */
                0x69 | 0x65 | 0x75 | 0x6D | 0x7D | 0x79 | 0x61 | 0x71 => {
                    self.adc(&opcode.addressing_mode)
                }

                0xE9 | 0xE5 | 0xF5 | 0xED | 0xFD | 0xF9 | 0xE1 | 0xF1 => {
                    self.sbc(&opcode.addressing_mode)
                }

                /* ------ LOGICAL BIT OPERATIONS ------ */
                // and opcode
                0x29 | 0x25 | 0x35 | 0x2D | 0x3D | 0x39 | 0x21 | 0x31 => {
                    self.and(&opcode.addressing_mode)
                }

                // right bit shift
                0x4A => self.lsr_acc(),
                0x46 | 0x56 | 0x4E | 0x5E => self.lsr(&opcode.addressing_mode),

                // left bit shift
                0x0A => self.asl_acc(),
                0x06 | 0x16 | 0x0E | 0x1E => self.asl(&opcode.addressing_mode),

                // Or operation
                0x09 | 0x05 | 0x15 | 0x0D | 0x1D | 0x19 | 0x01 | 0x11 => {
                    self.ora(&opcode.addressing_mode)
                }

                // Exclusive or
                0x49 | 0x45 | 0x55 | 0x4D | 0x5D | 0x59 | 0x41 | 0x51 => {
                    self.eor(&opcode.addressing_mode)
                }

                /* ------ BRANCH OPERATIONS ------ */
                // Branch if flag set/not set crap
                0xB0 => self.bcs(),
                0xF0 => self.beq(),
                0xD0 => self.bne(),
                0x10 => self.bpl(),
                0x30 => self.bmi(),
                0x50 => self.bvc(),
                0x70 => self.bvs(),
                0x90 => self.bcc(),

                // Absolute addressing
                0x4C => {
                    let mem_address = self.read_mem_u16(self.program_counter);
                    self.program_counter = mem_address;
                }

                // Indirect addressing
                0x6C => {
                    let mem_address = self.read_mem_u16(self.program_counter);
                    //6502 bug mode with with page boundary:
                    //  if address $3000 contains $40, $30FF contains $80, and $3100 contains $50,
                    // the result of JMP ($30FF) will be a transfer of control to $4080 rather than $5080 as you intended
                    // i.e. the 6502 took the low byte of the address from $30FF and the high byte from $3000

                    let indirect_ref = if mem_address & 0x00FF == 0x00FF {
                        let lo = self.read_mem_u8(mem_address);
                        let hi = self.read_mem_u8(mem_address & 0xFF00);
                        (hi as u16) << 8 | (lo as u16)
                    } else {
                        self.read_mem_u16(mem_address)
                    };

                    self.program_counter = indirect_ref;
                }

                // jump and return from subroutine
                // 0x20 => self.jsr(&AddressingMode::Absolute),
                0x20 => {
                    self.push_stack_u16(self.program_counter + 2 - 1);
                    let target_address = self.read_mem_u16(self.program_counter);
                    self.program_counter = target_address;
                }

                // return from subroutine
                0x60 => {
                    self.program_counter = self.read_stack_u16() + 1;
                }

                /* ------ STATUS OPERATIONS ------ */
                // clear flags
                0x18 => self
                    .processor_status
                    .set_flag_false(ProcessorStatusFlags::CarryFlag),
                0xD8 => self
                    .processor_status
                    .set_flag_false(ProcessorStatusFlags::DecimalMode),
                0x58 => self
                    .processor_status
                    .set_flag_false(ProcessorStatusFlags::InterruptDisable),
                0xB8 => self
                    .processor_status
                    .set_flag_false(ProcessorStatusFlags::Overflow),

                // set flags
                0x38 => self
                    .processor_status
                    .set_flag_true(ProcessorStatusFlags::CarryFlag),
                0xF8 => self
                    .processor_status
                    .set_flag_true(ProcessorStatusFlags::DecimalMode),
                0x78 => self
                    .processor_status
                    .set_flag_true(ProcessorStatusFlags::InterruptDisable),

                /* ------ COMPARING INSTRUCTIONS ------ */
                // bit
                0x24 | 0x2C => self.bit(&opcode.addressing_mode),

                // cmp
                0xC9 | 0xC5 | 0xD5 | 0xCD | 0xDD | 0xD9 | 0xC1 | 0xD1 => {
                    self.cmp(&opcode.addressing_mode)
                }

                // cmx
                0xE0 | 0xE4 | 0xEC => self.cpx(&opcode.addressing_mode),

                // cmy
                0xC0 | 0xC4 | 0xCC => self.cpy(&opcode.addressing_mode),

                /* ------ TRANSFER INSTRUCTIONS ------ */
                // transfer operations
                0xAA => self.set_register_x(self.accumulator), // { self.index_register_x = self.accumulator; self.processor_status.update_zero_and_negative_flags(self.index_register_x); },
                0xA8 => self.set_register_y(self.accumulator), //{ self.index_register_y = self.accumulator; self.processor_status.update_zero_and_negative_flags(self.index_register_y); },
                0xBA => self.set_register_x(
                    self.read_mem_u8(self.stack_pointer - self.stack_base.wrapping_sub(1) as u16),
                ),
                0x8A => self.set_register_a(self.index_register_x), //{ self.accumulator = self.index_register_x; self.processor_status.update_zero_and_negative_flags(self.accumulator); } ,
                0x9A => self.set_register_y(
                    self.read_mem_u8(self.stack_pointer - self.stack_base.wrapping_sub(1) as u16),
                ), // self.index_register_y = self.read_mem_u8(self.stack_pointer - self.stack_base.wrapping_sub(1) as u16),
                0x98 => self.set_register_a(self.index_register_y), //{ self.accumulator = self.index_register_y; self.processor_status.update_zero_and_negative_flags(self.accumulator); },

                /* ------ INCREMENT AND DECREMENT INSTRUCTIONS ------ */
                // INC
                0xE6 | 0xF6 | 0xEE | 0xFE => self.inc(&opcode.addressing_mode),
                0xE8 => self.set_register_x(self.index_register_x.wrapping_add(1)),
                0xC8 => self.set_register_y(self.index_register_y.wrapping_add(1)),

                // dec
                0xC6 | 0xD6 | 0xCE | 0xDE => self.dec(&opcode.addressing_mode), // from point in memory
                0xCA => self.set_register_x(self.index_register_x.wrapping_sub(1)),
                0x88 => self.set_register_y(self.index_register_y.wrapping_sub(1)),

                /* ------ STACK OPERATIONS ------ */
                // push the accumulator onto the stack
                0x48 => self.push_stack(self.accumulator),

                0x08 => self.accumulator = self.pop_stack(),

                0x28 => self.processor_status.0 = self.pop_stack(),

                /* ------ ROTATE OPERATIONS ------ */
                0x2A => self.rol_acc(),
                0x26 | 0x36 | 0x2E | 0x3E => self.rol(&opcode.addressing_mode),

                0x6A => self.ror_acc(),
                0x66 | 0x76 | 0x6E | 0x7E => self.ror(&opcode.addressing_mode),

                /* ------ STORE OPERATIONS ------ */
                // sta
                0x85 | 0x95 | 0x8D | 0x9D | 0x99 | 0x81 | 0x91 => self.sta(&opcode.addressing_mode),

                0x84 | 0x94 | 0x8C => self.sty(&opcode.addressing_mode),

                0x86 | 0x96 | 0x8E => self.stx(&opcode.addressing_mode),

                /* ------ NO OPERATION ------ */
                0xEA => continue,

                /* ------ BREAK AND INTERRUPT OPERATIONS ------ */
                0x40 => self.rti(),

                0x00 => {
                    warn!("instruction brk is not implemented because we don't have interrupts");
                    break;
                }
                _ => todo!(),
            }

            // increment the program counter
            if program_counter_state == self.program_counter {
                self.program_counter += (opcode.bytes - 1) as u16;
            }
        }
    }
}

impl CPU {
    fn push_stack(&mut self, data: u8) {
        self.write_mem_u8(self.stack_pointer - self.stack_base as u16, data);
        self.stack_base = self.stack_base.wrapping_add(1);
    }

    fn pop_stack(&mut self) -> u8 {
        self.stack_base = self.stack_base.wrapping_sub(1);
        let val = self.read_mem_u8(self.stack_pointer - self.stack_base as u16);
        return val;
    }

    fn push_stack_u16(&mut self, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.push_stack(hi);
        self.push_stack(lo);
    }

    fn read_stack_u16(&mut self) -> u16 {
        let lo = self.pop_stack() as u16;
        let hi = self.pop_stack() as u16;

        hi << 8 | lo
    }

    /// Helper function that loads field data into register A
    fn add_to_register_a(&mut self, data: u8) {
        let sum = self.accumulator as u16
            + data as u16
            + (if self
                .processor_status
                .has_flag_set(ProcessorStatusFlags::CarryFlag)
            {
                1
            } else {
                0
            }) as u16;

        let carry = sum > 0xff;

        if carry {
            self.processor_status
                .set_flag_true(ProcessorStatusFlags::CarryFlag);
        } else {
            self.processor_status
                .set_flag_false(ProcessorStatusFlags::CarryFlag);
        }

        let result = sum as u8;

        // checks if the sign bit changed
        if (data ^ result) & (result ^ self.accumulator) & 0x80 != 0 {
            self.processor_status
                .set_flag_true(ProcessorStatusFlags::Overflow);
        } else {
            self.processor_status
                .set_flag_false(ProcessorStatusFlags::Overflow);
        }

        self.set_register_a(result);
    }

    fn set_register_a(&mut self, data: u8) {
        self.accumulator = data;
        self.processor_status
            .update_zero_and_negative_flags(self.accumulator);
    }

    fn set_register_x(&mut self, data: u8) {
        self.index_register_x = data;
        self.processor_status
            .update_zero_and_negative_flags(self.index_register_x);
    }

    fn set_register_y(&mut self, data: u8) {
        self.index_register_y = data;
        self.processor_status
            .update_zero_and_negative_flags(self.index_register_y);
    }

    /// Add with carry
    /// Adds the contents of a memory location to the accumulator with the carry bit
    /// if it overflows then we set the carry bit
    fn adc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.read_mem_u8(addr);

        self.add_to_register_a(value);
    }

    /// Subtract with carry
    /// subtract the contents of memory location to the accumulator together with
    /// the not of the carry bit
    fn sbc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.read_mem_u8(addr);

        self.add_to_register_a(((value as i8).wrapping_neg().wrapping_sub(1)) as u8);
    }

    /// Loads a value into the a register
    fn lda(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.read_mem_u8(addr);

        self.set_register_a(value);
    }

    fn ldx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.read_mem_u8(addr);

        self.set_register_x(value);
    }

    fn ldy(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let value = self.read_mem_u8(addr);

        self.set_register_y(value);
    }

    /// Logical and
    fn and(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let val = self.read_mem_u8(addr);
        self.set_register_a(self.accumulator & val);
    }

    /// Arithmetic shift left. the 7 bit is placed in the carry flag
    fn asl(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let val = self.read_mem_u8(addr);
        let res: u16 = (val as u16) << 1;
        self.processor_status
            .set_flag(ProcessorStatusFlags::CarryFlag, res > 0xFF);

        self.write_mem_u8(addr, res as u8);
    }

    /// Arithmetic shift on the accumulate register
    fn asl_acc(&mut self) {
        let res: u16 = (self.accumulator as u16) << 1;
        self.processor_status
            .set_flag(ProcessorStatusFlags::CarryFlag, res > 0xFF);

        self.set_register_a(res as u8);
    }

    /// Helper function for the branch functions
    fn branch(&mut self) {
        let jump: i8 = self.read_mem_u8(self.program_counter) as i8;
        let jump_addr = self
            .program_counter
            .wrapping_add(1)
            .wrapping_add(jump as u16);

        self.program_counter = jump_addr;
    }

    /// Branch carry if clear - if the carry flag is clear add the displacement
    /// to the program counter to branch the program to a new location
    fn bcc(&mut self) {
        if self
            .processor_status
            .has_flag_set(ProcessorStatusFlags::CarryFlag)
            == false
        {
            self.branch();
        }
    }

    /// Branch carry if set - if the carry flag is set add the displacement
    /// to the program counter to branch the program to a new location
    fn bcs(&mut self) {
        if self
            .processor_status
            .has_flag_set(ProcessorStatusFlags::CarryFlag)
            == true
        {
            self.branch();
        }
    }

    /// Branch if zero flag is set
    fn beq(&mut self) {
        if self
            .processor_status
            .has_flag_set(ProcessorStatusFlags::ZeroFlag)
            == true
        {
            self.branch();
        }
    }

    /// Branch if zero flag is not set
    fn bne(&mut self) {
        if self
            .processor_status
            .has_flag_set(ProcessorStatusFlags::ZeroFlag)
            == false
        {
            self.branch();
        }
    }

    /// Branch if positive
    fn bpl(&mut self) {
        if self
            .processor_status
            .has_flag_set(ProcessorStatusFlags::Negative)
            == false
        {
            self.branch();
        }
    }

    /// Branch if zero flag is not set
    fn bmi(&mut self) {
        if self
            .processor_status
            .has_flag_set(ProcessorStatusFlags::Negative)
            == true
        {
            self.branch();
        }
    }

    /// Preforms the and operation with the accumulator register
    /// and the data at the address and updates the cpu flags accordingly
    fn bit(&mut self, mode: &AddressingMode) {
        let addr: u16 = self.get_operand_address(mode);
        let mem: u8 = self.read_mem_u8(addr);
        let res: u8 = self.accumulator & mem;

        if res == 0 {
            self.processor_status
                .set_flag_true(ProcessorStatusFlags::ZeroFlag);
        }

        self.processor_status
            .set_flag(ProcessorStatusFlags::Negative, (res >> 6 & 1) != 0); // bit 6
        self.processor_status
            .set_flag(ProcessorStatusFlags::Overflow, (res >> 7 & 1) != 0); // bit 6
    }

    /// Branch if overflow clear
    fn bvc(&mut self) {
        if self
            .processor_status
            .has_flag_set(ProcessorStatusFlags::Overflow)
            == false
        {
            let addr: u8 = self.read_mem_u8(self.program_counter); // gets the program counter
            self.program_counter += addr as u16;
        }
    }

    /// Branch if overflow set
    fn bvs(&mut self) {
        if self
            .processor_status
            .has_flag_set(ProcessorStatusFlags::Overflow)
            == true
        {
            let addr: u8 = self.read_mem_u8(self.program_counter); // gets the program counter
            self.program_counter += addr as u16;
        }
    }

    /// This is a helper function that sets the flags for a comparison operation between u8 a and u8 b
    /// field a is the source and field b is the integer you are comparing to
    fn set_compare_flags(&mut self, a: u8, b: u8) {
        if a < b {
            self.processor_status
                .set_flag_false(ProcessorStatusFlags::ZeroFlag);
            self.processor_status
                .set_flag_false(ProcessorStatusFlags::CarryFlag);
            self.processor_status
                .set_flag(ProcessorStatusFlags::Negative, (a >> 7 & 1) != 0);
        } else if a == b {
            self.processor_status
                .set_flag_true(ProcessorStatusFlags::ZeroFlag);
            self.processor_status
                .set_flag_true(ProcessorStatusFlags::CarryFlag);
            self.processor_status
                .set_flag_false(ProcessorStatusFlags::Negative);
        } else {
            self.processor_status
                .set_flag_false(ProcessorStatusFlags::ZeroFlag);
            self.processor_status
                .set_flag_true(ProcessorStatusFlags::CarryFlag);
            self.processor_status
                .set_flag(ProcessorStatusFlags::Negative, (a >> 7 & 1) != 0);
        }
    }

    /// Compares the value stored in memory with the value in the accumulator
    /// if A < mem it sets Z and C flags are zero, and N is the 7th bit of A
    /// if A = mem then Z and C are ones
    /// if A > mem then Z is zero and C is one, and N is the 7th bit of A
    fn cmp(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let val = self.read_mem_u8(addr);

        self.set_compare_flags(self.accumulator, val);
    }

    /// same as cmp but compares the address to the x register
    fn cpx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let val = self.read_mem_u8(addr);

        self.set_compare_flags(self.index_register_x, val);
    }

    /// same as cmp but compares the address to the y register
    fn cpy(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let val = self.read_mem_u8(addr);

        self.set_compare_flags(self.index_register_y, val);
    }

    fn dec(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let val = self.read_mem_u8(addr).wrapping_sub(1);

        self.write_mem_u8(addr, val);
        self.processor_status.update_zero_and_negative_flags(val);
    }

    fn inc(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let val = self.read_mem_u8(addr).wrapping_add(1);

        self.write_mem_u8(addr, val);
        self.processor_status.update_zero_and_negative_flags(val);
    }

    fn eor(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let val = self.read_mem_u8(addr);

        let xor_res = self.accumulator ^ val;
        self.accumulator = xor_res;
        self.processor_status
            .update_zero_and_negative_flags(xor_res);
    }

    /*
    fn jmp(&mut self, mode: &AddressingMode) {
      let addr = self.get_operand_address(mode);
      self.program_counter = self.read_mem_u16(addr);
    }

    /// Pushes the address (minus one) of the return point on to the
    /// stack and then sets the program counter to the target memory address
    fn jsr(&mut self, mode: &AddressingMode) {
      let addr = self.get_operand_address(mode);

      let bytes = (self.program_counter + 2).to_le_bytes();

      self.push_stack(bytes[1]);
      self.push_stack(bytes[0]);

      self.program_counter = addr;
    }

    /// used at the end of a subroutine to return from the subroutine
    /// gets the return value from the stack
    fn rts(&mut self) {
      self.program_counter = u16::from_le_bytes([self.pop_stack(), self.pop_stack()]);
    }
    */

    /// preforms the logical shift right to the defined memory address
    fn lsr(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let val = self.read_mem_u8(addr);
        let res: u16 = (val as u16) >> 1;
        self.write_mem_u8(addr, res as u8);

        self.processor_status
            .set_flag(ProcessorStatusFlags::CarryFlag, res & 1 == 1);
        self.processor_status
            .update_zero_and_negative_flags(res as u8);
    }

    /// preforms the logical shift right to the accumulator register
    fn lsr_acc(&mut self) {
        let mut data = self.accumulator;
        if data & 1 == 1 {
            self.processor_status
                .set_flag_true(ProcessorStatusFlags::CarryFlag);
        } else {
            self.processor_status
                .set_flag_false(ProcessorStatusFlags::CarryFlag);
        }
        data = data >> 1;
        self.set_register_a(data)
    }

    /// Logical Inclusive OR preformed on the accumulator using the contents of the address
    fn ora(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let val = self.read_mem_u8(addr);

        self.accumulator |= val;
        self.processor_status
            .update_zero_and_negative_flags(self.accumulator);
    }

    /// shifts the bits in the accumulator register one place to the left
    /// bit 0 is filled with the value of the carry flag, the old bit 7 becomes the carry flag
    fn rol_acc(&mut self) {
        let old_carry: bool = self
            .processor_status
            .has_flag_set(ProcessorStatusFlags::CarryFlag);
        let mut val = self.accumulator << 1;
        if old_carry {
            val |= 1;
        }
        self.processor_status
            .set_flag(ProcessorStatusFlags::CarryFlag, val >> 7 == 1);
        self.accumulator = val;
    }

    /// shifts the bits at the memory location one place to the left
    /// bit 0 is filled with the value of the carry flag, the old bit 7 becomes the carry flag
    fn rol(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mem_val = self.read_mem_u8(addr);
        let mut val = mem_val << 1;
        if self
            .processor_status
            .has_flag_set(ProcessorStatusFlags::CarryFlag)
            == true
        {
            val = val | 1;
        }
        self.processor_status
            .set_flag(ProcessorStatusFlags::CarryFlag, mem_val >> 7 == 1);
        self.write_mem_u8(addr, val);
    }

    /// shifts the bits in the accumulator register one place to the right
    /// bit 0 is filled with the value of the carry flag, the old bit 7 becomes the carry flag
    fn ror_acc(&mut self) {
        let mut val = self.accumulator >> 1;

        if self
            .processor_status
            .has_flag_set(ProcessorStatusFlags::CarryFlag)
            == true
        {
            val |= 0b1000_0000;
        }
        self.processor_status.set_flag(
            ProcessorStatusFlags::CarryFlag,
            (self.accumulator >> 0 & 1) != 0,
        );

        self.accumulator = val;
    }

    /// shifts the bits at the memory location one place to the right
    /// bit 0 is filled with the value of the carry flag, the old bit 7 becomes the carry flag
    fn ror(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        let mem_val = self.read_mem_u8(addr);
        let mut val = mem_val >> 1;

        if self
            .processor_status
            .has_flag_set(ProcessorStatusFlags::CarryFlag)
            == true
        {
            val |= 0b1000_0000;
        }
        self.processor_status.set_flag(
            ProcessorStatusFlags::CarryFlag,
            (self.accumulator >> 0 & 1) != 0,
        );

        self.write_mem_u8(addr, val);
    }

    /// return from interrupt; this instruction is called at the end of an interrupt
    /// loop and pulls the processor flags from the stack and the program counter from the stack
    fn rti(&mut self) {
        let cpu_flags = self.pop_stack();
        let program_counter = u16::from_le_bytes([self.pop_stack(), self.pop_stack()]);

        self.processor_status.0 = cpu_flags;
        self.program_counter = program_counter;
    }

    /// Loads memory into the a register
    fn sta(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.write_mem_u8(addr, self.accumulator);
    }

    /// Loads memory into the x register
    fn stx(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.write_mem_u8(addr, self.index_register_x);
    }

    /// Loads memory into the y register
    fn sty(&mut self, mode: &AddressingMode) {
        let addr = self.get_operand_address(mode);
        self.write_mem_u8(addr, self.index_register_y);
    }

    /*
    fn inx(&mut self) {
      self.index_register_x = self.index_register_x.wrapping_add(1);
      self.processor_status.update_zero_and_negative_flags(self.index_register_x);
    }

    /// Adds one to the y register
    fn iny(&mut self) {
      self.index_register_y = self.index_register_y.wrapping_add(1);
      self.processor_status.update_zero_and_negative_flags(self.index_register_y);
    }
    */

    /// Forces the generation of an interrupt and pushes the flags and current
    /// instruction to the stack. It sets program counter to u16 value in 0xFFFE
    fn brk(&mut self) {
        /*
        let bytes = self.program_counter.to_le_bytes();

        self.push_stack(bytes[1]);
        self.push_stack(bytes[0]);
        self.push_stack(self.processor_status.0);

        self.processor_status.set_flag_true(ProcessorStatusFlags::BreakCommand);

        let interrupt_handler = self.read_mem_u16(0xFFFE);

        self.program_counter = interrupt_handler;
        */
    }
}

#[cfg(test)]
mod tests {
    use crate::cpu::rom::test::test_rom;

    use super::*;

    #[test]
    fn cpu_new() {
        let bus = Bus::new(test_rom());
        let mut cpu = CPU::new(bus);
        assert_eq!(cpu.index_register_x, 0);
    }

    #[test]
    fn cpu_lda_from_memory() {
        let bus = Bus::new(test_rom());
        let mut cpu = CPU::new(bus);
        cpu.write_mem_u8(0x10, 0xF1); // this should set off the negative flag
        cpu.write_mem_u8(0x11, 0x00); // this should set off the zero flag

        // test the negative flag
        cpu.load_and_run_program(vec![0xa5, 0x10, 0x00]);
        assert_eq!(cpu.accumulator, 0xF1);
        assert_eq!(
            cpu.processor_status
                .has_flag_set(ProcessorStatusFlags::Negative),
            true
        );

        // test the positive flag
        cpu.load_and_run_program(vec![0xa5, 0x11, 0x00]);
        assert_eq!(cpu.accumulator, 0x00);
        assert_eq!(
            cpu.processor_status
                .has_flag_set(ProcessorStatusFlags::ZeroFlag),
            true
        );
    }

    #[test]
    fn cpu_adc_from_memory() {
        let bus = Bus::new(test_rom());
        let mut cpu = CPU::new(bus);
        cpu.write_mem_u8(0x10, 0x05); // this should set off the zero flag

        cpu.load_and_run_program(vec![0x65, 0x10, 0x65, 0x10]);
        assert_eq!(cpu.accumulator, 0x0A);
    }

    #[test]
    fn cpu_bit() {
        let bus = Bus::new(test_rom());
        let mut cpu = CPU::new(bus);
        cpu.write_mem_u8(0x11, 0b1011_1111); // this should set off the overflow flag
        cpu.load_and_run_program(vec![0xA9, 0xFF, 0x24, 0x11]);

        assert_eq!(
            cpu.processor_status
                .has_flag_set(ProcessorStatusFlags::Overflow),
            true
        );
    }

    #[test]
    fn cpu_clear_set_flag_instructions() {
        let bus = Bus::new(test_rom());
        let mut cpu = CPU::new(bus);
        cpu.write_mem_u8(0x11, 0x00);
        cpu.load_and_run_program(vec![0x38, 0xF8, 0x18, 0x00]);

        assert_eq!(
            cpu.processor_status
                .has_flag_set(ProcessorStatusFlags::DecimalMode),
            true
        );
        assert_eq!(
            cpu.processor_status
                .has_flag_set(ProcessorStatusFlags::CarryFlag),
            false
        );
    }

    #[test]
    fn cpu_compare_instructions() {
        let bus = Bus::new(test_rom());
        let mut cpu = CPU::new(bus);
        cpu.write_mem_u8(0x11, 0x00);
        cpu.load_and_run_program(vec![0xA9, 0x81, 0xC9, 0x02, 0x00]);

        assert!(cpu
            .processor_status
            .has_flag_set(ProcessorStatusFlags::CarryFlag));
        assert!(cpu
            .processor_status
            .has_flag_set(ProcessorStatusFlags::Negative));
    }

    #[test]
    fn cpu_increment_decrement() {
        let bus = Bus::new(test_rom());
        let mut cpu = CPU::new(bus);
        cpu.write_mem_u8(0x11, 0x05);
        cpu.load_and_run_program(vec![0xC6, 0x11, 0xC6, 0x11, 0xE6, 0x11, 0xE8]);

        assert_eq!(cpu.read_mem_u8(0x11), 4);
        assert_eq!(cpu.index_register_x, 1);
    }

    #[test]
    fn cpu_eor() {
        let bus = Bus::new(test_rom());
        let mut cpu = CPU::new(bus);
        cpu.write_mem_u8(0x11, 0x05);
        cpu.load_and_run_program(vec![0xA9, 0b1111_1100, 0x49, 0b1111_1110, 0x00]);

        assert_eq!(cpu.accumulator, 2);
    }

    /*
    #[test]
    fn cpu_jmp() {
      let mut cpu = CPU::new();
      cpu.write_mem_u8(0x11, 0x05);
      cpu.load_and_run_program(vec![0x4C, 0x00, 0x60, 0x00, 0x00, 0x00, 0xA9, 0x05, 0x00]);

      assert_eq!(cpu.accumulator, 5);
    }
    */

    #[test]
    fn cpu_ora() {
        let bus = Bus::new(test_rom());
        let mut cpu = CPU::new(bus);
        cpu.load_and_run_program(vec![0xA9, 0b0000_0010, 0x4A]);

        assert_eq!(cpu.accumulator, 1);
    }

    #[test]
    fn cpu_acc_stack() {
        let bus = Bus::new(test_rom());
        let mut cpu = CPU::new(bus);
        cpu.load_and_run_program(vec![0xA9, 0x01, 0x48, 0xA9, 0x02, 0x48, 0x28]);

        assert_eq!(
            cpu.processor_status
                .has_flag_set(ProcessorStatusFlags::ZeroFlag),
            true
        );
    }

    #[test]
    fn cpu_rotate_instructions() {
        let bus = Bus::new(test_rom());
        let mut cpu = CPU::new(bus);
        cpu.write_mem_u8(0x02, 0xFF);
        cpu.load_and_run_program(vec![0x26, 0x02, 0x66, 0x02]);

        assert_eq!(0xFF, cpu.read_mem_u8(0x02));
    }

    #[test]
    fn cpu_transfer_operations() {
        let bus = Bus::new(test_rom());
        let mut cpu = CPU::new(bus);
        cpu.load_and_run_program(vec![0xA9, 0x05, 0x48, 0xBA, 0xAA]);

        assert_eq!(cpu.accumulator, 5);
        assert_eq!(cpu.index_register_x, 5);
    }

    #[test]
    fn test_0xa9_lda_immediate_load_data() {
        let bus = Bus::new(test_rom());
        let mut cpu = CPU::new(bus);
        cpu.load_and_run_program(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.accumulator, 5);
        assert!(cpu.processor_status.0 & 0b0000_0010 == 0b00);
        assert!(cpu.processor_status.0 & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xaa_tax_move_a_to_x() {
        let bus = Bus::new(test_rom());
        let mut cpu = CPU::new(bus);
        cpu.accumulator = 10;
        cpu.load_and_run_program(vec![0xaa, 0x00]);

        assert_eq!(cpu.index_register_x, 10)
    }

    #[test]
    fn test_5_ops_working_together() {
        let bus = Bus::new(test_rom());
        let mut cpu = CPU::new(bus);
        cpu.load_and_run_program(vec![0xa9, 0xc0, 0xaa, 0xe8, 0x00]);

        assert_eq!(cpu.index_register_x, 0xc1)
    }

    #[test]
    fn test_inx_overflow() {
        let bus = Bus::new(test_rom());
        let mut cpu = CPU::new(bus);
        cpu.index_register_x = 0xFF;
        cpu.load_and_run_program(vec![0xe8, 0xe8, 0x00]);

        assert_eq!(cpu.index_register_x, 2)
    }

    #[test]
    fn test_lda_from_memory() {
        let bus = Bus::new(test_rom());
        let mut cpu = CPU::new(bus);
        cpu.write_mem_u8(0x10, 0x55);

        cpu.load_and_run_program(vec![0xa5, 0x10, 0x00]);

        assert_eq!(cpu.accumulator, 0x55);
    }
}
