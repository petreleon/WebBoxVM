use super::*;

pub(in crate::arm64::machine) fn trace_chase_assert_check(
    cpu: &mut Armv8Cpu,
    bus: &SystemBus,
    pc: u64,
    pa: u64,
    step: u64,
) -> bool {
    let signature = [
        Some(0xb400_0093),
        Some(0x3940_0260),
        Some(0x7100_bc1f),
        Some(0x54ff_f3c0),
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

pub(in crate::arm64::machine) fn trace_path_extend_strlen(
    cpu: &mut Armv8Cpu,
    bus: &SystemBus,
    pc: u64,
    pa: u64,
    step: u64,
) -> bool {
    let signature = [Some(0xaa00_03f3), Some(0xaa13_03f7), Some(0x9103_c3f6)];
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
