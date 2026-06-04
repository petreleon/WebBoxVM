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
        Opcode::SmeLdrZa | Opcode::SmeStrZa => {
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
        | Opcode::Smstop => exec_nop_like(cpu, instr),
        Opcode::Wfi | Opcode::Wfe => advance_timer_deadline(cpu),
        _ => return Ok(None),
    }
    Ok(Some(Flow::Advance))
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
