use super::*;

mod arithmetic;
mod branching;
mod condition_logic;
mod control;
mod control_nop;
mod memory;
mod scalar_basic;
mod simd_data_ops;
mod simd_fp;
mod system_sve;

pub(super) enum Flow {
    Advance,
    Return,
}

pub(super) fn execute_local_body(
    cpu: &mut Armv8Cpu,
    instr: Instr,
) -> Result<Option<Flow>, &'static str> {
    if let Some(flow) = scalar_basic::execute(cpu, instr)? {
        return Ok(Some(flow));
    }
    if let Some(flow) = branching::execute(cpu, instr)? {
        return Ok(Some(flow));
    }
    if let Some(flow) = condition_logic::execute(cpu, instr)? {
        return Ok(Some(flow));
    }
    if let Some(flow) = arithmetic::execute(cpu, instr)? {
        return Ok(Some(flow));
    }
    if let Some(flow) = simd_fp::execute(cpu, instr)? {
        return Ok(Some(flow));
    }
    Ok(None)
}

pub(super) fn execute_body(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    instr: Instr,
) -> Result<Flow, &'static str> {
    if let Some(flow) = scalar_basic::execute(cpu, instr)? {
        return Ok(flow);
    }
    if let Some(flow) = memory::execute(cpu, bus, instr)? {
        return Ok(flow);
    }
    if let Some(flow) = branching::execute(cpu, instr)? {
        return Ok(flow);
    }
    if let Some(flow) = condition_logic::execute(cpu, instr)? {
        return Ok(flow);
    }
    if let Some(flow) = arithmetic::execute(cpu, instr)? {
        return Ok(flow);
    }
    if let Some(flow) = system_sve::execute(cpu, bus, instr)? {
        return Ok(flow);
    }
    if let Some(flow) = simd_fp::execute(cpu, instr)? {
        return Ok(flow);
    }
    if let Some(flow) = control::execute(cpu, bus, instr)? {
        return Ok(flow);
    }
    unreachable!("unknown opcode {:?}", instr.op)
}
