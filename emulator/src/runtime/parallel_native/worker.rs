use super::*;
use std::sync::atomic::Ordering;

pub(super) fn run(
    core: usize,
    cpu: &mut Armv8Cpu,
    cache: &mut DecodeCache,
    shared: &SharedRun<'_>,
) {
    let mut seen_tlb_epoch = shared.tlb_epoch.load(Ordering::Acquire);
    let mut local_since_irq_poll = 0u8;

    while !shared.stop.load(Ordering::Acquire) {
        if apply_power_on(core, cpu, cache, shared) {
            seen_tlb_epoch = shared.tlb_epoch.load(Ordering::Acquire);
        }
        match shared.lifecycle[core].load(Ordering::Acquire) {
            LIFE_OFF => {
                if idle::coordinate(core, cpu, shared) {
                    break;
                }
                continue;
            }
            LIFE_WAITING => {
                if idle::wake_if_ready(core, cpu, shared) {
                    continue;
                }
                if idle::coordinate(core, cpu, shared) {
                    break;
                }
                continue;
            }
            LIFE_STARTING => {
                std::thread::yield_now();
                continue;
            }
            _ => {}
        }
        if !shared.claim_step() {
            shared.stop.store(true, Ordering::Release);
            break;
        }
        let ticket = shared.next_cycle.fetch_add(1, Ordering::AcqRel);
        cpu.sys.cycle_count = cpu.sys.cycle_count.max(ticket);
        let epoch = shared.tlb_epoch.load(Ordering::Acquire);
        if epoch != seen_tlb_epoch {
            cpu.tlb.invalidate_all();
            seen_tlb_epoch = epoch;
        }

        let Some((pc, _pa, instr)) = instruction::fetch(cpu, cache, shared) else {
            retire(cpu, shared);
            continue;
        };
        if psci::handle(core, cpu, instr, shared) {
            retire(cpu, shared);
            continue;
        }
        if fp_simd_access_traps(cpu) && is_fp_simd_access(instr) {
            take_fp_simd_trap(cpu, pc);
            shared.exec_faults.fetch_add(1, Ordering::Relaxed);
            retire(cpu, shared);
            continue;
        }
        if instr.op == Opcode::Tlbi {
            seen_tlb_epoch = shared.tlb_epoch.fetch_add(1, Ordering::AcqRel) + 1;
            cpu.tlb.invalidate_all();
        }

        shared.observe_local_overlap(true);
        let local_result = try_execute_local(cpu, instr);
        shared.observe_local_overlap(false);
        if let Some(result) = local_result {
            if let Err(error) = result {
                instruction::handle_execute_fault(cpu, pc, instr, error, shared);
            }
            local_since_irq_poll = local_since_irq_poll.saturating_add(1);
            if local_since_irq_poll >= 32 {
                instruction::deliver_irq(core, cpu, shared);
                local_since_irq_poll = 0;
            }
        } else {
            instruction::execute_shared(core, cpu, pc, instr, shared);
            local_since_irq_poll = 0;
        }
        if instr.op == Opcode::Wfi && !idle::has_wake_event(core, cpu, shared) {
            idle::park_after_wfi(core, cpu, shared);
        }
        retire(cpu, shared);
    }
}

fn apply_power_on(
    core: usize,
    cpu: &mut Armv8Cpu,
    cache: &mut DecodeCache,
    shared: &SharedRun<'_>,
) -> bool {
    if shared.lifecycle[core].load(Ordering::Acquire) != LIFE_BOOT_READY {
        return false;
    }
    let entry = shared.power_entry[core].load(Ordering::Acquire);
    let context = shared.power_context[core].load(Ordering::Acquire);
    initialize_powered_on_core(
        cpu,
        cache,
        entry,
        context,
        shared.next_cycle.load(Ordering::Acquire),
    );
    if shared.lifecycle[core]
        .compare_exchange(
            LIFE_BOOT_READY,
            LIFE_RUNNABLE,
            Ordering::AcqRel,
            Ordering::Acquire,
        )
        .is_ok()
    {
        true
    } else {
        cpu.lifecycle = CpuLifecycle::PoweredOff;
        false
    }
}

fn retire(cpu: &Armv8Cpu, shared: &SharedRun<'_>) {
    shared
        .next_cycle
        .fetch_max(cpu.sys.cycle_count, Ordering::Relaxed);
}
