use super::*;

pub(in crate::arm64::machine) fn trace_read_u64(
    cpu: &mut Armv8Cpu,
    bus: &SystemBus,
    va: u64,
) -> Option<u64> {
    let mut tlb = cpu.tlb.clone();
    translate(&cpu.sys, &mut tlb, &bus.mem, va)
        .ok()
        .and_then(|pa| bus.mem.read(pa, 8))
}

pub(in crate::arm64::machine) fn trace_read_u32(
    cpu: &mut Armv8Cpu,
    bus: &SystemBus,
    va: u64,
) -> Option<u64> {
    let mut tlb = cpu.tlb.clone();
    translate(&cpu.sys, &mut tlb, &bus.mem, va)
        .ok()
        .and_then(|pa| bus.mem.read(pa, 4))
}

pub(in crate::arm64::machine) fn trace_read_argv(
    cpu: &mut Armv8Cpu,
    bus: &SystemBus,
    argv: u64,
    max: usize,
) -> Vec<String> {
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

pub(in crate::arm64::machine) fn trace_statx(cpu: &mut Armv8Cpu, bus: &SystemBus, sx: u64) {
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

pub(in crate::arm64::machine) fn trace_read_c_string(
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

pub(in crate::arm64::machine) fn format_trace_string(value: Option<&str>) -> String {
    match value {
        Some(value) => format!("{value:?}"),
        None => "<unreadable>".to_string(),
    }
}

pub(in crate::arm64::machine) fn trace_read_u8(
    cpu: &mut Armv8Cpu,
    bus: &SystemBus,
    va: u64,
) -> Option<u64> {
    trace_translate(cpu, bus, va).and_then(|pa| bus.mem.read(pa, 1))
}

pub(in crate::arm64::machine) fn trace_translate(
    cpu: &Armv8Cpu,
    bus: &SystemBus,
    va: u64,
) -> Option<u64> {
    let mut tlb = cpu.tlb.clone();
    translate(&cpu.sys, &mut tlb, &bus.mem, va).ok()
}
