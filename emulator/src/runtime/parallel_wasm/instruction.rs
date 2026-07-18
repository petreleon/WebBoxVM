use super::*;
use std::sync::atomic::{Ordering, fence};

pub(super) fn fetch(
    cpu: &mut Armv8Cpu,
    cache: &mut DecodeCache,
    control: &WasmParallelControl,
) -> Option<(u64, Instr)> {
    let pc = cpu.regs.pc;
    let _guard = control
        .gate
        .read()
        .unwrap_or_else(|poison| poison.into_inner());
    let bus = unsafe { &*bus_ptr(control) };
    let pa = if cpu.sys.sctlr_el1 & SCTLR_MMU_ENABLE == 0 {
        pc
    } else {
        match translate(&cpu.sys, &mut cpu.tlb, &bus.mem, pc) {
            Ok(pa) => pa,
            Err(_) => {
                control.fetch_faults.fetch_add(1, Ordering::Relaxed);
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
    drop(_guard);
    let Some(instr) = instr else {
        cpu.regs.pc += INSTRUCTION_SIZE;
        return None;
    };
    Some((pc, instr))
}

pub(super) fn execute_shared(
    core: usize,
    cpu: &mut Armv8Cpu,
    pc: u64,
    instr: Instr,
    control: &WasmParallelControl,
) {
    let _guard = control
        .gate
        .write()
        .unwrap_or_else(|poison| poison.into_inner());
    let bus = unsafe { &mut *bus_ptr(control) };
    if matches!(instr.op, Opcode::Stxr | Opcode::Stxp)
        && cpu.exclusive_epoch != control.memory_epoch.load(Ordering::Acquire)
    {
        cpu.clear_exclusive();
    }
    if matches!(instr.op, Opcode::Dmb | Opcode::Dsb | Opcode::Isb) {
        fence(Ordering::SeqCst);
    }
    if handle_gic_sysreg_access(cpu, bus, instr) {
        deliver_external_irq(cpu, bus, core);
        return;
    }
    bus.begin_cpu_instruction();
    let result = execute(cpu, bus, instr);
    if result.is_ok() && matches!(instr.op, Opcode::Ldxr | Opcode::Ldxp) && cpu.exclusive.is_some()
    {
        cpu.exclusive_epoch = control.memory_epoch.load(Ordering::Acquire);
    }
    if bus.dma_write_during_instruction() || !bus.memory_writes().is_empty() {
        control.memory_epoch.fetch_add(1, Ordering::AcqRel);
    }
    bus.finish_cpu_instruction();
    if let Err(error) = result {
        handle_execute_fault(cpu, pc, instr, error, control);
    }
    if bus.external_irq_poll_needed_for_cpu(core) {
        bus.refresh_interrupts();
        deliver_external_irq(cpu, bus, core);
    }
}

pub(super) fn deliver_irq(core: usize, cpu: &mut Armv8Cpu, control: &WasmParallelControl) {
    let _guard = control
        .gate
        .write()
        .unwrap_or_else(|poison| poison.into_inner());
    let bus = unsafe { &mut *bus_ptr(control) };
    if bus.external_irq_poll_needed_for_cpu(core) {
        bus.refresh_interrupts();
        deliver_external_irq(cpu, bus, core);
    }
}

pub(super) fn handle_execute_fault(
    cpu: &mut Armv8Cpu,
    pc: u64,
    instr: Instr,
    error: &str,
    control: &WasmParallelControl,
) {
    control.exec_faults.fetch_add(1, Ordering::Relaxed);
    if is_data_abort_fault(error) {
        take_data_abort(cpu, pc, instr, error, false);
    } else {
        cpu.regs.pc += INSTRUCTION_SIZE;
    }
}

unsafe fn bus_ptr(control: &WasmParallelControl) -> *mut SystemBus {
    control.bus_ptr.load(Ordering::Acquire) as usize as *mut SystemBus
}
