pub struct Bus {
    cpu_vram: [u8; 2048],
}

const RAM: u16 = 0x0000;
const RAM_MIRRORS_END: u16 = 0x1FFFF;
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
    fn mem_read(&self, addr: u16) -> u8 {
        Ram..=RAM_MIRRORS_END {}
    }
}
