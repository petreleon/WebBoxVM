use super::*;
pub(super) const PSCI_VERSION: u64 = 0x8400_0000;
pub(super) const PSCI_CPU_SUSPEND32: u64 = 0x8400_0001;
pub(super) const PSCI_CPU_SUSPEND64: u64 = 0xc400_0001;
pub(super) const PSCI_CPU_OFF: u64 = 0x8400_0002;
pub(super) const PSCI_CPU_ON32: u64 = 0x8400_0003;
pub(super) const PSCI_CPU_ON64: u64 = 0xc400_0003;
pub(super) const PSCI_AFFINITY_INFO32: u64 = 0x8400_0004;
pub(super) const PSCI_AFFINITY_INFO64: u64 = 0xc400_0004;
pub(super) const PSCI_MIGRATE_INFO_TYPE: u64 = 0x8400_0006;
pub(super) const PSCI_SYSTEM_OFF: u64 = 0x8400_0008;
pub(super) const PSCI_SYSTEM_RESET: u64 = 0x8400_0009;
pub(super) const PSCI_SUSPEND_POWERDOWN: u64 = 1 << 16;
pub(super) const PSCI_SUCCESS: i32 = 0;
pub(super) const PSCI_NOT_SUPPORTED: i32 = -1;
pub(super) const PSCI_INVALID_PARAMETERS: i32 = -2;
pub(super) const PSCI_ALREADY_ON: i32 = -4;
pub(super) const PSCI_AFFINITY_ON: i32 = 0;
pub(super) const PSCI_AFFINITY_OFF: i32 = 1;
impl Machine {
    pub(super) fn handle_psci_call(
        &mut self,
        caller: usize,
        instr: Instr,
        num_cores: usize,
    ) -> bool {
        if instr.op != Opcode::Hvc
            || instr.imm != 0
            || !matches!(self.cpus[caller].pstate.el(), 1 | 2)
        {
            return false;
        }
        let function = self.cpus[caller].regs.x(0);
        let arg1 = self.cpus[caller].regs.x(1);
        let arg2 = self.cpus[caller].regs.x(2);
        let arg3 = self.cpus[caller].regs.x(3);
        if function == PSCI_CPU_OFF {
            self.cpus[caller].lifecycle = CpuLifecycle::PoweredOff;
            self.finish_core(caller, num_cores);
            return true;
        }
        if function == PSCI_SYSTEM_OFF {
            for cpu in &mut self.cpus {
                cpu.lifecycle = CpuLifecycle::PoweredOff;
            }
            self.finish_core(caller, num_cores);
            return true;
        }
        if function == PSCI_SYSTEM_RESET {
            self.finish_core(caller, num_cores);
            self.psci_system_reset();
            return true;
        }
        let result = match function {
            PSCI_VERSION => 2,
            PSCI_CPU_SUSPEND32 | PSCI_CPU_SUSPEND64 => self.psci_cpu_suspend(arg1, arg2),
            PSCI_CPU_ON32 | PSCI_CPU_ON64 => self.psci_cpu_on(arg1, arg2, arg3),
            PSCI_AFFINITY_INFO32 | PSCI_AFFINITY_INFO64 => self.psci_affinity_info(arg1, arg2),
            PSCI_MIGRATE_INFO_TYPE => PSCI_NOT_SUPPORTED,
            _ => PSCI_NOT_SUPPORTED,
        };
        let cpu = &mut self.cpus[caller];
        cpu.regs.set_x(0, psci_result(result));
        cpu.regs.pc += INSTRUCTION_SIZE;
        self.finish_core(caller, num_cores);
        true
    }

    fn psci_cpu_suspend(&self, power_state: u64, entry: u64) -> i32 {
        // PSCI 0.2 permits core powerdown to be downgraded to standby and
        // return SUCCESS at the instruction following the call.
        match power_state {
            0 => PSCI_SUCCESS,
            PSCI_SUSPEND_POWERDOWN
                if entry & (INSTRUCTION_SIZE - 1) == 0 && self.bus.mem.contains_range(entry, 4) =>
            {
                PSCI_SUCCESS
            }
            _ => PSCI_INVALID_PARAMETERS,
        }
    }

