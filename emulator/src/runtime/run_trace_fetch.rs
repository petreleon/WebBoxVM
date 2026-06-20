use super::*;

impl Machine {
    pub(super) fn trace_fetch_hooks(
        &mut self,
        core: usize,
        pc: u64,
        pa: u64,
        trace_options: TraceOptions,
    ) {
        let cpu = &mut self.cpus[core];
        if trace_options.chase_assert
            && cpu.pstate.el() == 0
            && self.trace.counters.chase_assert < 64
            && trace_chase_assert_check(cpu, &self.bus, pc, pa, self.total_steps)
        {
            self.trace.counters.chase_assert += 1;
        }

        if trace_options.path_extend
            && cpu.pstate.el() == 0
            && self.trace.counters.path_extend < 256
            && trace_path_extend_strlen(cpu, &self.bus, pc, pa, self.total_steps)
        {
            self.trace.counters.path_extend += 1;
        }

        if (trace_options.undecoded || (trace_options.el0_undecoded && cpu.pstate.el() == 0))
            && self.trace.counters.undecoded < 512
        {
            self.trace_undecoded(core, pc, pa);
        }
    }

    fn trace_undecoded(&mut self, core: usize, pc: u64, pa: u64) {
        let raw = self.bus.mem.read(pa, 4).unwrap_or(0) as u32;
        if decode(raw).is_some() {
            return;
        }

        eprintln!(
            "UNDECODED step={} core={} el={} pc=0x{pc:016x} pa=0x{pa:016x} raw=0x{raw:08x}",
            self.total_steps,
            core,
            self.cpus[core].pstate.el(),
        );
        self.trace.counters.undecoded += 1;
    }
}
