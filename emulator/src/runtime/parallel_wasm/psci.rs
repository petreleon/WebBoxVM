use super::*;
use crate::runtime::psci::*;
use std::sync::atomic::Ordering;

pub(super) fn handle(
    caller: usize,
    cpu: &mut Armv8Cpu,
    instr: Instr,
    control: &WasmParallelControl,
) -> bool {
    if instr.op != Opcode::Hvc || instr.imm != 0 || !matches!(cpu.pstate.el(), 1 | 2) {
        return false;
    }
    let function = cpu.regs.x(0);
    let arg1 = cpu.regs.x(1);
    let arg2 = cpu.regs.x(2);
    let arg3 = cpu.regs.x(3);
    if function == PSCI_CPU_OFF {
        power_off(caller, cpu, control);
        return true;
    }
    if function == PSCI_SYSTEM_OFF {
        latch_system_off(cpu, control);
        return true;
    }
    if function == PSCI_SYSTEM_RESET {
        cpu.regs.pc += INSTRUCTION_SIZE;
        request_reset(control);
        return true;
    }
    let result = match function {
        PSCI_VERSION => 2,
        PSCI_CPU_SUSPEND32 | PSCI_CPU_SUSPEND64 => suspend(arg1, arg2, control),
        PSCI_CPU_ON32 | PSCI_CPU_ON64 => cpu_on(arg1, arg2, arg3, control),
        PSCI_AFFINITY_INFO32 | PSCI_AFFINITY_INFO64 => affinity_info(arg1, arg2, control),
        PSCI_MIGRATE_INFO_TYPE => PSCI_NOT_SUPPORTED,
        _ => PSCI_NOT_SUPPORTED,
    };
    cpu.regs.set_x(0, psci_result(result));
    cpu.regs.pc += INSTRUCTION_SIZE;
    true
}

fn suspend(power_state: u64, entry: u64, control: &WasmParallelControl) -> i32 {
    if power_state == 0 {
        return PSCI_SUCCESS;
    }
    let _guard = control
        .gate
        .read()
        .unwrap_or_else(|poison| poison.into_inner());
    let bus = unsafe { &*bus_ptr(control) };
    if power_state == PSCI_SUSPEND_POWERDOWN
        && entry & (INSTRUCTION_SIZE - 1) == 0
        && bus.mem.contains_range(entry, 4)
    {
        PSCI_SUCCESS
    } else {
        PSCI_INVALID_PARAMETERS
    }
}

fn cpu_on(affinity: u64, entry: u64, context: u64, control: &WasmParallelControl) -> i32 {
    let Some(target) = affinity_core_id(affinity, control.lifecycle.len()) else {
        return PSCI_INVALID_PARAMETERS;
    };
    let _guard = control
        .gate
        .write()
        .unwrap_or_else(|poison| poison.into_inner());
    if control.system_off.load(Ordering::Acquire) {
        return PSCI_ALREADY_ON;
    }
    let bus = unsafe { &*bus_ptr(control) };
    if entry & (INSTRUCTION_SIZE - 1) != 0 || !bus.mem.contains_range(entry, 4) {
        return PSCI_INVALID_PARAMETERS;
    }
    if control.lifecycle[target]
        .compare_exchange(LIFE_OFF, LIFE_STARTING, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return PSCI_ALREADY_ON;
    }
    control.power_entry[target].store(entry, Ordering::Relaxed);
    control.power_context[target].store(context, Ordering::Relaxed);
    control.lifecycle[target].store(LIFE_BOOT_READY, Ordering::Release);
    PSCI_SUCCESS
}

fn power_off(caller: usize, cpu: &mut Armv8Cpu, control: &WasmParallelControl) {
    let _guard = control
        .gate
        .write()
        .unwrap_or_else(|poison| poison.into_inner());
    cpu.lifecycle = CpuLifecycle::PoweredOff;
    control.deadlines[caller].store(NO_DEADLINE, Ordering::Relaxed);
    control.lifecycle[caller].store(LIFE_OFF, Ordering::Release);
}

fn latch_system_off(cpu: &mut Armv8Cpu, control: &WasmParallelControl) {
    let _guard = control
        .gate
        .write()
        .unwrap_or_else(|poison| poison.into_inner());
    control.system_off.store(true, Ordering::Release);
    for (core, state) in control.lifecycle.iter().enumerate() {
        control.deadlines[core].store(NO_DEADLINE, Ordering::Relaxed);
        state.store(LIFE_OFF, Ordering::Release);
    }
    cpu.lifecycle = CpuLifecycle::PoweredOff;
    control.stop.store(true, Ordering::Release);
}

fn request_reset(control: &WasmParallelControl) {
    let _guard = control
        .gate
        .read()
        .unwrap_or_else(|poison| poison.into_inner());
    if !control.system_off.load(Ordering::Acquire) {
        control.reset_requested.store(true, Ordering::Release);
    }
    control.stop.store(true, Ordering::Release);
}

fn affinity_info(affinity: u64, lowest_level: u64, control: &WasmParallelControl) -> i32 {
    if lowest_level == 0 {
        let Some(target) = affinity_core_id(affinity, control.lifecycle.len()) else {
            return PSCI_INVALID_PARAMETERS;
        };
        return affinity_state(control.lifecycle[target].load(Ordering::Acquire));
    }
    if !(1..=3).contains(&lowest_level) || flat_parent_affinity(affinity, lowest_level).is_none() {
        return PSCI_INVALID_PARAMETERS;
    }
    control
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

unsafe fn bus_ptr(control: &WasmParallelControl) -> *mut SystemBus {
    control.bus_ptr.load(Ordering::Acquire) as usize as *mut SystemBus
}
