
mod cpu;

use cpu::cpu::CPU;

extern crate env_logger;
pub use log::{debug, error, log_enabled, info, Level};

fn main() {
    std::env::set_var("RUST_LOG", "trace");
    env_logger::init();

    let mut cpu = CPU::new();
    cpu.load_and_run_program(vec![0xA9, 0x05, 0x48, 0xBA, 0xAA]);

    info!("{}", cpu.log_dump_registers_string());
}
