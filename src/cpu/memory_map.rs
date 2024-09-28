
use byteorder::{ByteOrder, LittleEndian, ReadBytesExt}; // 1.3.4

use crate::cpu::cpu::CPU;

/// This defines the memory
/// and has some implementations for managing that memory
/// This holds the memory. All of it <3.
/// The first 256 byte page of memory (0x0000 - 0x00FF) is Zero Page
/// The second page (0x0100-0x01FF) is the system stack
/// The other reserved parts of the memory map is 0xFFFA to 0xFFFF
/// that part has to be programed with the interrupt handler (0xFFFA/B)
/// the power reset location and the BRK/interrupt request handler
pub struct MemoryMap
{
    memory: [u8; 0xFFFF]
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

    /// its a 16 bit address that identifies the location of the least signigicant byte
    /// of another 16 bit memory address which is the real target of the instruction
    Indirect,

    /// same as indirect but we add the x register
    IndexedIndirect,

    /// same as indirect but we add the y register
    IndirectIndexed,
}

impl MemoryMap
{
    pub fn new() -> MemoryMap
    {
        MemoryMap{
            memory: [0; 0xFFFF]
        }
    }

    pub fn read_mem_u8(&self, loc: u16) -> u8 {
        return self.memory[loc as usize];
    }

    pub fn write_mem_u8(&mut self, loc: u16, data: u8) {
        self.memory[loc as usize] = data;
    }

    pub fn read_mem_u16(&self, loc: u16) -> u16 {
        let mem_parts = [
            self.memory[loc as usize],
            self.memory[(loc + 1) as usize]
        ];

        let mut mem_parts_ref = &mem_parts[..];
        match mem_parts_ref.read_u16::<LittleEndian>() {
            Ok(a) => return a,
            Err(_) => {
                panic!("read_16::<LittleEndian> returned error");
            }
        }
    }

    pub fn write_mem_u16(&mut self, loc: u16, data: u16) {
        let mut buffer = [0 as u8; 2];
        LittleEndian::write_u16(&mut buffer, data);
        
        self.memory[loc as usize] = buffer[0];
        self.memory[(loc + 1) as usize] = buffer[1];
    }

    // if its supposed to return a u8 then you can cast it to a u8 <3
    pub fn read_mem_mode(&mut self, cpu_ref: &CPU, mode: AddressingMode, loc: u16) -> u16
    {
        match mode {
            AddressingMode::ZeroPage => { self.read_mem_u8(loc) as u16 }
            AddressingMode::ZeroPageX => { self.read_mem_u8(loc + cpu_ref.index_register_x as u16) as u16 }
            AddressingMode::ZeroPageY => { self.read_mem_u8(loc + cpu_ref.index_register_y as u16) as u16 }
            _ => return 0
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_write_u8() {
        let mut mem_map = MemoryMap::new();

        mem_map.write_mem_u8(0x0000, 2);
        assert_eq!(mem_map.read_mem_u8(0x0000), 2);
    }

    #[test]
    fn read_write_u16() {
        let mut mem_map = MemoryMap::new();

        mem_map.write_mem_u16(0xF000, 65533);
        assert_eq!(mem_map.read_mem_u16(0xF000), 65533);
    }
}

