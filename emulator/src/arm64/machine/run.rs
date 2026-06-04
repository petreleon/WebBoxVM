use super::*;

impl Machine {
    /// Run up to `max_total_steps` across all cores using round-robin.
    /// Each core executes one instruction per turn.
    pub fn run(&mut self, max_total_steps: usize) -> usize {
        let start_steps = self.total_steps;
        let num_cores = self.cpus.len();
        let mut report_interval = 1_000_000u64;
        let trace_options = self.trace.options;
        let trace_fetch_hooks = trace_options.has_fetch_hooks();
        let trace_instruction_hooks = trace_options.has_instruction_hooks();
        let trace_syscall_returns = trace_options.has_syscall_return_hooks();

        while (self.total_steps - start_steps) < max_total_steps as u64 {
            let core = self.active_core;
            self.report_progress(start_steps, &mut report_interval, core);

            let pc = self.cpus[core].regs.pc;
            let Some(pa) = self.translate_fetch(core, pc, num_cores) else {
                continue;
            };

            if trace_fetch_hooks {
                self.trace_fetch_hooks(core, pc, pa, trace_options);
            }

            let instr = self.caches[core].fetch(&self.bus.mem, pa);
            let Some(instr) = instr else {
                self.cpus[core].regs.pc += INSTRUCTION_SIZE;
                self.finish_core(core, num_cores);
                continue;
            };

            if trace_instruction_hooks {
                self.trace_instruction_hooks(core, pc, pa, instr, trace_options);
            }

            if self.handle_gic_access(core, instr, num_cores)
                || self.handle_fp_simd_trap(core, pc, pa, instr, trace_options, num_cores)
                || self.execute_or_fault(core, pc, pa, instr, trace_options, num_cores)
            {
                continue;
            }

            if trace_syscall_returns {
                self.trace_syscall_return(core, instr);
            }
            self.deliver_irq(core);
            self.finish_core(core, num_cores);
        }

        (self.total_steps - start_steps) as usize
    }
}
