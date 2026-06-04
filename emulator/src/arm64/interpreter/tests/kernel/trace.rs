use super::*;

#[test]
#[ignore = "slow: loads 37 MB kernel"]
fn real_kernel_runs_past_prologue_trace() {
    let init_script =
        b"#!/bin/sh\necho '=== WEBBOXVM ==='\nmount -t proc proc /proc\nexec /bin/sh\n".to_vec();
    let KernelFixture {
        mut cpu,
        mut bus,
        dtb_addr,
    } = load_real_kernel_fixture(init_script, true, true);
    let mut trace = EfiTrace::new(&bus);
    let mut steps = 0u64;
    let mut last_pc = cpu.regs.pc;
    let mut efi_stub_done = false;
    let mut pages_bump = 0x4800_0000u64;
    let trace_steps = std::env::var("WEBBOXVM_KERNEL_TRACE_STEPS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(60_000);

    print_trace_preamble(&bus, dtb_addr, trace_steps);

    'main: for _ in 0..trace_steps {
        if !efi_stub_done && cpu.regs.pc == 0 {
            println!("=== EFI Stub done ({} steps) => main kernel ===", steps);
            efi_stub_done = true;
            cpu.regs.pc = 0xffff800080080000;
            cpu.regs.set_x(0, dtb_addr);
            cpu.regs.set_x(1, 0);
            cpu.regs.set_x(2, 0);
            cpu.regs.set_x(3, 0);
            last_pc = cpu.regs.pc;
            continue;
        }

        match handle_kernel_shortcut(&mut cpu, &mut bus, &mut pages_bump, &mut steps) {
            KernelShortcut::None => {}
            KernelShortcut::Handled => continue,
            KernelShortcut::Relocation(records) => {
                println!(
                    "Fast-forwarded EFI relocation loop: {} records, X9={:#x}, X10={:#x}, X23={:#x}",
                    records,
                    cpu.regs.x(9),
                    cpu.regs.x(10),
                    cpu.regs.x(23)
                );
                continue;
            }
        }

        let pa = match translate(&cpu.sys, &mut cpu.tlb, &bus.mem, cpu.regs.pc) {
            Ok(a) => a,
            Err(_) => {
                println!("TRANSLATION FAULT step={} PC={:#016x}", steps, cpu.regs.pc);
                break;
            }
        };
        let raw = match bus.mem.read(pa, 4) {
            Some(v) => v as u32,
            None => {
                println!(
                    "MEMORY FAULT step={} PC={:#016x} PA={:#016x}",
                    steps, cpu.regs.pc, pa
                );
                break;
            }
        };

        if let Some(instr) = decode(raw) {
            trace.observe_call_and_return(steps, &cpu, &bus, &instr);
            trace.record_instruction(steps, &cpu, &instr);
            if let Err(e) = execute(&mut cpu, &mut bus, instr) {
                println!(
                    "EXECUTE ERROR step={} PC={:#016x}: {:?}",
                    steps, cpu.regs.pc, e
                );
                break 'main;
            }
            steps += 1;
            if !efi_stub_done && cpu.regs.pc == last_pc {
                println!("Stalled PC={:#016x} after {} steps", cpu.regs.pc, steps);
                break;
            }
            last_pc = cpu.regs.pc;
        } else {
            println!(
                "UNKNOWN INSTR step={} PC={:#016x} raw={:#010x}",
                steps, cpu.regs.pc, raw
            );
            trace.print_recent(30);
            break;
        }
    }

    trace.print_summary(&cpu, &bus, steps);
}
