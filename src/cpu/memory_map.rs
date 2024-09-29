
use byteorder::{ByteOrder, LittleEndian, ReadBytesExt}; // 1.3.4

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
    pub memory: [u8; 0xFFFF]
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
        let lo = self.read_mem_u8(loc) as u16;
        let hi = self.read_mem_u8(loc + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    pub fn write_mem_u16(&mut self, loc: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.write_mem_u8(loc, lo);
        self.write_mem_u8(loc + 1, hi);
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

