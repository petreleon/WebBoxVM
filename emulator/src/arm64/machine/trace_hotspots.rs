use super::*;

pub(in crate::arm64::machine) fn trace_mprotect_loop_state(
    cpu: &Armv8Cpu,
    pc: u64,
    pa: u64,
    instr: Instr,
    step: u64,
) {
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

pub(in crate::arm64::machine) fn trace_rwsem_loop(
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
