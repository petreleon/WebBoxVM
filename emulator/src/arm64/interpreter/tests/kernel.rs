use super::*;
use crate::arm64::{Armv8Cpu, decode, execute};
use crate::arm64::mmu::translate;
use crate::bus::SystemBus;

#[test]
#[ignore = "slow: loads 37 MB kernel"]
fn real_kernel_runs_past_prologue() {
    use crate::loader::kernel::{load_kernel, KERNEL_LOAD};
    use crate::efi::setup_efi_tables;

    let mut cpu = Armv8Cpu::new();
    let mut bus = SystemBus::new();

    let _entry = load_kernel(&mut bus, "/Users/petreleon/code/WebBoxVM/Image.gz").unwrap();

    let (handle, st) = setup_efi_tables(&mut bus, KERNEL_LOAD, 0x024f_0000);
    cpu.regs.set_x(0, handle);
    cpu.regs.set_x(1, st);
    cpu.regs.sp = 0x43FF_F000;

    bus.write(0x43FFE000, 4, 0xD65F03C0);
    cpu.regs.set_x(30, 0x43FFE000);

    cpu.regs.pc = KERNEL_LOAD + 0x01da7ee0;

    let mut steps = 0;
    let mut last_pc = cpu.regs.pc;
    for _ in 0..1000 {
        let pa = match translate(&cpu.sys, &mut cpu.tlb, &bus.mem, cpu.regs.pc) {
            Ok(addr) => addr,
            Err(_) => {
                println!("Translation fault at step {} PC=0x{:016x}", steps, cpu.regs.pc);
                break;
            }
        };
        let raw = match bus.mem.read(pa, 4) {
            Some(v) => v as u32,
            None => {
                println!("Memory fault at step {} PC=0x{:016x} PA=0x{:016x}", steps, cpu.regs.pc, pa);
                break;
            }
        };
        if let Some(instr) = decode(raw) {
            if let Err(e) = execute(&mut cpu, &mut bus, instr) {
                println!("EXECUTE ERROR at step {} PC=0x{:016x}: {:?}", steps, cpu.regs.pc, e);
                break;
            }
            steps += 1;
            if cpu.regs.pc == last_pc {
                println!("Stalled at PC=0x{:016x} after {} steps", cpu.regs.pc, steps);
                break;
            }
            last_pc = cpu.regs.pc;
        } else {
            println!("UNKNOWN INSTRUCTION at step {} PC=0x{:016x} raw=0x{:08x}", steps, cpu.regs.pc, raw);
            break;
        }
    }

    println!("EFI stub executed {} instructions, X0=0x{:016x}", steps, cpu.regs.x(0));
    println!("  Final: PC=0x{:016x} SP=0x{:016x}", cpu.regs.pc, cpu.regs.sp);
    assert!(steps >= 200, "Only executed {} instructions, expected at least 200", steps);
}

#[test]
#[ignore = "slow: loads 37 MB kernel"]
fn real_kernel_runs_past_prologue_trace() {
    // Debug-only test: prints trace
    use crate::loader::kernel::{load_kernel, KERNEL_LOAD};
    use crate::efi::setup_efi_tables;

    let mut cpu = Armv8Cpu::new();
    let mut bus = SystemBus::new();

    let _entry = load_kernel(&mut bus, "/Users/petreleon/code/WebBoxVM/Image.gz").unwrap();

    let (handle, st) = setup_efi_tables(&mut bus, KERNEL_LOAD, 0x024f_0000);
    cpu.regs.set_x(0, handle);
    cpu.regs.set_x(1, st);
    cpu.regs.sp = 0x43FF_F000;
    // Seed X18 — EFI stub may dereference it via [X18, #offset].
    cpu.regs.set_x(18, st);

    bus.write(0x43FFE000, 4, 0xD65F03C0);
    cpu.regs.set_x(30, 0x43FFE000);

    cpu.regs.pc = KERNEL_LOAD + 0x01da7ee0;

    let mut steps = 0;
    let mut last_pc = cpu.regs.pc;
    for _ in 0..20000 {
        let pa = match translate(&cpu.sys, &mut cpu.tlb, &bus.mem, cpu.regs.pc) {
            Ok(addr) => addr,
            Err(_) => {
                println!("Translation fault at step {} PC=0x{:016x}", steps, cpu.regs.pc);
                break;
            }
        };
        let raw = match bus.mem.read(pa, 4) {
            Some(v) => v as u32,
            None => {
                println!("Memory fault at step {} PC=0x{:016x} PA=0x{:016x}", steps, cpu.regs.pc, pa);
                break;
            }
        };
        if let Some(instr) = decode(raw) {
            if let Err(e) = execute(&mut cpu, &mut bus, instr) {
                println!("EXECUTE ERROR at step {} PC=0x{:016x}: {:?}", steps, cpu.regs.pc, e);
                break;
            }
            steps += 1;
            if cpu.regs.pc == last_pc {
                println!("Stalled at PC=0x{:016x} after {} steps", cpu.regs.pc, steps);
                break;
            }
            last_pc = cpu.regs.pc;
        } else {
            println!("UNKNOWN INSTRUCTION at step {} PC=0x{:016x} raw=0x{:08x}", steps, cpu.regs.pc, raw);
            break;
        }
    }

    println!("EFI stub executed {} instructions", steps);
    println!("  X0=0x{:016x} X1=0x{:016x} X2=0x{:016x} X18=0x{:016x}", cpu.regs.x(0), cpu.regs.x(1), cpu.regs.x(2), cpu.regs.x(18));
    println!("  PC=0x{:016x} SP=0x{:016x} X30=0x{:016x}", cpu.regs.pc, cpu.regs.sp, cpu.regs.x(30));
    println!("  UART output: {:?}", bus.uart.output_string());
    // No assert — this is just a debug trace
}

#[test]
fn synthetic_kernel_boots_to_uart() {
    use crate::loader::kernel::{load_raw_image, KERNEL_LOAD};
    use std::fs;

    let mut cpu = Armv8Cpu::new();
    let mut bus = SystemBus::new();

    let data = fs::read("/tmp/kernel_raw.bin")
        .expect("kernel not found; run build_kernel.sh first");
    load_raw_image(&mut bus, &data);

    cpu.regs.sp = 0x43FF_F000;

    let result = run(&mut cpu, &mut bus, KERNEL_LOAD, 50);
    println!("Result: {:?}", result);
    println!("UART output bytes: {:?}", bus.uart.output);
    assert!(result.is_ok(), "Synthetic kernel crashed: {:?}", result);
    assert!(bus.uart.output_string().contains("Uncompressing Linux..."), "UART output missing expected message");
}
