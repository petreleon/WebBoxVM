use super::*;

pub(in crate::arm64::machine) fn trace_writev(cpu: &mut Armv8Cpu, bus: &SystemBus) {
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
        for off in 0..len.min(4096) {
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

pub(in crate::arm64::machine) fn trace_syscall_path_entry(
    cpu: &mut Armv8Cpu,
    bus: &SystemBus,
    pc: u64,
    step: u64,
) -> Option<TraceSyscall> {
    let nr = cpu.regs.x(8);
    let args = syscall_args(cpu);

    match nr {
        56 => trace_openat(cpu, bus, pc, step, args),
        78 => trace_readlinkat(cpu, bus, pc, step, args),
        79 => trace_newfstatat(cpu, bus, pc, step, args),
        291 => trace_statx_enter(cpu, bus, pc, step, args),
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
        221 => trace_execve(cpu, bus, pc, step, args),
        281 => trace_execveat(cpu, bus, pc, step, args),
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

fn trace_openat(cpu: &mut Armv8Cpu, bus: &SystemBus, pc: u64, step: u64, args: [u64; 6]) {
    let path = trace_read_c_string(cpu, bus, args[1], 256);
    eprintln!(
        "SYSCALL enter step={step} pc=0x{pc:016x} openat dfd={} path={} flags=0x{:x} mode=0o{:o}",
        args[0] as i64,
        format_trace_string(path.as_deref()),
        args[2],
        args[3],
    );
}

fn trace_readlinkat(cpu: &mut Armv8Cpu, bus: &SystemBus, pc: u64, step: u64, args: [u64; 6]) {
    let path = trace_read_c_string(cpu, bus, args[1], 256);
    eprintln!(
        "SYSCALL enter step={step} pc=0x{pc:016x} readlinkat dfd={} path={} buf=0x{:016x} bufsiz={}",
        args[0] as i64,
        format_trace_string(path.as_deref()),
        args[2],
        args[3],
    );
}

fn trace_newfstatat(cpu: &mut Armv8Cpu, bus: &SystemBus, pc: u64, step: u64, args: [u64; 6]) {
    let path = trace_read_c_string(cpu, bus, args[1], 256);
    eprintln!(
        "SYSCALL enter step={step} pc=0x{pc:016x} newfstatat dfd={} path={} statbuf=0x{:016x} flags=0x{:x}",
        args[0] as i64,
        format_trace_string(path.as_deref()),
        args[2],
        args[3],
    );
}

fn trace_statx_enter(cpu: &mut Armv8Cpu, bus: &SystemBus, pc: u64, step: u64, args: [u64; 6]) {
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

fn trace_execve(cpu: &mut Armv8Cpu, bus: &SystemBus, pc: u64, step: u64, args: [u64; 6]) {
    let path = trace_read_c_string(cpu, bus, args[0], 512);
    let argv = trace_read_argv(cpu, bus, args[1], 8);
    eprintln!(
        "EXEC enter step={step} pc=0x{pc:016x} execve path={} argv=[{}] envp=0x{:016x}",
        format_trace_string(path.as_deref()),
        argv.join(", "),
        args[2],
    );
}
