use super::*;

pub(crate) fn trace_execve(
    cpu: &mut Armv8Cpu,
    bus: &SystemBus,
    pc: u64,
    step: u64,
    args: [u64; 6],
) -> bool {
    let path = trace_read_c_string(cpu, bus, args[0], 512);
    let argv = trace_read_argv(cpu, bus, args[1], 8);
    trace_exec_line(format!(
        "EXEC enter step={step} pc=0x{pc:016x} execve path={} argv=[{}] envp=0x{:016x}",
        format_trace_string(path.as_deref()),
        argv.join(", "),
        args[2],
    ))
}

pub(crate) fn trace_execveat(
    cpu: &mut Armv8Cpu,
    bus: &SystemBus,
    pc: u64,
    step: u64,
    args: [u64; 6],
) -> bool {
    let path = trace_read_c_string(cpu, bus, args[1], 512);
    let argv = trace_read_argv(cpu, bus, args[2], 8);
    trace_exec_line(format!(
        "EXEC enter step={step} pc=0x{pc:016x} execveat dfd={} path={} argv=[{}] envp=0x{:016x} flags=0x{:x}",
        args[0] as i64,
        format_trace_string(path.as_deref()),
        argv.join(", "),
        args[3],
        args[4],
    ))
}

fn trace_exec_line(line: String) -> bool {
    if !trace_filter_allows("WEBBOXVM_TRACE_EXEC_FILTER", &line) {
        return false;
    }
    eprintln!("{line}");
    true
}
