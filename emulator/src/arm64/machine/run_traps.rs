use super::*;

impl Machine {
    pub(super) fn handle_fp_simd_trap(
        &mut self,
        core: usize,
        pc: u64,
        pa: u64,
        instr: Instr,
        trace_options: TraceOptions,
        num_cores: usize,
    ) -> bool {
        let cpu = &mut self.cpus[core];
        if !fp_simd_access_traps(cpu) || !is_fp_simd_access(instr) {
            return false;
        }

        if trace_options.fp_traps && self.trace.counters.fp_simd_trap < 128 {
            eprintln!(
                "FP_SIMD_TRAP step={} core={} el={} pc=0x{pc:016x} pa=0x{pa:016x} instr={instr:?} \
                 cpacr=0x{:016x} fpen={}",
                self.total_steps,
                core,
                cpu.pstate.el(),
                cpu.sys.cpacr_el1,
                (cpu.sys.cpacr_el1 & CPACR_FPEN_MASK) >> CPACR_FPEN_SHIFT,
            );
            self.trace.counters.fp_simd_trap += 1;
        }

        take_fp_simd_trap(cpu, pc);
        self.finish_core(core, num_cores);
        true
    }

    pub(super) fn trace_syscall_return(&mut self, core: usize, instr: Instr) {
        let cpu = &mut self.cpus[core];
        if instr.op == Opcode::Eret
            && cpu.pstate.el() == 0
            && let Some(syscall) = self.trace.pending_syscalls[core].take()
        {
            trace_syscall_path_return(cpu, &self.bus, syscall);
        }
    }
}
