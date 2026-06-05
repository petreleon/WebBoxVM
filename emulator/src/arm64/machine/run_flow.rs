use super::*;

impl Machine {
    pub(super) fn finish_core(&mut self, core: usize, num_cores: usize) {
        self.total_steps += 1;
        self.active_core = (core + 1) % num_cores;
    }

    pub(super) fn report_progress(&self, start_steps: u64, report_interval: &mut u64, core: usize) {
        let elapsed = self.total_steps - start_steps;
        if elapsed == 0 || !elapsed.is_multiple_of(*report_interval) {
            return;
        }

        eprintln!(
            "DIAG {:>9}M steps | fetch_faults={:>7} exec_faults={:>7} | PC=0x{:016x}",
            elapsed / 1_000_000,
            self.fetch_faults,
            self.exec_faults,
            self.cpus[core].regs.pc
        );
        if elapsed >= 10_000_000 {
            *report_interval = 100_000_000;
        }
    }

    pub(super) fn translate_fetch(
        &mut self,
        core: usize,
        pc: u64,
        num_cores: usize,
    ) -> Option<u64> {
        let cpu = &mut self.cpus[core];
        match translate(&cpu.sys, &mut cpu.tlb, &self.bus.mem, pc) {
            Ok(pa) => Some(pa),
            Err(_) => {
                self.fetch_faults += 1;
                if cpu.sys.vbar_el1 != 0 && (cpu.sys.sctlr_el1 & SCTLR_MMU_ENABLE) != 0 {
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
        let cpu = &mut self.cpus[core];
        if !handle_gic_sysreg_access(cpu, &mut self.bus, instr) {
            return false;
        }
        self.finish_core(core, num_cores);
        true
    }

    pub(super) fn deliver_irq(&mut self, core: usize) {
        self.bus.refresh_interrupts();
        deliver_external_irq(&mut self.cpus[core], &mut self.bus);
    }
}
