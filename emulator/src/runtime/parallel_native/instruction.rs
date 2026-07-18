use super::*;
use std::sync::atomic::{Ordering, fence};

pub(super) fn fetch(
    cpu: &mut Armv8Cpu,
    cache: &mut DecodeCache,
    shared: &SharedRun<'_>,
) -> Option<(u64, u64, Instr)> {
    let pc = cpu.regs.pc;
    let bus = shared
        .bus
        .read()
        .unwrap_or_else(|poison| poison.into_inner());
    let pa = if cpu.sys.sctlr_el1 & SCTLR_MMU_ENABLE == 0 {
        pc
    } else {
        match translate(&cpu.sys, &mut cpu.tlb, &bus.mem, pc) {
            Ok(pa) => pa,
            Err(_) => {
                shared.fetch_faults.fetch_add(1, Ordering::Relaxed);
                if cpu.sys.vbar_el1 != 0 {
                    cpu.sys.far_el1 = pc;
                    take_instruction_abort(cpu, pc);
                } else {
                    cpu.regs.pc += INSTRUCTION_SIZE;
                }
                return None;
            }
        }
    };
    let instr = cache.fetch(&bus.mem, pa);
    drop(bus);
    let Some(instr) = instr else {
        cpu.regs.pc += INSTRUCTION_SIZE;
        return None;
    };
    Some((pc, pa, instr))
}

pub(super) fn execute_shared(
    core: usize,
    cpu: &mut Armv8Cpu,
    pc: u64,
    instr: Instr,
    shared: &SharedRun<'_>,
) {
    let mut bus = shared
        .bus
        .write()
        .unwrap_or_else(|poison| poison.into_inner());
    if matches!(instr.op, Opcode::Stxr | Opcode::Stxp)
        && cpu.exclusive_epoch != shared.memory_epoch.load(Ordering::Acquire)
    {
        cpu.clear_exclusive();
    }
    if matches!(instr.op, Opcode::Dmb | Opcode::Dsb | Opcode::Isb) {
        fence(Ordering::SeqCst);
    }
    if handle_gic_sysreg_access(cpu, &mut bus, instr) {
        deliver_external_irq(cpu, &mut bus, core);
        return;
    }

    bus.begin_cpu_instruction();
    let result = execute(cpu, &mut bus, instr);
    if result.is_ok() && matches!(instr.op, Opcode::Ldxr | Opcode::Ldxp) && cpu.exclusive.is_some()
    {
        cpu.exclusive_epoch = shared.memory_epoch.load(Ordering::Acquire);
    }
    let wrote = bus.dma_write_during_instruction() || !bus.memory_writes().is_empty();
    if wrote {
        shared.memory_epoch.fetch_add(1, Ordering::AcqRel);
    }
    bus.finish_cpu_instruction();
    if let Err(error) = result {
        handle_execute_fault(cpu, pc, instr, error, shared);
    }
    if bus.external_irq_poll_needed_for_cpu(core) {
        bus.refresh_interrupts();
        deliver_external_irq(cpu, &mut bus, core);
    }
}

pub(super) fn deliver_irq(core: usize, cpu: &mut Armv8Cpu, shared: &SharedRun<'_>) {
    let mut bus = shared
        .bus
        .write()
        .unwrap_or_else(|poison| poison.into_inner());
    if !bus.external_irq_poll_needed_for_cpu(core) {
        return;
    }
    bus.refresh_interrupts();
    deliver_external_irq(cpu, &mut bus, core);
}

pub(super) fn handle_execute_fault(
    cpu: &mut Armv8Cpu,
    pc: u64,
    instr: Instr,
    error: &str,
    shared: &SharedRun<'_>,
) {
    shared.exec_faults.fetch_add(1, Ordering::Relaxed);
    if is_data_abort_fault(error) {
        take_data_abort(cpu, pc, instr, error, false);
    } else {
        cpu.regs.pc += INSTRUCTION_SIZE;
    }
}
