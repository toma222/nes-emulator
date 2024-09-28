
mod cpu;

use cpu::cpu::CPU;

extern crate env_logger;
pub use log::{debug, error, log_enabled, info, Level};

fn main() {
    std::env::set_var("RUST_LOG", "trace");
    env_logger::init();

    let mut cpu = CPU::new();
    cpu.memory.write_mem_u8(0x11, 0b1011_1111); // this should set off the zero flag
    cpu.load_and_run_program(vec![0xA9, 0xFF, 0x24, 0x11]);

    info!("{}", cpu.log_dump_registers_string());
}
