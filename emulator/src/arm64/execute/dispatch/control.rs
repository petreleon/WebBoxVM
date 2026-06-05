use super::*;

pub(super) fn execute(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    instr: Instr,
) -> Result<Option<Flow>, &'static str> {
    match instr.op {
        Opcode::Tlbi => {
            cpu.tlb.invalidate_all();
        }
        Opcode::DcZva | Opcode::DcGzva => exec_dc_zva(cpu, bus, instr)?,
        Opcode::DcGva => exec_dc_gva(cpu, bus, instr)?,
        Opcode::Svc => {
            exec_svc(cpu, instr.imm)?;
            return Ok(Some(Flow::Return));
        }
        Opcode::Eret => {
            exec_eret(cpu)?;
            return Ok(Some(Flow::Return));
        }
        Opcode::Brk => {
            exec_brk(cpu, bus, instr)?;
            return Ok(Some(Flow::Return));
        }
        Opcode::Udf => {
            exec_udf(cpu)?;
            return Ok(Some(Flow::Return));
        }
        Opcode::Cfinv => exec_cfinv(cpu),
        Opcode::Rmif => exec_rmif(cpu, instr),
        Opcode::Setf8 | Opcode::Setf16 => exec_setf(cpu, instr),
        Opcode::SmeLdrZa | Opcode::SmeStrZa | Opcode::SmeSmlal => {
            exec_udf(cpu)?;
            return Ok(Some(Flow::Return));
        }
        Opcode::Nop
        | Opcode::NopBarrier
        | Opcode::Chkfeat
        | Opcode::GcsPushM
        | Opcode::GcsPushX
        | Opcode::GcsPopM
        | Opcode::GcsPopX
        | Opcode::GcsPopCx
        | Opcode::GcsSs1
        | Opcode::GcsSs2
        | Opcode::Smstop
        | Opcode::Pacia1716
        | Opcode::Pacib1716
        | Opcode::Autia1716
        | Opcode::Autib1716
        | Opcode::Paciaz
        | Opcode::Paciasp
        | Opcode::Pacibz
        | Opcode::Pacibsp
        | Opcode::Autiaz
        | Opcode::Autiasp
        | Opcode::Autibz
        | Opcode::Autibsp
        | Opcode::Xpaclri
        | Opcode::Bti
        | Opcode::BtiC
        | Opcode::BtiJ
        | Opcode::BtiJc
        | Opcode::Sev
        | Opcode::Sevl
        | Opcode::Esb
        | Opcode::PsbCsync
        | Opcode::TsbCsync
        | Opcode::GcsbDsync
        | Opcode::Csdb
        | Opcode::Clrbhb => exec_nop_like(cpu, instr),
        Opcode::Wfi | Opcode::Wfe => advance_timer_deadline(cpu),
        _ => return Ok(None),
    }
    Ok(Some(Flow::Advance))
}

fn exec_cfinv(cpu: &mut Armv8Cpu) {
    cpu.pstate.set_nzcv(
        cpu.pstate.n(),
        cpu.pstate.z(),
        !cpu.pstate.c(),
        cpu.pstate.v(),
    );
}

fn exec_rmif(cpu: &mut Armv8Cpu, instr: Instr) {
    let bits = cpu.regs.x(instr.rn).rotate_right(instr.imm as u32) as u8;
    let mask = instr.cond;
    let n = if mask & 8 != 0 {
        bits & 8 != 0
    } else {
        cpu.pstate.n()
    };
    let z = if mask & 4 != 0 {
        bits & 4 != 0
    } else {
        cpu.pstate.z()
    };
    let c = if mask & 2 != 0 {
        bits & 2 != 0
    } else {
        cpu.pstate.c()
    };
    let v = if mask & 1 != 0 {
        bits & 1 != 0
    } else {
        cpu.pstate.v()
    };
    cpu.pstate.set_nzcv(n, z, c, v);
}

fn exec_setf(cpu: &mut Armv8Cpu, instr: Instr) {
    let size = if instr.op == Opcode::Setf8 { 8 } else { 16 };
    let value = cpu.regs.w(instr.rn);
    let low_mask = (1u32 << size) - 1;
    let n = value & (1 << (size - 1)) != 0;
    let z = value & low_mask == 0;
    let v = ((value >> size) ^ (value >> (size - 1))) & 1 != 0;
    cpu.pstate.set_nzcv(n, z, cpu.pstate.c(), v);
}

fn exec_nop_like(cpu: &mut Armv8Cpu, instr: Instr) {
    if instr.cond == 1 {
        let bits = instr.imm as u8;
        if bits & 2 != 0 {
            trace_daif(cpu, "daifset");
            cpu.pstate = cpu.pstate.with_irq_masked(true);
            trace_daif(cpu, "daifset ->");
        }
    } else if instr.cond == 2 {
        let bits = instr.imm as u8;
        if bits & 2 != 0 {
            trace_daif(cpu, "daifclr");
            cpu.pstate = cpu.pstate.with_irq_masked(false);
            trace_daif(cpu, "daifclr ->");
        }
    } else if instr.cond == 3 {
        cpu.clear_exclusive();
    } else if instr.cond == 4 {
        advance_timer_deadline(cpu);
    }
}

fn advance_timer_deadline(cpu: &mut Armv8Cpu) {
    if let Some(deadline) = cpu.sys.next_timer_deadline()
        && deadline > cpu.sys.cycle_count
    {
        cpu.sys.cycle_count = deadline;
    }
}
