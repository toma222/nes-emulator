use crate::cpu::memory::Mem;

/// The bus is a wiring between devices
/// and we got 2kb of it ;)
/// also the bus is mirrored 3 times
pub struct Bus {
    cpu_vram: [u8; 2048],
}

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFF;
const PPU_REGISTERS: u16 = 0x2000;
const PPU_REGISTERS_MIRRORS_END: u16 = 0x3FFF;

impl Bus {
    pub fn new() -> Self {
        Bus {
            cpu_vram: [0; 2048],
        }
    }
}

impl Mem for Bus {
    fn read_mem_u8(&self, addr: u16) -> u8 {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b00000111_11111111;
                self.cpu_vram[mirror_down_addr as usize]
            }

            PPU_REGISTERS..=PPU_REGISTERS_MIRRORS_END => {
                let _mirror_down_addr = addr & 0b00100000_00000111;
                todo!("PPU is not supported yet")
            }

            _ => {
                println!("Ignoring mem access at {}", addr);
                0
            }
        }
    }

    fn write_mem_u8(&mut self, addr: u16, data: u8) {
        match addr {
            RAM..=RAM_MIRRORS_END => {
                let mirror_down_addr = addr & 0b11111111111;
                self.cpu_vram[mirror_down_addr as usize] = data;
            }
            PPU_REGISTERS..=PPU_REGISTERS_MIRRORS_END => {
                let _mirror_down_addr = addr & 0b00100000_00000111;
                todo!("PPU is not supported yet");
            }
            _ => {
                println!("Ignoring mem write-access at {}", addr);
            }
        }
    }
}
