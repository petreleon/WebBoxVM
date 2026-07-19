use super::*;
use std::sync::atomic::Ordering;

pub(super) fn run(lease: &registry::WorkerLease) -> Result<(), &'static str> {
    let core = lease.core;
    let control = lease.control();
    let cpu = unsafe { &mut *cpu_ptr(control, core) };
    let cache = unsafe { &mut *cache_ptr(control, core) };
    let mut seen_tlb_epoch = control.tlb_epoch.load(Ordering::Acquire);
    let mut local_since_irq_poll = 0u8;

    while !control.stop.load(Ordering::Acquire) {
        if control.system_off.load(Ordering::Acquire) {
            break;
        }
        if apply_power_on(core, cpu, cache, control) {
            seen_tlb_epoch = control.tlb_epoch.load(Ordering::Acquire);
        }
        let lifecycle = control.lifecycle[core].load(Ordering::Acquire);
        if lifecycle == LIFE_RUNNABLE && cpu.lifecycle == CpuLifecycle::WaitingForInterrupt {
            cpu.lifecycle = CpuLifecycle::Runnable;
            cpu.waiting_for_event = false;
            cpu.event_register = control.event_registers[core].load(Ordering::Acquire);
        }
        match lifecycle {
            LIFE_OFF => {
                if idle::coordinate(core, cpu, control) {
                    break;
                }
                continue;
            }
            LIFE_WAITING | LIFE_WAITING_EVENT => {
                if idle::wake_if_ready(core, cpu, control) {
                    continue;
                }
                if idle::coordinate(core, cpu, control) {
                    break;
                }
                continue;
            }
            LIFE_STARTING => {
                std::hint::spin_loop();
                continue;
            }
            _ => {}
        }
        if !control.claim_step() {
            control.stop.store(true, Ordering::Release);
            break;
        }
        let ticket = control.next_cycle.fetch_add(1, Ordering::AcqRel);
        cpu.sys.cycle_count = cpu.sys.cycle_count.max(ticket);
        let epoch = control.tlb_epoch.load(Ordering::Acquire);
        if epoch != seen_tlb_epoch {
            cpu.tlb.invalidate_all();
            seen_tlb_epoch = epoch;
        }
        let Some((pc, instr)) = instruction::fetch(cpu, cache, control) else {
            retire(cpu, control);
            continue;
        };
        if psci::handle(core, cpu, instr, control) {
            retire(cpu, control);
            continue;
        }
        if fp_simd_access_traps(cpu) && is_fp_simd_access(instr) {
            take_fp_simd_trap(cpu, pc);
            control.exec_faults.fetch_add(1, Ordering::Relaxed);
            retire(cpu, control);
            continue;
        }
        if instr.op == Opcode::Tlbi {
            seen_tlb_epoch = control.tlb_epoch.fetch_add(1, Ordering::AcqRel) + 1;
            cpu.tlb.invalidate_all();
        }
        control.observe_local_overlap(true);
        let local_result = try_execute_local(cpu, instr);
        control.observe_local_overlap(false);
        if let Some(result) = local_result {
            if let Err(error) = result {
                instruction::handle_execute_fault(cpu, pc, instr, error, control);
            }
            local_since_irq_poll = local_since_irq_poll.saturating_add(1);
            if local_since_irq_poll >= 32 {
                instruction::deliver_irq(core, cpu, control);
                local_since_irq_poll = 0;
            }
        } else {
            instruction::execute_shared(core, cpu, pc, instr, control);
            local_since_irq_poll = 0;
        }
        match instr.op {
            Opcode::Sevl => events::set_local(core, cpu, control),
            Opcode::Sev => events::broadcast(core, cpu, control),
            Opcode::Wfe => events::park_after_wait(core, cpu, control, true),
            Opcode::Wfi => events::park_after_wait(core, cpu, control, false),
            _ => {}
        }
        retire(cpu, control);
    }
    Ok(())
}

fn apply_power_on(
    core: usize,
    cpu: &mut Armv8Cpu,
    cache: &mut DecodeCache,
    control: &WasmParallelControl,
) -> bool {
    let _guard = control
        .gate
        .write()
        .unwrap_or_else(|poison| poison.into_inner());
    if control.system_off.load(Ordering::Acquire) {
        return false;
    }
    if control.lifecycle[core].load(Ordering::Acquire) != LIFE_BOOT_READY {
        return false;
    }
    cpu.reset();
    *cache = DecodeCache::new();
    cpu.lifecycle = CpuLifecycle::Runnable;
    cpu.pstate = crate::arch::arm64::ProcessorState::el1h_masked();
    cpu.sys.sctlr_el1 &= !SCTLR_MMU_ENABLE;
    cpu.sys.cycle_count = control.next_cycle.load(Ordering::Acquire);
    cpu.regs.pc = control.power_entry[core].load(Ordering::Acquire);
    cpu.regs
        .set_x(0, control.power_context[core].load(Ordering::Acquire));
    control.event_registers[core].store(false, Ordering::Relaxed);
    control.deadlines[core].store(NO_DEADLINE, Ordering::Release);
    control.lifecycle[core].store(LIFE_RUNNABLE, Ordering::Release);
    true
}

fn retire(cpu: &Armv8Cpu, control: &WasmParallelControl) {
    control
        .next_cycle
        .fetch_max(cpu.sys.cycle_count, Ordering::Relaxed);
}

unsafe fn cpu_ptr(control: &WasmParallelControl, core: usize) -> *mut Armv8Cpu {
    let base = control.cpu_base.load(Ordering::Acquire) as usize as *mut Armv8Cpu;
    unsafe { base.add(core) }
}

unsafe fn cache_ptr(control: &WasmParallelControl, core: usize) -> *mut DecodeCache {
    let base = control.cache_base.load(Ordering::Acquire) as usize as *mut DecodeCache;
    unsafe { base.add(core) }
}
