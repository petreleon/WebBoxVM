use super::*;

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

pub(crate) fn trace_stack_chk_enter(
    cpu: &mut Armv8Cpu,
    bus: &SystemBus,
    pc: u64,
    pa: u64,
    step: u64,
) {
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

pub(crate) fn trace_stack_chk_call(
    cpu: &mut Armv8Cpu,
    bus: &SystemBus,
    pc: u64,
    instr: Instr,
    step: u64,
) {
    let target = (pc as i64).wrapping_add(instr.imm as i64) as u64;
    let Some(pa) = translate_read_only(&cpu.sys, Some(&cpu.tlb), &bus.mem, target).ok() else {
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
