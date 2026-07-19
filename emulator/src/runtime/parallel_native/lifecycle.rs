use super::*;

pub(in crate::runtime) fn initialize_powered_on_core(
    cpu: &mut Armv8Cpu,
    cache: &mut DecodeCache,
    entry: u64,
    context: u64,
    cycle: u64,
) {
    cpu.reset();
    *cache = DecodeCache::new();
    cpu.lifecycle = CpuLifecycle::Runnable;
    cpu.pstate = crate::arch::arm64::ProcessorState::el1h_masked();
    cpu.sys.sctlr_el1 &= !SCTLR_MMU_ENABLE;
    cpu.sys.cycle_count = cycle;
    cpu.regs.pc = entry;
    cpu.regs.set_x(0, context);
}

pub(in crate::runtime) fn lifecycle_code(cpu: &Armv8Cpu) -> u8 {
    match cpu.lifecycle {
        CpuLifecycle::PoweredOff => LIFE_OFF,
        CpuLifecycle::Runnable => LIFE_RUNNABLE,
        CpuLifecycle::WaitingForInterrupt if cpu.waiting_for_event => LIFE_WAITING_EVENT,
        CpuLifecycle::WaitingForInterrupt => LIFE_WAITING,
    }
}

pub(super) fn sync_cpus(
    cpus: &mut [Armv8Cpu],
    caches: &mut [DecodeCache],
    cycle: u64,
    shared: &SharedRun<'_>,
    system_off: bool,
) {
    for core in 0..cpus.len() {
        cpus[core].event_register = shared.event_registers[core].load(Ordering::Acquire);
        cpus[core].waiting_for_event = false;
        if system_off {
            cpus[core].lifecycle = CpuLifecycle::PoweredOff;
            cpus[core].event_register = false;
            continue;
        }
        match shared.lifecycle[core].load(Ordering::Acquire) {
            LIFE_OFF => cpus[core].lifecycle = CpuLifecycle::PoweredOff,
            LIFE_WAITING => cpus[core].lifecycle = CpuLifecycle::WaitingForInterrupt,
            LIFE_WAITING_EVENT => {
                cpus[core].lifecycle = CpuLifecycle::WaitingForInterrupt;
                cpus[core].waiting_for_event = true;
            }
            LIFE_STARTING | LIFE_BOOT_READY => {
                let entry = shared.power_entry[core].load(Ordering::Acquire);
                let context = shared.power_context[core].load(Ordering::Acquire);
                initialize_powered_on_core(
                    &mut cpus[core],
                    &mut caches[core],
                    entry,
                    context,
                    cycle,
                );
            }
            _ => cpus[core].lifecycle = CpuLifecycle::Runnable,
        }
    }
}
