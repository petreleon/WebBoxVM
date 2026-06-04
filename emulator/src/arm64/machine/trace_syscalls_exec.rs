use super::*;

pub(in crate::arm64::machine) fn trace_execveat(
    cpu: &mut Armv8Cpu,
    bus: &SystemBus,
    pc: u64,
    step: u64,
    args: [u64; 6],
) {
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
