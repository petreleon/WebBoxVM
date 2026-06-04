//! Multi-core ARM64 machine — orchestrates N CPU cores sharing one SystemBus.
//!
//! Each core runs one instruction at a time in round-robin fashion,
//! with a per-core decode cache to avoid re-decoding the same page.

use crate::arm64::gic_sysregs::handle_gic_sysreg_access;
use crate::arm64::machine_trace::{TraceOptions, TraceState, TraceSyscall};
use crate::arm64::{Armv8Cpu, DecodeCache, Instr, Opcode, cond_taken, decode, execute, translate};
use crate::bus::SystemBus;
use crate::constants::*;

/// Multi-core virtual machine with shared memory bus.
pub struct Machine {
    pub cpus: Vec<Armv8Cpu>,
    pub bus: SystemBus,
    caches: Vec<DecodeCache>, // one decode cache per core
    trace: TraceState,
    pub active_core: usize,
    pub total_steps: u64,
    pub fetch_faults: u64,
    pub exec_faults: u64,
}

impl Machine {
    /// Create a machine with `num_cores` CPUs sharing a single system bus.
    pub fn new(num_cores: usize) -> Self {
        Self::with_trace_options(num_cores, TraceOptions::from_env())
    }

    pub(crate) fn with_trace_options(num_cores: usize, trace_options: TraceOptions) -> Self {
        let cpus: Vec<_> = (0..num_cores)
            .map(|i| Armv8Cpu::with_core(i as u32))
            .collect();
        let caches = (0..num_cores).map(|_| DecodeCache::new()).collect();
        Self {
            cpus,
            bus: SystemBus::new(),
            caches,
            trace: TraceState::new(num_cores, trace_options),
            active_core: 0,
            total_steps: 0,
            fetch_faults: 0,
            exec_faults: 0,
        }
    }

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
            let cpu = &mut self.cpus[core];
            let cache = &mut self.caches[core];

            if (self.total_steps - start_steps) >= max_total_steps as u64 {
                break;
            }

            // Periodic diagnostic report
            if (self.total_steps - start_steps) > 0
                && (self.total_steps - start_steps).is_multiple_of(report_interval)
            {
                let pc = cpu.regs.pc;
                eprintln!(
                    "DIAG {:>9}M steps | fetch_faults={:>7} exec_faults={:>7} | PC=0x{:016x}",
                    (self.total_steps - start_steps) / 1_000_000,
                    self.fetch_faults,
                    self.exec_faults,
                    pc
                );
                if (self.total_steps - start_steps) >= 10_000_000 {
                    report_interval = 100_000_000;
                }
            }

            let pc = cpu.regs.pc;

            let pa = match translate(&cpu.sys, &mut cpu.tlb, &self.bus.mem, pc) {
                Ok(pa) => pa,
                Err(_) => {
                    self.fetch_faults += 1;
                    if cpu.sys.vbar_el1 != 0 && (cpu.sys.sctlr_el1 & SCTLR_MMU_ENABLE) != 0 {
                        cpu.sys.far_el1 = pc;
                        take_instruction_abort(cpu, pc);
                    } else {
                        cpu.regs.pc += INSTRUCTION_SIZE;
                    }
                    self.total_steps += 1;
                    self.active_core = (core + 1) % num_cores;
                    continue;
                }
            };

            if trace_fetch_hooks {
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

                if (trace_options.undecoded
                    || (trace_options.el0_undecoded && cpu.pstate.el() == 0))
                    && self.trace.counters.undecoded < 512
                {
                    let raw = self.bus.mem.read(pa, 4).unwrap_or(0) as u32;
                    if decode(raw).is_none() {
                        eprintln!(
                            "UNDECODED step={} core={} el={} pc=0x{pc:016x} pa=0x{pa:016x} raw=0x{raw:08x}",
                            self.total_steps,
                            core,
                            cpu.pstate.el(),
                        );
                        self.trace.counters.undecoded += 1;
                    }
                }
            }

            let instr = cache.fetch(&self.bus.mem, pa);

