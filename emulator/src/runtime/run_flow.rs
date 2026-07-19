use super::*;

impl Machine {
    pub(super) fn finish_core(&mut self, core: usize, num_cores: usize) {
        self.finish_core_steps(core, num_cores, 1);
    }

    /// Finish a cooperatively executed JIT quantum for one runnable core.
    ///
    /// The JIT updates the core-local cycle counter before calling this method.
    /// Advancing global time by the same number of guest instructions preserves
    /// the serial scheduler invariant, then the next quantum starts at the next
    /// runnable core in round-robin order.
    #[cfg(feature = "wasm")]
    pub(crate) fn finish_jit_core(&mut self, core: usize, steps: u64) {
        self.finish_core_steps(core, self.cpus.len(), steps);
    }

    fn finish_core_steps(&mut self, core: usize, num_cores: usize, steps: u64) {
        self.total_steps = self.total_steps.wrapping_add(steps);
        self.virtual_time = self
            .virtual_time
            .saturating_add(steps)
            .max(self.cpus[core].sys.cycle_count);
        let next = if core + 1 == num_cores { 0 } else { core + 1 };
        self.active_core = self.find_runnable_from(next).unwrap_or(next);
    }

    pub(super) fn report_progress(&self, start_steps: u64, next_report_at: &mut u64, core: usize) {
        let total_steps = self.total_steps;
        if total_steps < *next_report_at {
            return;
        }

        let elapsed = total_steps.saturating_sub(start_steps);
        eprintln!(
            "DIAG {:>9}M steps | fetch_faults={:>7} exec_faults={:>7} | PC=0x{:016x}",
            elapsed / 1_000_000,
            self.fetch_faults,
            self.exec_faults,
            self.cpus[core].regs.pc
        );
        let next_elapsed = if elapsed >= 10_000_000 {
            if elapsed < 100_000_000 {
                100_000_000
            } else {
                elapsed.saturating_add(100_000_000)
            }
        } else {
            elapsed.saturating_add(1_000_000)
        };
        *next_report_at = start_steps.saturating_add(next_elapsed);
    }

    pub(super) fn translate_fetch(
        &mut self,
        core: usize,
        pc: u64,
        num_cores: usize,
    ) -> Option<u64> {
        let cpu = &mut self.cpus[core];
        if cpu.sys.sctlr_el1 & SCTLR_MMU_ENABLE == 0 {
            return Some(pc);
        }
        match translate(&cpu.sys, &mut cpu.tlb, &self.bus.mem, pc) {
            Ok(pa) => Some(pa),
            Err(_) => {
                self.fetch_faults += 1;
                if cpu.sys.vbar_el1 != 0 {
                    cpu.sys.far_el1 = pc;
                    take_instruction_abort(cpu, pc);
                } else {
                    cpu.regs.pc += INSTRUCTION_SIZE;
                }
                self.finish_core(core, num_cores);
                None
            }
        }
    }

    pub(super) fn handle_gic_access(
        &mut self,
        core: usize,
        instr: Instr,
        num_cores: usize,
    ) -> bool {
        if !matches!(instr.op, Opcode::Mrs | Opcode::Msr) {
            return false;
        }
        let cpu = &mut self.cpus[core];
        if !handle_gic_sysreg_access(cpu, &mut self.bus, instr) {
            return false;
        }
        self.finish_core(core, num_cores);
        true
    }

    pub(super) fn deliver_irq(&mut self, core: usize) {
        if !self.bus.external_irq_poll_needed_for_cpu(core) {
            return;
        }
        self.bus.refresh_interrupts();
        if !self.bus.external_irq_poll_needed_for_cpu(core) {
            return;
        }
        deliver_external_irq(&mut self.cpus[core], &mut self.bus, core);
    }
}