    fn psci_cpu_on(&mut self, affinity: u64, entry: u64, context: u64) -> i32 {
        let Some(target_id) = affinity_core_id(affinity, self.cpus.len()) else {
            return PSCI_INVALID_PARAMETERS;
        };
        if entry & (INSTRUCTION_SIZE - 1) != 0 || !self.bus.mem.contains_range(entry, 4) {
            return PSCI_INVALID_PARAMETERS;
        }
        if self.cpus[target_id].lifecycle != CpuLifecycle::PoweredOff {
            return PSCI_ALREADY_ON;
        }

        let target = &mut self.cpus[target_id];
        target.reset();
        target.lifecycle = CpuLifecycle::Runnable;
        target.pstate = crate::arch::arm64::ProcessorState::el1h_masked();
        target.sys.sctlr_el1 &= !SCTLR_MMU_ENABLE;
        target.sys.cycle_count = self.virtual_time;
        target.regs.pc = entry;
        target.regs.set_x(0, context);
        PSCI_SUCCESS
    }

    fn psci_affinity_info(&self, affinity: u64, lowest_level: u64) -> i32 {
        if lowest_level == 0 {
            let Some(target_id) = affinity_core_id(affinity, self.cpus.len()) else {
                return PSCI_INVALID_PARAMETERS;
            };
            return lifecycle_affinity_state(self.cpus[target_id].lifecycle);
        }
        if !(1..=3).contains(&lowest_level)
            || flat_parent_affinity(affinity, lowest_level).is_none()
        {
            return PSCI_INVALID_PARAMETERS;
        }
        self.cpus
            .iter()
            .map(|cpu| lifecycle_affinity_state(cpu.lifecycle))
            .min()
            .unwrap_or(PSCI_AFFINITY_OFF)
    }

    pub(super) fn psci_system_reset(&mut self) {
        let num_cores = self.cpus.len();
        self.bus.cold_reset_devices(num_cores);
        if !self.reset_memory.is_empty() {
            self.bus.mem = crate::memory::PhysicalMemory::new();
            for (addr, bytes) in &self.reset_memory {
                self.bus
                    .mem
                    .write_bytes(*addr, bytes)
                    .expect("validated reset image must fit in guest memory");
            }
        }
        for cpu in &mut self.cpus {
            cpu.reset();
            cpu.lifecycle = CpuLifecycle::PoweredOff;
        }
        let primary = &mut self.cpus[0];
        primary.lifecycle = CpuLifecycle::Runnable;
        primary.pstate = crate::arch::arm64::ProcessorState::el1h_masked();
        primary.sys.sctlr_el1 &= !SCTLR_MMU_ENABLE;
        primary.regs.pc = self.reset_entry;
        primary.regs.set_x(0, self.reset_arg0);
        self.caches = (0..num_cores).map(|_| DecodeCache::new()).collect();
        self.trace.pending_syscalls.fill(None);
        self.active_core = 0;
        self.virtual_time = 0;
    }
}

pub(super) fn affinity_core_id(affinity: u64, cpu_count: usize) -> Option<usize> {
    let affinity = affinity & !0x8000_0000;
    (affinity & !0xff == 0)
        .then_some(affinity as usize)
        .filter(|&core| core < cpu_count)
}

pub(super) fn flat_parent_affinity(affinity: u64, lowest_level: u64) -> Option<()> {
    let affinity = affinity & !0x8000_0000;
    let ignored_bits = 8 * lowest_level;
    (affinity >> ignored_bits == 0).then_some(())
}

fn lifecycle_affinity_state(lifecycle: CpuLifecycle) -> i32 {
    match lifecycle {
        CpuLifecycle::PoweredOff => PSCI_AFFINITY_OFF,
        CpuLifecycle::Runnable | CpuLifecycle::WaitingForInterrupt => PSCI_AFFINITY_ON,
    }
}

pub(super) fn psci_result(result: i32) -> u64 {
    result as i64 as u64
}

#[cfg(test)]
mod tests;
