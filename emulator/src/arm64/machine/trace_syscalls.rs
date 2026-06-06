use super::*;

pub(in crate::arm64::machine) fn trace_syscall_path_entry(
    cpu: &mut Armv8Cpu,
    bus: &SystemBus,
    pc: u64,
    step: u64,
) -> Option<TraceSyscall> {
    let nr = cpu.regs.x(8);
    let args = syscall_args(cpu);

    match nr {
        56 if trace_openat(cpu, bus, pc, step, args) => {}
        78 if trace_readlinkat(cpu, bus, pc, step, args) => {}
        79 if trace_newfstatat(cpu, bus, pc, step, args) => {}
        291 if trace_statx_enter(cpu, bus, pc, step, args) => {}
        _ => return None,
    }

    Some(TraceSyscall { nr, args, pc, step })
}

pub(in crate::arm64::machine) fn trace_exec_entry(
    cpu: &mut Armv8Cpu,
    bus: &SystemBus,
    pc: u64,
    step: u64,
) -> Option<TraceSyscall> {
    let nr = cpu.regs.x(8);
    let args = syscall_args(cpu);

    match nr {
        221 if trace_execve(cpu, bus, pc, step, args) => {}
        281 if trace_execveat(cpu, bus, pc, step, args) => {}
        _ => return None,
    }

    Some(TraceSyscall { nr, args, pc, step })
}

pub(in crate::arm64::machine) fn trace_syscall_path_return(
    cpu: &mut Armv8Cpu,
    bus: &SystemBus,
    syscall: TraceSyscall,
) {
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

fn syscall_args(cpu: &Armv8Cpu) -> [u64; 6] {
    [
        cpu.regs.x(0),
        cpu.regs.x(1),
        cpu.regs.x(2),
        cpu.regs.x(3),
        cpu.regs.x(4),
        cpu.regs.x(5),
    ]
}

fn trace_openat(cpu: &mut Armv8Cpu, bus: &SystemBus, pc: u64, step: u64, args: [u64; 6]) -> bool {
    let path = trace_read_c_string(cpu, bus, args[1], 256);
    trace_path_line(format!(
        "SYSCALL enter step={step} pc=0x{pc:016x} openat dfd={} path={} flags=0x{:x} mode=0o{:o}",
        args[0] as i64,
        format_trace_string(path.as_deref()),
        args[2],
        args[3],
    ))
}

fn trace_readlinkat(
    cpu: &mut Armv8Cpu,
    bus: &SystemBus,
    pc: u64,
    step: u64,
    args: [u64; 6],
) -> bool {
    let path = trace_read_c_string(cpu, bus, args[1], 256);
    trace_path_line(format!(
        "SYSCALL enter step={step} pc=0x{pc:016x} readlinkat dfd={} path={} buf=0x{:016x} bufsiz={}",
        args[0] as i64,
        format_trace_string(path.as_deref()),
        args[2],
        args[3],
    ))
}

fn trace_newfstatat(
    cpu: &mut Armv8Cpu,
    bus: &SystemBus,
    pc: u64,
    step: u64,
    args: [u64; 6],
) -> bool {
    let path = trace_read_c_string(cpu, bus, args[1], 256);
    trace_path_line(format!(
        "SYSCALL enter step={step} pc=0x{pc:016x} newfstatat dfd={} path={} statbuf=0x{:016x} flags=0x{:x}",
        args[0] as i64,
        format_trace_string(path.as_deref()),
        args[2],
        args[3],
    ))
}

fn trace_statx_enter(
    cpu: &mut Armv8Cpu,
    bus: &SystemBus,
    pc: u64,
    step: u64,
    args: [u64; 6],
) -> bool {
    let path = trace_read_c_string(cpu, bus, args[1], 256);
    trace_path_line(format!(
        "SYSCALL enter step={step} pc=0x{pc:016x} statx dfd={} path={} flags=0x{:x} mask=0x{:x} statxbuf=0x{:016x}",
        args[0] as i64,
        format_trace_string(path.as_deref()),
        args[2],
        args[3],
        args[4],
    ))
}

fn trace_path_line(line: String) -> bool {
    if !trace_filter_allows("WEBBOXVM_TRACE_PATH_FILTER", &line) {
        return false;
    }
    eprintln!("{line}");
    true
}