            if let Some(instr) = instr {
                if trace_instruction_hooks {
                    if trace_options.syscall_dispatch
                        && (0xffff_8000_8002_8800..=0xffff_8000_8002_8a00).contains(&pc)
                    {
                        eprintln!(
                            "DISPATCH step={} pc=0x{pc:016x} instr={instr:?} \
                             x0=0x{:016x} x1=0x{:016x} x2=0x{:016x} x3=0x{:016x} \
                             x8=0x{:016x} x16=0x{:016x} x17=0x{:016x} \
                             x19=0x{:016x} x20=0x{:016x} x21=0x{:016x} pstate=0x{:x}",
                            self.total_steps,
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
                        && let Some(syscall) =
                            trace_syscall_path_entry(cpu, &self.bus, pc, self.total_steps)
                    {
                        self.trace.pending_syscalls[core] = Some(syscall);
                        self.trace.counters.syscall_path += 1;
                    }
                    if trace_options.exec
                        && cpu.pstate.el() == 0
                        && instr.op == Opcode::Svc
                        && self.trace.counters.exec < 1024
                        && self.trace.pending_syscalls[core].is_none()
                        && let Some(syscall) =
                            trace_exec_entry(cpu, &self.bus, pc, self.total_steps)
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
                        let trace_bpf_range =
                            (0xffff_8000_8004_66b0..=0xffff_8000_8004_6b40).contains(&pc);
                        let trace_bpf_cache_flush = (0xffff_8000_8003_6e40..=0xffff_8000_8003_6ec0)
                            .contains(&pc)
                            && (0xffff_8000_8004_66b0..=0xffff_8000_8004_6b40)
                                .contains(&cpu.regs.x(30));
                        if trace_bpf_range || trace_bpf_cache_flush {
                            eprintln!(
                                "BPF step={} pc=0x{pc:016x} instr={instr:?} \
                                 x0=0x{:016x} x1=0x{:016x} x2=0x{:016x} x3=0x{:016x} \
                                 x4=0x{:016x} x5=0x{:016x} x19=0x{:016x} x20=0x{:016x} \
                                 x21=0x{:016x} x22=0x{:016x} x23=0x{:016x} lr=0x{:016x} \
                                 sp=0x{:016x} pstate=0x{:x}",
                                self.total_steps,
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
                    }
                    if trace_options.mprotect_loop
                        && (0xffff_8000_8037_e840..=0xffff_8000_8037_e864).contains(&pc)
                    {
                        trace_mprotect_loop_state(cpu, pc, pa, instr, self.total_steps);
                    }
                }

                if handle_gic_sysreg_access(cpu, &mut self.bus, instr) {
                    self.total_steps += 1;
                    self.active_core = (core + 1) % num_cores;
                    continue;
                }

                if is_fp_simd_access(instr) && fp_simd_access_traps(cpu) {
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
                    self.total_steps += 1;
                    self.active_core = (core + 1) % num_cores;
                    continue;
                }

                if let Err(err) = execute(cpu, &mut self.bus, instr) {
                    if trace_options.el0_fault_raw
                        && cpu.pstate.el() == 0
                        && is_data_abort_fault(err)
                    {
                        let is_main_exec_fault =
                            (0x0000_aaaa_0000_0000..=0x0000_aaab_ffff_ffff).contains(&pc);
                        if self.trace.counters.el0_fault_raw < 512 || is_main_exec_fault {
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
                                let Some(window_pa) =
                                    pa.checked_add_signed(offset * INSTRUCTION_SIZE as i64)
                                else {
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
                    if trace_options.faults && self.exec_faults < 64 {
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
                    self.exec_faults += 1;
                    if is_data_abort_fault(err) {
                        take_data_abort(cpu, pc, instr, err, trace_options.el0_faults);
                    } else {
                        cpu.regs.pc += INSTRUCTION_SIZE;
                    }
                    self.total_steps += 1;
                    self.active_core = (core + 1) % num_cores;
                    continue;
                }
                if trace_syscall_returns
                    && instr.op == Opcode::Eret
                    && cpu.pstate.el() == 0
                    && let Some(syscall) = self.trace.pending_syscalls[core].take()
                {
                    trace_syscall_path_return(cpu, &self.bus, syscall);
                }
                deliver_external_irq(cpu, &mut self.bus);
            } else {
                // Decode failed — skip the bad instruction
                cpu.regs.pc += INSTRUCTION_SIZE;
            }

            self.total_steps += 1;
            self.active_core = (core + 1) % num_cores;
        }

        (self.total_steps - start_steps) as usize
    }

    pub fn core(&self, n: usize) -> &Armv8Cpu {
        &self.cpus[n]
    }
    pub fn core_mut(&mut self, n: usize) -> &mut Armv8Cpu {
        &mut self.cpus[n]
    }

    pub fn inject_irq(&mut self, int_id: u32) {
        self.bus.gic.set_pending(int_id);
    }
}

fn deliver_external_irq(cpu: &mut Armv8Cpu, bus: &mut SystemBus) {
    if cpu.sys.vbar_el1 == 0 || cpu.sys.irq_pending || cpu.pstate.irq_masked() {
        return;
    }

    let Some(int_id) = bus.gic.next_pending_enabled() else {
        return;
    };

    cpu.sys.irq_pending = true;
    cpu.sys.last_irq_id = int_id;
    cpu.clear_exclusive();
    let from_lower_el = cpu.pstate.el() == 0;
    cpu.sys.spsr_el1 = cpu.pstate.to_u64();
    cpu.sys.elr_el1 = cpu.regs.pc;
    cpu.sys.esr_el1 = 0;
    cpu.enter_el1_exception(from_lower_el);
    cpu.regs.pc = cpu.sys.vbar_el1
        + if from_lower_el {
            VBAR_IRQ_LOWER_EL_AARCH64
        } else {
            VBAR_IRQ_CURRENT_EL
        };
}

fn trace_mprotect_loop_state(cpu: &Armv8Cpu, pc: u64, pa: u64, instr: Instr, step: u64) {
    let branch_taken = if instr.op == Opcode::BCond {
        Some(cond_taken(cpu, instr.cond))
    } else {
        None
    };
    let branch_target = if instr.op == Opcode::BCond {
        (pc as i64).wrapping_add(instr.imm as i64) as u64
    } else {
        0
    };
    eprintln!(
        "MPROT step={step} pc=0x{pc:016x} pa=0x{pa:016x} instr={instr:?} \
         x0=0x{:016x} x1=0x{:016x} x2=0x{:016x} x19=0x{:016x} \
         x22=0x{:016x} x23=0x{:016x} x24=0x{:016x} x27=0x{:016x} \
         sp=0x{:016x} lr=0x{:016x} nzcv=N{}Z{}C{}V{} pstate=0x{:x} \
         taken={branch_taken:?} target=0x{branch_target:016x}",
        cpu.regs.x(0),
        cpu.regs.x(1),
        cpu.regs.x(2),
        cpu.regs.x(19),
        cpu.regs.x(22),
        cpu.regs.x(23),
        cpu.regs.x(24),
        cpu.regs.x(27),
        cpu.regs.sp,
        cpu.regs.x(30),
        u8::from(cpu.pstate.n()),
        u8::from(cpu.pstate.z()),
        u8::from(cpu.pstate.c()),
        u8::from(cpu.pstate.v()),
        cpu.pstate.to_u64(),
    );
}

fn trace_rwsem_loop(
    cpu: &mut Armv8Cpu,
    bus: &SystemBus,
    pc: u64,
    pa: u64,
    instr: Instr,
    step: u64,
) {
    let x24 = cpu.regs.x(24);
    let x26 = cpu.regs.x(26);
    let mem24 = trace_read_u64(cpu, bus, x24);
    let mem26 = trace_read_u64(cpu, bus, x26);
    let mem26_owner = trace_read_u64(cpu, bus, x26.wrapping_add(8));
    let owner_task = mem26_owner.unwrap_or(0) & !0x3;
    let owner_on_cpu = trace_read_u32(cpu, bus, owner_task.wrapping_add(0x34));
    let mem26_wait_next = trace_read_u64(cpu, bus, x26.wrapping_add(16));
    let mem26_wait_prev = trace_read_u64(cpu, bus, x26.wrapping_add(24));
    eprintln!(
        "RWSEM step={step} pc=0x{pc:016x} pa=0x{pa:016x} instr={instr:?} \
         x0=0x{:016x} x1=0x{:016x} x2=0x{:016x} x3=0x{:016x} x4=0x{:016x} x5=0x{:016x} \
         x22=0x{:016x} x24=0x{x24:016x} mem24={mem24:?} x26=0x{x26:016x} mem26={mem26:?} \
         owner={mem26_owner:?} owner_on_cpu={owner_on_cpu:?} wait_next={mem26_wait_next:?} wait_prev={mem26_wait_prev:?} \
         x28=0x{:016x} sp=0x{:016x} pstate=0x{:x} timer_delta={}",
        cpu.regs.x(0),
        cpu.regs.x(1),
        cpu.regs.x(2),
        cpu.regs.x(3),
        cpu.regs.x(4),
        cpu.regs.x(5),
        cpu.regs.x(22),
        cpu.regs.x(28),
        cpu.regs.sp,
        cpu.pstate.to_u64(),
        cpu.sys.cntv_cval_el0.wrapping_sub(cpu.sys.cycle_count),
    );
}

fn trace_read_u64(cpu: &mut Armv8Cpu, bus: &SystemBus, va: u64) -> Option<u64> {
    let mut tlb = cpu.tlb.clone();
    translate(&cpu.sys, &mut tlb, &bus.mem, va)
        .ok()
        .and_then(|pa| bus.mem.read(pa, 8))
}

fn trace_read_u32(cpu: &mut Armv8Cpu, bus: &SystemBus, va: u64) -> Option<u64> {
    let mut tlb = cpu.tlb.clone();
    translate(&cpu.sys, &mut tlb, &bus.mem, va)
        .ok()
        .and_then(|pa| bus.mem.read(pa, 4))
}

fn is_stack_chk_signature(bus: &SystemBus, pa: u64) -> bool {
    [
        bus.mem.read(pa, 4),
        bus.mem.read(pa + 4, 4),
        bus.mem.read(pa + 8, 4),
        bus.mem.read(pa + 12, 4),
        bus.mem.read(pa + 16, 4),
    ] == [
        Some(0xd503_233f),
        Some(0xa9bf_7bfd),
        Some(0xf000_0340),
        Some(0x912c_6000),
        Some(0x9100_03fd),
    ]
}

fn trace_stack_chk_enter(cpu: &mut Armv8Cpu, bus: &SystemBus, pc: u64, pa: u64, step: u64) {
    if !is_stack_chk_signature(bus, pa) {
        return;
    }

    eprintln!(
        "STACK_CHK_ENTER step={step} pc=0x{pc:016x} pa=0x{pa:016x} \
         sp=0x{:016x} x29=0x{:016x} lr=0x{:016x} x0=0x{:016x} x1=0x{:016x} x2=0x{:016x} x3=0x{:016x}",
        cpu.regs.sp,
        cpu.regs.x(29),
        cpu.regs.x(30),
        cpu.regs.x(0),
        cpu.regs.x(1),
        cpu.regs.x(2),
        cpu.regs.x(3),
    );
}

fn trace_stack_chk_call(cpu: &mut Armv8Cpu, bus: &SystemBus, pc: u64, instr: Instr, step: u64) {
    let target = (pc as i64).wrapping_add(instr.imm as i64) as u64;
    let mut tlb = cpu.tlb.clone();
    let Some(pa) = translate(&cpu.sys, &mut tlb, &bus.mem, target).ok() else {
        return;
    };
    if !is_stack_chk_signature(bus, pa) {
        return;
    }

    eprintln!(
        "STACK_CHK_CALL step={step} pc=0x{pc:016x} target=0x{target:016x} \
         sp=0x{:016x} x29=0x{:016x} lr=0x{:016x} x0=0x{:016x} x1=0x{:016x} x2=0x{:016x} x3=0x{:016x}",
        cpu.regs.sp,
        cpu.regs.x(29),
        cpu.regs.x(30),
        cpu.regs.x(0),
        cpu.regs.x(1),
        cpu.regs.x(2),
        cpu.regs.x(3),
    );
}

fn trace_writev(cpu: &mut Armv8Cpu, bus: &SystemBus) {
    let fd = cpu.regs.x(0);
    let iov = cpu.regs.x(1);
    let iovcnt = cpu.regs.x(2).min(32);
    let mut text = String::new();

    for idx in 0..iovcnt {
        let Some(base) = trace_read_u64(cpu, bus, iov + idx * 16) else {
            continue;
        };
        let Some(len) = trace_read_u64(cpu, bus, iov + idx * 16 + 8) else {
            continue;
        };
        let len = len.min(4096);
        for off in 0..len {
            let Some(byte) = trace_read_u8(cpu, bus, base + off) else {
                break;
            };
            let ch = byte as u8;
            if ch.is_ascii_graphic() || matches!(ch, b'\n' | b'\r' | b'\t' | b' ') {
                text.push(ch as char);
            } else {
                text.push('.');
            }
        }
    }

    eprintln!("WRITEV fd={fd} iov=0x{iov:016x} iovcnt={iovcnt} text={text:?}");
}

fn trace_syscall_path_entry(
    cpu: &mut Armv8Cpu,
    bus: &SystemBus,
    pc: u64,
    step: u64,
) -> Option<TraceSyscall> {
    let nr = cpu.regs.x(8);
    let args = [
        cpu.regs.x(0),
        cpu.regs.x(1),
        cpu.regs.x(2),
        cpu.regs.x(3),
        cpu.regs.x(4),
        cpu.regs.x(5),
    ];

    match nr {
        56 => {
            let path = trace_read_c_string(cpu, bus, args[1], 256);
            eprintln!(
                "SYSCALL enter step={step} pc=0x{pc:016x} openat dfd={} path={} flags=0x{:x} mode=0o{:o}",
                args[0] as i64,
                format_trace_string(path.as_deref()),
                args[2],
                args[3],
            );
        }
        78 => {
            let path = trace_read_c_string(cpu, bus, args[1], 256);
            eprintln!(
                "SYSCALL enter step={step} pc=0x{pc:016x} readlinkat dfd={} path={} buf=0x{:016x} bufsiz={}",
                args[0] as i64,
                format_trace_string(path.as_deref()),
                args[2],
                args[3],
            );
        }
        79 => {
            let path = trace_read_c_string(cpu, bus, args[1], 256);
            eprintln!(
                "SYSCALL enter step={step} pc=0x{pc:016x} newfstatat dfd={} path={} statbuf=0x{:016x} flags=0x{:x}",
                args[0] as i64,
                format_trace_string(path.as_deref()),
                args[2],
                args[3],
            );
        }
        291 => {
            let path = trace_read_c_string(cpu, bus, args[1], 256);
            eprintln!(
                "SYSCALL enter step={step} pc=0x{pc:016x} statx dfd={} path={} flags=0x{:x} mask=0x{:x} statxbuf=0x{:016x}",
                args[0] as i64,
                format_trace_string(path.as_deref()),
                args[2],
                args[3],
                args[4],
            );
        }
        _ => return None,
    }

    Some(TraceSyscall { nr, args, pc, step })
}

fn trace_exec_entry(
    cpu: &mut Armv8Cpu,
    bus: &SystemBus,
    pc: u64,
    step: u64,
) -> Option<TraceSyscall> {
    let nr = cpu.regs.x(8);
    let args = [
        cpu.regs.x(0),
        cpu.regs.x(1),
        cpu.regs.x(2),
        cpu.regs.x(3),
        cpu.regs.x(4),
        cpu.regs.x(5),
    ];

    match nr {
        221 => {
            let path = trace_read_c_string(cpu, bus, args[0], 512);
            let argv = trace_read_argv(cpu, bus, args[1], 8);
            eprintln!(
                "EXEC enter step={step} pc=0x{pc:016x} execve path={} argv=[{}] envp=0x{:016x}",
                format_trace_string(path.as_deref()),
                argv.join(", "),
                args[2],
            );
        }
        281 => {
            let path = trace_read_c_string(cpu, bus, args[1], 512);
            let argv = trace_read_argv(cpu, bus, args[2], 8);
            eprintln!(
                "EXEC enter step={step} pc=0x{pc:016x} execveat dfd={} path={} argv=[{}] envp=0x{:016x} flags=0x{:x}",
                args[0] as i64,
                format_trace_string(path.as_deref()),
                argv.join(", "),
                args[3],
                args[4],
            );
        }
        _ => return None,
    }

    Some(TraceSyscall { nr, args, pc, step })
}

fn trace_syscall_path_return(cpu: &mut Armv8Cpu, bus: &SystemBus, syscall: TraceSyscall) {
    let ret = cpu.regs.x(0);
    let ret_signed = ret as i64;
    let ret_text = if (-4095..0).contains(&ret_signed) {
        format!("{ret_signed}")
    } else {
        format!("{} / 0x{ret:016x}", ret_signed)
    };

    eprintln!(
        "SYSCALL return step={} pc=0x{:016x} nr={} ret={}",
        syscall.step, syscall.pc, syscall.nr, ret_text
    );

    if syscall.nr == 291 && ret_signed >= 0 {
        trace_statx(cpu, bus, syscall.args[4]);
    }
}

fn trace_read_argv(cpu: &mut Armv8Cpu, bus: &SystemBus, argv: u64, max: usize) -> Vec<String> {
    if argv == 0 {
        return Vec::new();
    }

    let mut values = Vec::new();
    for idx in 0..max {
        let Some(ptr) = trace_read_u64(cpu, bus, argv + (idx as u64) * 8) else {
            values.push("<unreadable>".to_string());
            break;
        };
        if ptr == 0 {
            break;
        }
        let pa = trace_translate(cpu, bus, ptr)
            .map(|pa| format!("0x{pa:016x}"))
            .unwrap_or_else(|| "<unmapped>".to_string());
        values.push(format!(
            "0x{ptr:016x}/{pa}={}",
            format_trace_string(trace_read_c_string(cpu, bus, ptr, 512).as_deref())
        ));
    }
    values
}

fn trace_statx(cpu: &mut Armv8Cpu, bus: &SystemBus, sx: u64) {
    let mask = trace_read_u32(cpu, bus, sx).unwrap_or(0);
    let mode = trace_read_u32(cpu, bus, sx + 28).unwrap_or(0) & 0xffff;
    let ino = trace_read_u64(cpu, bus, sx + 32).unwrap_or(0);
    let dev_major = trace_read_u32(cpu, bus, sx + 136).unwrap_or(0);
    let dev_minor = trace_read_u32(cpu, bus, sx + 140).unwrap_or(0);
    let mnt_id = trace_read_u64(cpu, bus, sx + 144).unwrap_or(0);
    eprintln!(
        "  statx buf=0x{sx:016x} mask=0x{mask:x} mode=0o{mode:o} ino={ino} dev={dev_major}:{dev_minor} mnt_id={mnt_id}"
    );
}

fn trace_chase_assert_check(
    cpu: &mut Armv8Cpu,
    bus: &SystemBus,
    pc: u64,
    pa: u64,
    step: u64,
) -> bool {
    let signature = [
        Some(0xb400_0093), // cbz x19, assert
        Some(0x3940_0260), // ldrb w0, [x19]
        Some(0x7100_bc1f), // cmp w0, #'/'
        Some(0x54ff_f3c0), // b.eq ok
    ];
    if [
        bus.mem.read(pa, 4).map(|raw| raw as u32),
        bus.mem.read(pa + 4, 4).map(|raw| raw as u32),
        bus.mem.read(pa + 8, 4).map(|raw| raw as u32),
        bus.mem.read(pa + 12, 4).map(|raw| raw as u32),
    ] != signature
    {
        return false;
    }

    let p = cpu.regs.x(19);
    let first = if p == 0 {
        None
    } else {
        trace_read_u8(cpu, bus, p).map(|byte| byte as u8)
    };
    if first == Some(b'/') {
        return false;
    }

    let p_text = trace_read_c_string(cpu, bus, p, 512);
    let x20_text = trace_read_c_string(cpu, bus, cpu.regs.x(20), 256);
    let x21_text = trace_read_c_string(cpu, bus, cpu.regs.x(21), 256);
    let x26_text = trace_read_c_string(cpu, bus, cpu.regs.x(26), 256);
    eprintln!(
        "CHASE_ASSERT_FAIL step={step} pc=0x{pc:016x} pa=0x{pa:016x} \
         p=x19=0x{p:016x} first={first:?} p_text={} \
         x20=0x{:016x} x20_text={} x21=0x{:016x} x21_text={} \
         x22=0x{:016x} x23=0x{:016x} x24=0x{:016x} x26=0x{:016x} x26_text={}",
        format_trace_string(p_text.as_deref()),
        cpu.regs.x(20),
        format_trace_string(x20_text.as_deref()),
        cpu.regs.x(21),
        format_trace_string(x21_text.as_deref()),
        cpu.regs.x(22),
        cpu.regs.x(23),
        cpu.regs.x(24),
        cpu.regs.x(26),
        format_trace_string(x26_text.as_deref()),
    );
    true
}

fn trace_path_extend_strlen(
    cpu: &mut Armv8Cpu,
    bus: &SystemBus,
    pc: u64,
    pa: u64,
    step: u64,
) -> bool {
    let signature = [
        Some(0xaa00_03f3), // mov x19, x0
        Some(0xaa13_03f7), // mov x23, x19
        Some(0x9103_c3f6), // add x22, sp, #0xf0
    ];
    if [
        bus.mem.read(pa, 4).map(|raw| raw as u32),
        bus.mem.read(pa + 4, 4).map(|raw| raw as u32),
        bus.mem.read(pa + 8, 4).map(|raw| raw as u32),
    ] != signature
    {
        return false;
    }

    let slot = cpu.regs.x(20);
    let Some(old_ptr) = trace_read_u64(cpu, bus, slot) else {
        return false;
    };
    if old_ptr == 0 {
        return false;
    }

    let old = trace_read_c_string(cpu, bus, old_ptr, 256);
    let reported_len = cpu.regs.x(0);
    let suspicious = old.as_deref().is_some_and(|s| {
        s.starts_with('/') && reported_len != s.len() as u64
            || s == "sys" && reported_len != 3
            || s == "devices" && reported_len != 7
            || s == "virtual" && reported_len != 7
    });
    if !suspicious {
        return false;
    }

    eprintln!(
        "PATH_EXTEND_STRLEN step={step} pc=0x{pc:016x} pa=0x{pa:016x} \
         slot=0x{slot:016x} old_ptr=0x{old_ptr:016x} old={} reported_len={reported_len}",
        format_trace_string(old.as_deref()),
    );
    true
}

fn trace_read_c_string(
    cpu: &mut Armv8Cpu,
    bus: &SystemBus,
    va: u64,
    max_len: u64,
) -> Option<String> {
    if va == 0 {
        return None;
    }

    let mut s = String::new();
    for offset in 0..max_len {
        let byte = trace_read_u8(cpu, bus, va + offset)? as u8;
        if byte == 0 {
            return Some(s);
        }
        if byte.is_ascii_graphic() || matches!(byte, b'\n' | b'\r' | b'\t' | b' ') {
            s.push(byte as char);
        } else {
            s.push('.');
        }
    }
    s.push_str("...");
    Some(s)
}

fn format_trace_string(value: Option<&str>) -> String {
    match value {
        Some(value) => format!("{value:?}"),
        None => "<unreadable>".to_string(),
    }
}

fn trace_read_u8(cpu: &mut Armv8Cpu, bus: &SystemBus, va: u64) -> Option<u64> {
    trace_translate(cpu, bus, va).and_then(|pa| bus.mem.read(pa, 1))
}

fn trace_translate(cpu: &Armv8Cpu, bus: &SystemBus, va: u64) -> Option<u64> {
    let mut tlb = cpu.tlb.clone();
    translate(&cpu.sys, &mut tlb, &bus.mem, va).ok()
}

fn take_instruction_abort(cpu: &mut Armv8Cpu, fault_pc: u64) {
    let from_lower_el = cpu.pstate.el() == 0;
    let ec = if from_lower_el {
        ESR_EC_INSN_ABORT_LOWER_EL
    } else {
        ESR_EC_INSN_ABORT_CURRENT_EL
    };
    take_sync_exception(cpu, fault_pc, ec, ESR_FSC_TRANSLATION_LEVEL3, from_lower_el);
}

fn take_data_abort(
    cpu: &mut Armv8Cpu,
    fault_pc: u64,
    instr: Instr,
    err: &str,
    trace_el0_faults: bool,
) {
    let from_lower_el = cpu.pstate.el() == 0;
    let ec = if from_lower_el {
        ESR_EC_DATA_ABORT_LOWER_EL
    } else {
        ESR_EC_DATA_ABORT_CURRENT_EL
    };
    let fsc = if err.contains("permission fault") {
        ESR_FSC_PERMISSION_LEVEL3
    } else if err.contains("access flag fault") {
        ESR_FSC_ACCESS_FLAG_LEVEL3
    } else {
        ESR_FSC_TRANSLATION_LEVEL3
    };
    let iss = fsc
        | if memory_fault_is_write(instr) {
            ESR_DATA_ABORT_WNR
        } else {
            0
        };
    if from_lower_el && trace_el0_faults {
        eprintln!(
            "EL0 DATA ABORT pc=0x{fault_pc:016x} instr={instr:?} far=0x{:016x} iss=0x{iss:x} \
             x0=0x{:016x} x1=0x{:016x} x2=0x{:016x} x3=0x{:016x} \
             x8=0x{:016x} sp=0x{:016x} sp_el0=0x{:016x} sp_el1=0x{:016x} elr=0x{:016x} spsr=0x{:x}",
            cpu.sys.far_el1,
            cpu.regs.x(0),
            cpu.regs.x(1),
            cpu.regs.x(2),
            cpu.regs.x(3),
            cpu.regs.x(8),
            cpu.regs.sp,
            cpu.sys.sp_el0,
            cpu.sys.sp_el1,
            cpu.sys.elr_el1,
            cpu.sys.spsr_el1,
        );
    }
    take_sync_exception(cpu, fault_pc, ec, iss, from_lower_el);
}

fn take_fp_simd_trap(cpu: &mut Armv8Cpu, fault_pc: u64) {
    let from_lower_el = cpu.pstate.el() == 0;
    take_sync_exception(
        cpu,
        fault_pc,
        ESR_EC_FP_ASIMD,
        ESR_FP_ASIMD_ISS_AARCH64,
        from_lower_el,
    );
}

fn is_data_abort_fault(err: &str) -> bool {
    err.contains("translation fault")
        || err.contains("permission fault")
        || err.contains("access flag fault")
}

fn take_sync_exception(cpu: &mut Armv8Cpu, fault_pc: u64, ec: u64, iss: u64, from_lower_el: bool) {
    cpu.clear_exclusive();
    cpu.sys.elr_el1 = fault_pc;
    cpu.sys.spsr_el1 = cpu.pstate.to_u64();
    cpu.sys.esr_el1 = (ec << 26) | iss;

    cpu.enter_el1_exception(from_lower_el);
    let vector = if from_lower_el {
        VBAR_SYNC_LOWER_EL_AARCH64
    } else {
        VBAR_SYNC_CURRENT_EL
    };
    cpu.regs.pc = cpu.sys.vbar_el1 + vector;
}

fn memory_fault_is_write(instr: Instr) -> bool {
    matches!(
        instr.op,
        Opcode::Str
            | Opcode::Stp
            | Opcode::SimdStr
            | Opcode::SimdStp
            | Opcode::SimdSt1Multi
            | Opcode::SimdSt1Lane
            | Opcode::SimdSt4Single
            | Opcode::SimdSt4
            | Opcode::Stxr
            | Opcode::Stlr
            | Opcode::Stxp
            | Opcode::Atomic
            | Opcode::Cas
            | Opcode::Casp
            | Opcode::DcZva
    )
}

fn fp_simd_access_traps(cpu: &Armv8Cpu) -> bool {
    let fpen = (cpu.sys.cpacr_el1 & CPACR_FPEN_MASK) >> CPACR_FPEN_SHIFT;
    match cpu.pstate.el() {
        0 => fpen != CPACR_FPEN_TRAP_NONE,
        1 => matches!(fpen, CPACR_FPEN_TRAP_EL0_EL1 | CPACR_FPEN_TRAP_EL1_EL0),
        _ => false,
    }
}

fn is_fp_simd_access(instr: Instr) -> bool {
    match instr.op {
        Opcode::Mrs | Opcode::Msr => {
            let sysreg_id = instr.imm as u16;
            matches!(sysreg_id, SYSREG_FPCR | SYSREG_FPSR)
        }
        op => is_fp_simd_opcode(op),
    }
}

fn is_fp_simd_opcode(op: Opcode) -> bool {
    matches!(
        op,
        Opcode::SimdLdp
            | Opcode::SimdStp
            | Opcode::SimdLdr
            | Opcode::SimdStr
            | Opcode::SimdMovi
            | Opcode::SimdLd1
            | Opcode::SimdLd1Multi
            | Opcode::SimdLd1Lane
            | Opcode::SimdLd1r
            | Opcode::SimdLd2
            | Opcode::SimdLd3
            | Opcode::SimdSt1Multi
            | Opcode::SimdSt1Lane
            | Opcode::SimdLd4
            | Opcode::SimdSt4Single
            | Opcode::SimdSt4
            | Opcode::SimdAese
            | Opcode::SimdAesd
            | Opcode::SimdAesmc
            | Opcode::SimdAesimc
            | Opcode::SimdEor3
            | Opcode::SimdBcax
            | Opcode::SimdRax1
            | Opcode::SimdXar
            | Opcode::SimdDupByte
            | Opcode::SimdDupElem
            | Opcode::SimdFmovReg64
            | Opcode::SimdFmovGprToD
            | Opcode::SimdFmovGprToS
            | Opcode::SimdFmovDToGpr
            | Opcode::SimdFmovSToGpr
            | Opcode::SimdFmovLaneToGpr
            | Opcode::SimdUmov
            | Opcode::SimdInsGprLane
            | Opcode::SimdCmeqZero
            | Opcode::SimdCmeqReg
            | Opcode::SimdCmhsReg
            | Opcode::SimdShrn
            | Opcode::SimdAddhn
            | Opcode::SimdAddVec
            | Opcode::SimdSubVec
            | Opcode::SimdMulVec
            | Opcode::SimdAddp
            | Opcode::SimdAddv
            | Opcode::SimdExt
            | Opcode::SimdUmaxp
            | Opcode::SimdUminp
            | Opcode::SimdCnt
            | Opcode::SimdCmtst
            | Opcode::SimdShlImm
            | Opcode::SimdSli
            | Opcode::SimdSri
            | Opcode::SimdSshr
            | Opcode::SimdUshr
            | Opcode::SimdUshl
            | Opcode::SimdXtn
            | Opcode::SimdRev64
            | Opcode::SimdRev32
            | Opcode::SimdNot
            | Opcode::SimdBsl
            | Opcode::SimdBit
            | Opcode::SimdBif
            | Opcode::SimdAnd
            | Opcode::SimdBic
            | Opcode::SimdOrr
            | Opcode::SimdEor
            | Opcode::SimdInsElem
            | Opcode::SimdUzp1
            | Opcode::SimdTrn1
            | Opcode::SimdZip1
            | Opcode::SimdZip2
            | Opcode::SimdTbl
            | Opcode::SimdBicImm
            | Opcode::SimdMvni
            | Opcode::SimdUshll
            | Opcode::SimdSshll
            | Opcode::SimdShll
            | Opcode::SimdSsubw
            | Opcode::SimdUmlal
            | Opcode::SimdUqsub
            | Opcode::SimdFcvtzu
            | Opcode::SimdFpNeg
            | Opcode::FpAdd
            | Opcode::FpSub
            | Opcode::FpMul
            | Opcode::FpDiv
            | Opcode::FpNeg
            | Opcode::FpAbs
            | Opcode::FpSqrt
            | Opcode::FpFcvt
            | Opcode::FpFrintm
            | Opcode::FpFrintn
            | Opcode::FpFrinta
            | Opcode::FpFrintx
            | Opcode::FpFrintz
            | Opcode::FpMovImm
            | Opcode::Fmadd
            | Opcode::Fmsub
            | Opcode::Fnmsub
            | Opcode::Scvtf
            | Opcode::Ucvtf
            | Opcode::Fcvtzs
            | Opcode::Fcvtzu
            | Opcode::Fcvtas
            | Opcode::Fcmp
            | Opcode::Fcmpe
            | Opcode::Fccmp
            | Opcode::Fccmpe
            | Opcode::Fcsel
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_store_translation_fault_enters_data_abort_vector() {
        let mut machine = Machine::new(1);
        let pc_va = KERNEL_VA_BASE;
        let pc_pa = 0x4000_0000;
        let data_va = 0x0000_AAAA_DAD7_03F8;

        machine.bus.mem.write(pc_pa, 4, 0xF900_001F); // str xzr, [x0]
        machine
            .bus
            .mem
            .write(0x1000 + 256 * 8, 8, (0x2000 & DESC_ADDR_MASK) | DESC_VALID);
        machine
            .bus
            .mem
            .write(0x2000, 8, (0x3000 & DESC_ADDR_MASK) | DESC_VALID);
        machine
            .bus
            .mem
            .write(0x3000, 8, (0x4000 & DESC_ADDR_MASK) | DESC_VALID);
        machine
            .bus
            .mem
            .write(0x4000, 8, (pc_pa & DESC_ADDR_MASK) | DESC_VALID);

        let cpu = machine.core_mut(0);
        cpu.regs.pc = pc_va;
        cpu.regs.set_x(0, data_va);
        cpu.pstate = cpu.pstate.with_el(1);
        cpu.sys.vbar_el1 = KERNEL_VA_BASE + 0x1000;
        cpu.sys.ttbr0_el1 = 0x5000;
        cpu.sys.ttbr1_el1 = 0x1000;
        cpu.sys.tcr_el1 = (16 << TCR_T1SZ_SHIFT) | 16;
        cpu.sys.sctlr_el1 = SCTLR_MMU_ENABLE;

        machine.run(1);

        let cpu = machine.core(0);
        assert_eq!(cpu.regs.pc, cpu.sys.vbar_el1 + VBAR_SYNC_CURRENT_EL);
        assert_eq!(cpu.sys.far_el1, data_va);
        assert_eq!(cpu.sys.elr_el1, pc_va);
        assert_eq!(cpu.sys.esr_el1 >> 26, ESR_EC_DATA_ABORT_CURRENT_EL);
        assert_ne!(cpu.sys.esr_el1 & ESR_DATA_ABORT_WNR, 0);
    }
}
