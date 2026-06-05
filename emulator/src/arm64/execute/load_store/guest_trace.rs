use super::*;
use std::sync::OnceLock;

pub(in crate::arm64::execute) fn trace_syscall_frame_access(
    cpu: &mut Armv8Cpu,
    instr: &Instr,
    kind: &str,
    va: u64,
    pa: u64,
    size: u8,
    value: Option<u64>,
) {
    if !trace_syscall_frame_enabled()
        || cpu.pstate.el() != 1
        || cpu.trace_syscall_access_budget == 0
        || cpu.trace_syscall_stack_top == 0
    {
        return;
    }

    let stack_top = cpu.trace_syscall_stack_top;
    if !(stack_top.saturating_sub(0x300)..=stack_top).contains(&va) {
        return;
    }
    cpu.trace_syscall_access_budget -= 1;

    eprintln!(
        "FRAME {kind} pc={:#018x} va={:#018x} top_off=-{:#x} sp_off={:#x} pa={:#018x} size={} rd={} rn={} rm={} imm={:#x} base={:#018x} value={:#018x}",
        cpu.regs.pc,
        va,
        stack_top.wrapping_sub(va),
        va.wrapping_sub(cpu.regs.sp),
        pa,
        size,
        instr.rd,
        instr.rn,
        instr.rm,
        instr.imm,
        base_addr(cpu, instr.rn),
        value.unwrap_or(0),
    );
}

pub(in crate::arm64::execute) fn trace_text_store(
    cpu: &Armv8Cpu,
    bus: &mut SystemBus,
    instr: &Instr,
    kind: &str,
    va: u64,
    pa: u64,
    size: u8,
    value: u64,
) {
    let trace_text = trace_text_patch_enabled();
    let trace_store = trace_store_pa();
    if !trace_text && trace_store.is_none() {
        return;
    }
    let start = pa;
    let end = pa.saturating_add(size as u64);
    if let Some(target) = trace_store {
        if target < start || target >= end {
            return;
        }
    } else if end <= 0x4003_6e40 || start >= 0x4003_6ec8 {
        return;
    }
    let old = bus.read(pa, size).unwrap_or(0);
    eprintln!(
        "TEXT STORE {kind} pc=0x{:016x} instr={instr:?} va=0x{va:016x} pa=0x{pa:016x} size={size} old=0x{old:016x} new=0x{value:016x} \
         x0=0x{:016x} x1=0x{:016x} x2=0x{:016x} x3=0x{:016x} x4=0x{:016x} x5=0x{:016x} \
         x19=0x{:016x} x20=0x{:016x} x21=0x{:016x} lr=0x{:016x} sp=0x{:016x}",
        cpu.regs.pc,
        cpu.regs.x(0),
        cpu.regs.x(1),
        cpu.regs.x(2),
        cpu.regs.x(3),
        cpu.regs.x(4),
        cpu.regs.x(5),
        cpu.regs.x(19),
        cpu.regs.x(20),
        cpu.regs.x(21),
        cpu.regs.x(30),
        cpu.regs.sp,
    );
}

fn trace_syscall_frame_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| env::var_os("WEBBOXVM_TRACE_SYSCALL_FRAME").is_some())
}

fn trace_text_patch_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| env::var_os("WEBBOXVM_TRACE_TEXT_PATCH").is_some())
}

fn trace_store_pa() -> Option<u64> {
    static TARGET: OnceLock<Option<u64>> = OnceLock::new();
    *TARGET.get_or_init(|| {
        env::var_os("WEBBOXVM_TRACE_STORE_PA").and_then(|target| {
            let target = target.to_string_lossy();
            u64::from_str_radix(target.trim_start_matches("0x"), 16).ok()
        })
    })
}
