use super::*;

pub(crate) fn trace_writev(cpu: &mut Armv8Cpu, bus: &SystemBus) {
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
        append_iovec_text(cpu, bus, base, len, &mut text);
    }

    eprintln!("WRITEV fd={fd} iov=0x{iov:016x} iovcnt={iovcnt} text={text:?}");
}

fn append_iovec_text(cpu: &mut Armv8Cpu, bus: &SystemBus, base: u64, len: u64, text: &mut String) {
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
