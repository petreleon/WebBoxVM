use super::*;

impl Machine {
    pub(super) fn execute_or_fault(
        &mut self,
        core: usize,
        pc: u64,
        pa: u64,
        instr: Instr,
        trace_options: TraceOptions,
        num_cores: usize,
    ) -> bool {
        self.bus.begin_cpu_instruction();
        let result = {
            let cpu = &mut self.cpus[core];
            execute(cpu, &mut self.bus, instr)
        };
        if result.is_ok()
            && matches!(instr.op, Opcode::Ldxr | Opcode::Ldxp)
            && self.cpus[core].exclusive.is_some()
        {
            self.cpus[core].exclusive_epoch = self.memory_epoch;
        }
        self.apply_memory_write_invalidations(core);
        let Err(err) = result else {
            return false;
        };

        self.trace_execute_fault(core, pc, pa, instr, err, trace_options);
        self.exec_faults += 1;
        let cpu = &mut self.cpus[core];
        if is_data_abort_fault(err) {
            take_data_abort(cpu, pc, instr, err, trace_options.el0_faults);
        } else {
            cpu.regs.pc += INSTRUCTION_SIZE;
        }
        self.finish_core(core, num_cores);
        true
    }

    fn trace_execute_fault(
        &mut self,
        core: usize,
        pc: u64,
        pa: u64,
        instr: Instr,
        err: &str,
        trace_options: TraceOptions,
    ) {
        if trace_options.el0_fault_raw
            && self.cpus[core].pstate.el() == 0
            && is_data_abort_fault(err)
        {
            self.trace_el0_fault_raw(core, pc, pa, instr, err);
        }

        if trace_options.faults && self.exec_faults < 64 {
            let cpu = &self.cpus[core];
            eprintln!(
                "EXEC FAULT step={} core={} pc=0x{pc:016x} pa=0x{pa:016x} instr={instr:?}: {err} \
                 x0=0x{:016x} x1=0x{:016x} x2=0x{:016x} x3=0x{:016x} \
                 x29=0x{:016x} lr=0x{:016x} sp=0x{:016x} pstate=0x{:x} \
                 far_el1=0x{:016x} elr_el1=0x{:016x} spsr_el1=0x{:016x}",
                self.total_steps,
                core,
                cpu.regs.x(0),
                cpu.regs.x(1),
                cpu.regs.x(2),
                cpu.regs.x(3),
                cpu.regs.x(29),
                cpu.regs.x(30),
                cpu.regs.sp,
                cpu.pstate.to_u64(),
                cpu.sys.far_el1,
                cpu.sys.elr_el1,
                cpu.sys.spsr_el1,
            );
        }
    }

    fn trace_el0_fault_raw(&mut self, core: usize, pc: u64, pa: u64, instr: Instr, err: &str) {
        let is_main_exec_fault = (0x0000_aaaa_0000_0000..=0x0000_aaab_ffff_ffff).contains(&pc);
        if self.trace.counters.el0_fault_raw >= 512 && !is_main_exec_fault {
            return;
        }

        let cpu = &self.cpus[core];
        eprintln!(
            "EL0 FAULT RAW step={} core={} pc=0x{pc:016x} pa=0x{pa:016x} instr={instr:?} err={err} \
             x0=0x{:016x} x1=0x{:016x} x2=0x{:016x} x3=0x{:016x} x8=0x{:016x} sp=0x{:016x} lr=0x{:016x}",
            self.total_steps,
            core,
            cpu.regs.x(0),
            cpu.regs.x(1),
            cpu.regs.x(2),
            cpu.regs.x(3),
            cpu.regs.x(8),
            cpu.regs.sp,
            cpu.regs.x(30),
        );

        for offset in -4i64..=4 {
            let Some(window_pa) = pa.checked_add_signed(offset * INSTRUCTION_SIZE as i64) else {
                continue;
            };
            let raw = self.bus.mem.read(window_pa, 4).unwrap_or(0xffff_ffff);
            let marker = if offset == 0 { "*" } else { " " };
            eprintln!(
                "  {marker} off={offset:+} pa=0x{window_pa:016x} raw=0x{raw:08x} decoded={:?}",
                decode(raw as u32)
            );
        }
        self.trace.counters.el0_fault_raw += 1;
    }
}
