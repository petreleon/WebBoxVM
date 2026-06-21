use super::*;
use std::{env, sync::OnceLock};

impl Machine {
    pub(super) fn trace_instruction_hooks(
        &mut self,
        core: usize,
        pc: u64,
        pa: u64,
        instr: Instr,
        trace_options: TraceOptions,
    ) {
        if trace_options.syscall_dispatch
            && (0xffff_8000_8002_8800..=0xffff_8000_8002_8a00).contains(&pc)
        {
            trace_syscall_dispatch(&self.cpus[core], pc, instr, self.total_steps);
        }

        let cpu = &mut self.cpus[core];
        if trace_options.writev
            && cpu.pstate.el() == 0
            && instr.op == Opcode::Svc
            && cpu.regs.x(8) == 66
        {
            trace_writev(cpu, &self.bus);
        }
        if trace_options.syscall_paths
            && cpu.pstate.el() == 0
            && instr.op == Opcode::Svc
            && self.trace.counters.syscall_path < 8192
            && let Some(syscall) = trace_syscall_path_entry(cpu, &self.bus, pc, self.total_steps)
        {
            self.trace.pending_syscalls[core] = Some(syscall);
            self.trace.counters.syscall_path += 1;
        }
        if trace_options.exec
            && cpu.pstate.el() == 0
            && instr.op == Opcode::Svc
            && self.trace.counters.exec < 1024
            && self.trace.pending_syscalls[core].is_none()
            && let Some(syscall) = trace_exec_entry(cpu, &self.bus, pc, self.total_steps)
        {
            self.trace.pending_syscalls[core] = Some(syscall);
            self.trace.counters.exec += 1;
        }
        if trace_options.stack_chk && cpu.pstate.el() == 0 {
            trace_stack_chk_enter(cpu, &self.bus, pc, pa, self.total_steps);
        }
        if trace_options.stack_chk && cpu.pstate.el() == 0 && instr.op == Opcode::Bl {
            trace_stack_chk_call(cpu, &self.bus, pc, instr, self.total_steps);
        }
        if trace_options.rwsem
            && self.total_steps > 700_000_000
            && (0xffff_8000_80f3_8a80..=0xffff_8000_80f3_8d80).contains(&pc)
            && self.trace.counters.rwsem < 160
        {
            trace_rwsem_loop(cpu, &self.bus, pc, pa, instr, self.total_steps);
            self.trace.counters.rwsem += 1;
        }
        if trace_options.bpf {
            trace_bpf(cpu, pc, instr, self.total_steps);
        }
        if trace_options.mprotect_loop
            && (0xffff_8000_8037_e840..=0xffff_8000_8037_e864).contains(&pc)
        {
            trace_mprotect_loop_state(cpu, pc, pa, instr, self.total_steps);
        }
        if trace_options.pc_range
            && self.trace.counters.pc_range < pc_range_limit()
            && trace_pc_range_contains(pc)
        {
            trace_pc_range(cpu, pc, pa, instr, self.total_steps);
            self.trace.counters.pc_range += 1;
        }
    }
}

fn trace_syscall_dispatch(cpu: &Armv8Cpu, pc: u64, instr: Instr, step: u64) {
    eprintln!(
        "DISPATCH step={} pc=0x{pc:016x} instr={instr:?} \
         x0=0x{:016x} x1=0x{:016x} x2=0x{:016x} x3=0x{:016x} \
         x8=0x{:016x} x16=0x{:016x} x17=0x{:016x} \
         x19=0x{:016x} x20=0x{:016x} x21=0x{:016x} pstate=0x{:x}",
        step,
        cpu.regs.x(0),
        cpu.regs.x(1),
        cpu.regs.x(2),
        cpu.regs.x(3),
        cpu.regs.x(8),
        cpu.regs.x(16),
        cpu.regs.x(17),
        cpu.regs.x(19),
        cpu.regs.x(20),
        cpu.regs.x(21),
        cpu.pstate.to_u64(),
    );
}

fn trace_bpf(cpu: &Armv8Cpu, pc: u64, instr: Instr, step: u64) {
    let in_bpf_range = (0xffff_8000_8004_66b0..=0xffff_8000_8004_6b40).contains(&pc);
    let from_bpf_flush = (0xffff_8000_8003_6e40..=0xffff_8000_8003_6ec0).contains(&pc)
        && (0xffff_8000_8004_66b0..=0xffff_8000_8004_6b40).contains(&cpu.regs.x(30));
    if !in_bpf_range && !from_bpf_flush {
        return;
    }

    eprintln!(
        "BPF step={} pc=0x{pc:016x} instr={instr:?} \
         x0=0x{:016x} x1=0x{:016x} x2=0x{:016x} x3=0x{:016x} \
         x4=0x{:016x} x5=0x{:016x} x19=0x{:016x} x20=0x{:016x} \
         x21=0x{:016x} x22=0x{:016x} x23=0x{:016x} lr=0x{:016x} \
         sp=0x{:016x} pstate=0x{:x}",
        step,
        cpu.regs.x(0),
        cpu.regs.x(1),
        cpu.regs.x(2),
        cpu.regs.x(3),
        cpu.regs.x(4),
        cpu.regs.x(5),
        cpu.regs.x(19),
        cpu.regs.x(20),
        cpu.regs.x(21),
        cpu.regs.x(22),
        cpu.regs.x(23),
        cpu.regs.x(30),
        cpu.regs.sp,
        cpu.pstate.to_u64(),
    );
}

fn trace_pc_range_contains(pc: u64) -> bool {
    static RANGE: OnceLock<Option<(u64, u64)>> = OnceLock::new();
    match *RANGE.get_or_init(parse_pc_range) {
        Some((start, end)) => (start..=end).contains(&pc),
        None => false,
    }
}

fn parse_pc_range() -> Option<(u64, u64)> {
    let raw = env::var("WEBBOXVM_TRACE_PC_RANGE").ok()?;
    let (start, end) = raw.split_once('-')?;
    let start = parse_hex_u64(start)?;
    let end = parse_hex_u64(end)?;
    (start <= end).then_some((start, end))
}

fn parse_hex_u64(raw: &str) -> Option<u64> {
    let value = raw.trim().trim_start_matches("0x");
    u64::from_str_radix(value, 16).ok()
}

fn pc_range_limit() -> u64 {
    static LIMIT: OnceLock<u64> = OnceLock::new();
    *LIMIT.get_or_init(|| {
        env::var("WEBBOXVM_TRACE_PC_RANGE_LIMIT")
            .ok()
            .and_then(|raw| raw.parse().ok())
            .unwrap_or(2048)
    })
}

fn trace_pc_range(cpu: &Armv8Cpu, pc: u64, pa: u64, instr: Instr, step: u64) {
    eprintln!(
        "PC_RANGE step={} pc=0x{pc:016x} pa=0x{pa:016x} instr={instr:?} \
         x0=0x{:016x} x1=0x{:016x} x2=0x{:016x} x3=0x{:016x} \
         x4=0x{:016x} x19=0x{:016x} x20=0x{:016x} lr=0x{:016x} \
         sp=0x{:016x} pstate=0x{:x}",
        step,
        cpu.regs.x(0),
        cpu.regs.x(1),
        cpu.regs.x(2),
        cpu.regs.x(3),
        cpu.regs.x(4),
        cpu.regs.x(19),
        cpu.regs.x(20),
        cpu.regs.x(30),
        cpu.regs.sp,
        cpu.pstate.to_u64(),
    );
}
