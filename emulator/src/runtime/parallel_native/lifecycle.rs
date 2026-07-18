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

pub(in crate::runtime) fn lifecycle_code(lifecycle: CpuLifecycle) -> u8 {
    match lifecycle {
        CpuLifecycle::PoweredOff => LIFE_OFF,
        CpuLifecycle::Runnable => LIFE_RUNNABLE,
        CpuLifecycle::WaitingForInterrupt => LIFE_WAITING,
    }
}
