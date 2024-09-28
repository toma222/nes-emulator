
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

