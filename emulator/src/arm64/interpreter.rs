//! Interpreted execution: fetch-decode-execute loop.

use super::{Armv8Cpu, decode, execute};
use crate::bus::SystemBus;

/// Run the CPU starting at `entry` for exactly `steps` instructions.
/// Returns count of executed instructions.
pub fn run(cpu: &mut Armv8Cpu, bus: &mut SystemBus, entry: u64, max_steps: usize) -> Result<usize, RunError> {
    cpu.regs.pc = entry;

    for _count in 0..max_steps {
        let raw = fetch32(cpu, bus)?;
        let instr = decode(raw).ok_or(RunError::Decode(raw, cpu.regs.pc))?;
        execute(cpu, bus, instr).map_err(|e| RunError::Exec(e, cpu.regs.pc))?;
    }

    Ok(max_steps)
}

fn fetch32(cpu: &Armv8Cpu, bus: &SystemBus) -> Result<u32, RunError> {
    let word = bus.read(cpu.regs.pc, 4).ok_or(RunError::Fetch(cpu.regs.pc))?;
    Ok(word as u32)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunError {
    Fetch(u64),
    Decode(u32, u64),
    Exec(&'static str, u64),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_add_sequence() {
        let mut cpu = Armv8Cpu::new();
        let mut bus = SystemBus::new();

        let code: [u32; 3] = [
            0xD280_0140, // MOVZ X0, #10
            0xD280_0401, // MOVZ X1, #32
            0x9A01_0002, // ADD X2, X0, X1
        ];

        for (i, &word) in code.iter().enumerate() {
            bus.mem.write(0x4000_0000 + (i as u64 * 4), 4, word as u64);
        }

        let steps = run(&mut cpu, &mut bus, 0x4000_0000, 3).unwrap();
        assert_eq!(steps, 3);
        assert_eq!(cpu.regs.x(2), 42);
    }

    #[test]
    fn hello_uart() {
        let mut cpu = Armv8Cpu::new();
        let mut bus = SystemBus::new();

        // Assemble at 0x4000_0000:
        // 1. MOVZ X0, #0x4869    ; low 16 bits = 'H', 'i' packed
        // 2. MOVZ X1, #0x0900, LSL #16  ; X1 = 0x09000000 (UART base)
        // 3. STR X0, [X1]       ; write 8 bytes to UART (only low byte matters)
        // 4. NOP                ; done

        let code: [u32; 4] = [
            0xD282_4680, // MOVZ X0, #0x1234
            0xD2A1_2001, // MOVZ X1, #0x0900, LSL #16
            0xF900_0020, // STR X0, [X1]
            0xD503_201F, // NOP
        ];

        for (i, &word) in code.iter().enumerate() {
            bus.mem.write(0x4000_0000 + (i as u64 * 4), 4, word as u64);
        }

        let steps = run(&mut cpu, &mut bus, 0x4000_0000, 4).unwrap();
        assert_eq!(steps, 4);
        // UART receives low byte of X0 = 0x34 = '4'
        assert_eq!(&bus.uart.output, &[0x34]);
        assert_eq!(bus.uart.output_string(), "4");
    }

    #[test]
    fn boot_stub_to_kernel() {
        let mut cpu = Armv8Cpu::new();
        let mut bus = SystemBus::new();

        // Boot stub at 0x4000_0000 (in RAM):
        // BR X0             ; jump to address in X0
        let boot_pc = 0x4000_0000u64;
        let kernel_pc = 0x4000_0100u64;
        let boot_stub = [0xD61F_0000u32]; // BR X0

        // Kernel at 0x4000_0100:
        // MOVZ X0, #0x1234
        // MOVZ X1, #0x0900, LSL #16
        // STR X0, [X1]
        // NOP
        let kernel: [u32; 4] = [
            0xD282_4680, // MOVZ X0, #0x1234
            0xD2A1_2001, // MOVZ X1, #0x0900, LSL #16
            0xF900_0020, // STR X0, [X1]
            0xD503_201F, // NOP
        ];

        bus.mem.write(boot_pc, 4, boot_stub[0] as u64);
        for (i, &word) in kernel.iter().enumerate() {
            bus.mem.write(kernel_pc + (i as u64 * 4), 4, word as u64);
        }

        // Pre-set X0 to kernel entry point (this is what firmware does)
        cpu.regs.set_x(0, kernel_pc);

        // Run boot stub: single instruction BR X0
        let steps = run(&mut cpu, &mut bus, boot_pc, 1).unwrap();
        assert_eq!(steps, 1);
        assert_eq!(cpu.regs.pc, kernel_pc);

        // Run kernel: 3 instructions
        let _ = run(&mut cpu, &mut bus, kernel_pc, 3).unwrap();
        assert_eq!(bus.uart.output_string(), "4");
    }

    #[test]
    #[ignore = "slow: loads 37 MB kernel"]
    fn real_kernel_runs_past_prologue() {
        use crate::loader::{load_kernel, KERNEL_LOAD};
        use crate::efi::setup_efi_tables;

        let mut cpu = Armv8Cpu::new();
        let mut bus = SystemBus::new();

        let _entry = load_kernel(&mut bus, "/Users/petreleon/code/WebBoxVM/Image.gz").unwrap();

        // Setup EFI runtime environment for PE/EFI kernel boot
        let (handle, st) = setup_efi_tables(&mut bus, KERNEL_LOAD, 0x024f_0000);
        cpu.regs.set_x(0, handle);  // ImageHandle
        cpu.regs.set_x(1, st);       // SystemTable
        cpu.regs.sp = 0x43FF_F000;   // near top of 1 GiB RAM

        // Run the PE entry point
        cpu.regs.pc = KERNEL_LOAD + 0x01da7ee0;

        let mut steps = 0;
        for _ in 0..400 {
            let raw = match bus.read(cpu.regs.pc, 4) {
                Some(v) => v as u32,
                None => {
                    println!("Memory fault at step {} PC=0x{:016x}", steps, cpu.regs.pc);
                    break;
                }
            };
            if let Some(instr) = decode(raw) {
                if steps >= 70 {
                println!("Step {:3} PC=0x{:016x}: raw=0x{:08x} {:10?}  X0={:016x} X1={:016x} X2={:016x} X30={:016x}",
                    steps, cpu.regs.pc, raw, instr.op,
                    cpu.regs.x(0), cpu.regs.x(1), cpu.regs.x(2),
                    cpu.regs.x(30));
            }
                if let Err(e) = execute(&mut cpu, &mut bus, instr) {
                    println!("EXECUTE ERROR at step {} PC=0x{:016x}: {:?}", steps, cpu.regs.pc, e);
                    break;
                }
                steps += 1;
                if steps >= 200 { break; }
            } else {
                println!("UNKNOWN INSTRUCTION at step {} PC=0x{:016x} raw=0x{:08x}", steps, cpu.regs.pc, raw);
                break;
            }
        }

        // We should execute at least 30 real kernel/EFI stub instructions
        println!("EFI stub executed {} instructions, X0=0x{:016x}", steps, cpu.regs.x(0));
        assert!(steps >= 30, "Only executed {} instructions, expected at least 30", steps);
    }

    #[test]
    fn synthetic_kernel_boots_to_uart() {
        use crate::loader::{load_raw_image, KERNEL_LOAD};
        use std::fs;
        
        let mut cpu = Armv8Cpu::new();
        let mut bus = SystemBus::new();
        
        // Load the properly assembled ARM64 kernel binary
        // Built from /tmp/kernel.S using: aarch64-elf-as, aarch64-elf-ld, aarch64-elf-objcopy
        let data = fs::read("/tmp/kernel_raw.bin").expect("kernel not found; run build_kernel.sh first");
        load_raw_image(&mut bus, &data);
        
        // Set a valid stack pointer
        cpu.regs.sp = 0x43FF_F000;
        
        // Run the synthetic kernel
        let result = run(&mut cpu, &mut bus, KERNEL_LOAD, 50);
        println!("Result: {:?}", result);
        println!("UART output bytes: {:?}", bus.uart.output);
        // Should exhaust steps (hit infinite loop) or succeed
        assert!(result.is_ok(), "Synthetic kernel crashed: {:?}", result);
        
        // Verify UART output contains the expected message
        assert!(bus.uart.output_string().contains("Uncompressing Linux..."), "UART output missing expected message");
    }
}
