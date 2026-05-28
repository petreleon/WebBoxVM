use emulator::arm64::{Armv8Cpu, run};
use emulator::bus::SystemBus;

fn main() {
    let mut cpu = Armv8Cpu::new();
    let mut bus = SystemBus::new();

    // MOVZ X0, #0; ADD X0, X0, #1; ADD X0, X0, #1; NOP
    bus.write(0x4000_0000, 4, 0xD2800000);
    bus.write(0x4000_0004, 4, 0x91000400);
    bus.write(0x4000_0008, 4, 0x91000400);
    bus.write(0x4000_000C, 4, 0xD503201F);

    cpu.regs.pc = 0x4000_0000;

    match run(&mut cpu, &mut bus, 0x4000_0000, 4) {
        Ok(steps) => {
            // Use panic to display results (always visible in wasmtime)
            panic!("WASM64 OK: {} steps, X0=0x{:016x}", steps, cpu.regs.x(0));
        }
        Err(e) => {
            panic!("WASM64 ERROR: {:?}", e);
        }
    }
}
