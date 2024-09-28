
mod cpu;

use cpu::cpu::CPU;

extern crate env_logger;
pub use log::{debug, error, log_enabled, info, Level};

fn main() {
    std::env::set_var("RUST_LOG", "trace");
    env_logger::init();

    let mut cpu = CPU::new();
    cpu.memory.write_mem_u8(0x11, 0x05);
    cpu.load_and_run_program(vec![0x4C, 0x06, 0x80, 0x00, 0x00, 0x00, 0xA9, 0x05, 0x00]);

    info!("{}", cpu.log_dump_registers_string());

    println!("{}", cpu.memory.read_mem_u8(0x11));
}
