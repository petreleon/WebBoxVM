use super::*;

pub(super) fn execute(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    instr: Instr,
) -> Result<Option<Flow>, &'static str> {
    match instr.op {
        Opcode::Ldr
        | Opcode::LdrSign
        | Opcode::Str
        | Opcode::SimdLdr
        | Opcode::SimdStr
        | Opcode::SimdLd1
        | Opcode::SimdLd1Multi
        | Opcode::SimdLd1Lane
        | Opcode::SimdLd1r
        | Opcode::SimdLd2
        | Opcode::SimdLd3
        | Opcode::SimdSt1Multi
        | Opcode::SimdSt1Lane
        | Opcode::SimdLd4
        | Opcode::SimdSt4Single
        | Opcode::SimdSt4 => exec_ldr_str(cpu, bus, instr)?,
        Opcode::LdrLit => exec_ldr_lit(cpu, bus, instr)?,
        Opcode::Ldp | Opcode::Ldpsw | Opcode::Stp | Opcode::SimdLdp | Opcode::SimdStp => {
            exec_ldp_stp(cpu, bus, instr)?
        }
        Opcode::Ldxr | Opcode::Ldar | Opcode::Stxr | Opcode::Stlr | Opcode::Ldxp | Opcode::Stxp => {
            exec_exclusive(cpu, bus, instr)?
        }
        Opcode::Atomic | Opcode::AtomicPair | Opcode::Cas | Opcode::Casp => {
            exec_atomic(cpu, bus, instr)?
        }
        Opcode::MteIrg | Opcode::MteGmi => exec_mte_gpr(cpu, instr),
        Opcode::MteLdg | Opcode::MteStg | Opcode::MteStzg | Opcode::MteSt2g | Opcode::MteStz2g => {
            exec_mte_mem(cpu, bus, instr)?
        }
        _ => return Ok(None),
    }
    Ok(Some(Flow::Advance))
}
