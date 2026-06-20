use super::*;

#[test]
#[ignore = "slow: loads 37 MB kernel"]
fn real_kernel_runs_past_prologue() {
    let init_script = b"#!/bin/sh\necho '=== WEBBOXVM ==='\nmount -t proc proc /proc\nmount -t sysfs sysfs /sys\nexec /bin/sh\n".to_vec();
    let KernelFixture {
        mut cpu, mut bus, ..
    } = load_real_kernel_fixture(init_script, false, false);
    let mut steps = 0u64;
    let mut last_pc = cpu.regs.pc;
    let mut efi_stub_done = false;
    let mut pages_bump = 0x4800_0000u64;
    let mut history = std::collections::VecDeque::with_capacity(105);
    let mut decode_cache = crate::arch::arm64::DecodeCache::new();

    let total_steps = 500_000_000usize;
    for _ in 0..total_steps {
        if steps % 5_000_000 == 0 {
            eprintln!(
                "PROGRESS: {:.1}M steps, cache {}/{}",
                steps as f64 / 1_000_000.0,
                decode_cache.hits,
                decode_cache.misses
            );
        }
        if !efi_stub_done && cpu.regs.pc >= 0xffff800000000000 {
            println!(
                "EFI Stub done in {} steps at PC={:#018x}. Switching to JIT kernel...",
                steps, cpu.regs.pc
            );
            efi_stub_done = true;
            break;
        }

        match handle_kernel_shortcut(&mut cpu, &mut bus, &mut pages_bump, &mut steps) {
            KernelShortcut::None => {}
            KernelShortcut::Handled | KernelShortcut::Relocation(_) => continue,
        }

        let pa = match translate(&cpu.sys, &mut cpu.tlb, &bus.mem, cpu.regs.pc) {
            Ok(addr) => addr,
            Err(_) => {
                println!(
                    "Translation fault at step {} PC=0x{:016x}",
                    steps, cpu.regs.pc
                );
                break;
            }
        };
        let raw = match bus.mem.read(pa, 4) {
            Some(v) => v as u32,
            None => {
                println!(
                    "Memory fault at step {} PC=0x{:016x} PA=0x{:016x}",
                    steps, cpu.regs.pc, pa
                );
                break;
            }
        };

        let Some(instr) = decode_cache.fetch(&bus.mem, pa) else {
            println!(
                "UNKNOWN INSTRUCTION at step {} PC=0x{:016x} raw=0x{:08x}",
                steps, cpu.regs.pc, raw
            );
            break;
        };
        history.push_back((cpu.regs.pc, raw, instr));
        if history.len() > 100 {
            history.pop_front();
        }
        if instr.op == Opcode::Brk {
            print_recent_history(&history);
        }
        if let Err(e) = execute(&mut cpu, &mut bus, instr) {
            println!(
                "EXECUTE ERROR at step {} PC=0x{:016x}: {:?}",
                steps, cpu.regs.pc, e
            );
            break;
        }
        steps += 1;
        if cpu.regs.pc == last_pc {
            println!("Stalled at PC=0x{:016x} after {} steps", cpu.regs.pc, steps);
            break;
        }
        last_pc = cpu.regs.pc;
    }

    if efi_stub_done {
        println!("--- JIT KERNEL PHASE ---");
        let mut jit_engine = crate::arch::arm64::jit::JitEngine::new();
        let current_pc = cpu.regs.pc;
        let remaining = total_steps.saturating_sub(steps as usize);
        match jit_engine.run(&mut cpu, &mut bus, current_pc, remaining) {
            Ok(jit_steps) => steps += jit_steps as u64,
            Err(e) => println!("JIT ERROR: {}", e),
        }
    }

    print_boot_summary(&cpu, &bus, steps);
    scan_fixmap_for_uart(&mut cpu, &bus);
    scan_printk_log(&bus);
    assert!(
        steps > 1000,
        "Kernel should execute at least 1000 instructions"
    );
}

fn print_recent_history(
    history: &std::collections::VecDeque<(u64, u32, crate::arch::arm64::opcodes::Instr)>,
) {
    println!("--- LAST 100 EXECUTED INSTRUCTIONS ---");
    for (hist_pc, hist_raw, hist_instr) in history {
        println!("  0x{:016x}: {:08x} {:?}", hist_pc, hist_raw, hist_instr);
    }
    println!("--------------------------------------");
}
