use super::*;
use crate::runtime::psci::*;
use std::sync::atomic::Ordering;

pub(in crate::runtime) fn handle(
    caller: usize,
    cpu: &mut Armv8Cpu,
    instr: Instr,
    shared: &SharedRun<'_>,
) -> bool {
    if instr.op != Opcode::Hvc || instr.imm != 0 || !matches!(cpu.pstate.el(), 1 | 2) {
        return false;
    }
    let function = cpu.regs.x(0);
    let arg1 = cpu.regs.x(1);
    let arg2 = cpu.regs.x(2);
    let arg3 = cpu.regs.x(3);
    if function == PSCI_CPU_OFF {
        set_lifecycle(caller, cpu, LIFE_OFF, CpuLifecycle::PoweredOff, shared);
        return true;
    }
    if function == PSCI_SYSTEM_OFF {
        let _guard = shared
            .idle_gate
            .write()
            .unwrap_or_else(|poison| poison.into_inner());
        shared.system_off.store(true, Ordering::Release);
        shared.stop.store(true, Ordering::Release);
        for (core, state) in shared.lifecycle.iter().enumerate() {
            shared.deadlines[core].store(NO_DEADLINE, Ordering::Relaxed);
            state.store(LIFE_OFF, Ordering::Release);
        }
        cpu.lifecycle = CpuLifecycle::PoweredOff;
        return true;
    }
    if function == PSCI_SYSTEM_RESET {
        cpu.regs.pc += INSTRUCTION_SIZE;
        shared.reset_requested.store(true, Ordering::Release);
        shared.stop.store(true, Ordering::Release);
        return true;
    }
    let result = match function {
        PSCI_VERSION => 2,
        PSCI_CPU_SUSPEND32 | PSCI_CPU_SUSPEND64 => suspend(arg1, arg2, shared),
        PSCI_CPU_ON32 | PSCI_CPU_ON64 => cpu_on(arg1, arg2, arg3, shared),
        PSCI_AFFINITY_INFO32 | PSCI_AFFINITY_INFO64 => affinity_info(arg1, arg2, shared),
        PSCI_MIGRATE_INFO_TYPE => PSCI_NOT_SUPPORTED,
        _ => PSCI_NOT_SUPPORTED,
    };
    cpu.regs.set_x(0, psci_result(result));
    cpu.regs.pc += INSTRUCTION_SIZE;
    true
}

fn suspend(power_state: u64, entry: u64, shared: &SharedRun<'_>) -> i32 {
    if power_state == 0 {
        return PSCI_SUCCESS;
    }
    let bus = shared
        .bus
        .read()
        .unwrap_or_else(|poison| poison.into_inner());
    if power_state == PSCI_SUSPEND_POWERDOWN
        && entry & (INSTRUCTION_SIZE - 1) == 0
        && bus.mem.contains_range(entry, 4)
    {
        PSCI_SUCCESS
    } else {
        PSCI_INVALID_PARAMETERS
    }
}

fn cpu_on(affinity: u64, entry: u64, context: u64, shared: &SharedRun<'_>) -> i32 {
    let Some(target) = affinity_core_id(affinity, shared.lifecycle.len()) else {
        return PSCI_INVALID_PARAMETERS;
    };
    let _guard = shared
        .idle_gate
        .write()
        .unwrap_or_else(|poison| poison.into_inner());
    if shared.system_off.load(Ordering::Acquire) {
        return PSCI_ALREADY_ON;
    }
    let bus = shared
        .bus
        .read()
        .unwrap_or_else(|poison| poison.into_inner());
    if entry & (INSTRUCTION_SIZE - 1) != 0 || !bus.mem.contains_range(entry, 4) {
        return PSCI_INVALID_PARAMETERS;
    }
    drop(bus);
    if shared.lifecycle[target]
        .compare_exchange(LIFE_OFF, LIFE_STARTING, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return PSCI_ALREADY_ON;
    }
    if shared.system_off.load(Ordering::Acquire) {
        let _ = shared.lifecycle[target].compare_exchange(
            LIFE_STARTING,
            LIFE_OFF,
            Ordering::AcqRel,
            Ordering::Acquire,
        );
        return PSCI_ALREADY_ON;
    }
    shared.power_entry[target].store(entry, Ordering::Relaxed);
    shared.power_context[target].store(context, Ordering::Relaxed);
    if shared.lifecycle[target]
        .compare_exchange(
            LIFE_STARTING,
            LIFE_BOOT_READY,
            Ordering::AcqRel,
            Ordering::Acquire,
        )
        .is_ok()
    {
        PSCI_SUCCESS
    } else {
        PSCI_ALREADY_ON
    }
}

fn affinity_info(affinity: u64, lowest_level: u64, shared: &SharedRun<'_>) -> i32 {
    if lowest_level == 0 {
        let Some(target) = affinity_core_id(affinity, shared.lifecycle.len()) else {
            return PSCI_INVALID_PARAMETERS;
        };
        return affinity_state(shared.lifecycle[target].load(Ordering::Acquire));
    }
    if !(1..=3).contains(&lowest_level) || flat_parent_affinity(affinity, lowest_level).is_none() {
        return PSCI_INVALID_PARAMETERS;
    }
    shared
        .lifecycle
        .iter()
        .map(|state| affinity_state(state.load(Ordering::Acquire)))
        .min()
        .unwrap_or(PSCI_AFFINITY_OFF)
}

fn affinity_state(lifecycle: u8) -> i32 {
    if lifecycle == LIFE_OFF {
        PSCI_AFFINITY_OFF
    } else {
        PSCI_AFFINITY_ON
    }
}

fn set_lifecycle(
    core: usize,
    cpu: &mut Armv8Cpu,
    code: u8,
    lifecycle: CpuLifecycle,
    shared: &SharedRun<'_>,
) {
    let _guard = shared
        .idle_gate
        .write()
        .unwrap_or_else(|poison| poison.into_inner());
    cpu.lifecycle = lifecycle;
    shared.deadlines[core].store(NO_DEADLINE, Ordering::Relaxed);
    shared.lifecycle[core].store(code, Ordering::Release);
}
