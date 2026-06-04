use super::*;

pub(super) fn execute(cpu: &mut Armv8Cpu, instr: Instr) -> Result<Option<Flow>, &'static str> {
    match instr.op {
        Opcode::SimdMovi => {
            cpu.simd[instr.rd as usize] = if instr.cond == 0 {
                simd_replicate_byte(instr.imm as u8) & simd_vector_mask(instr.size as usize)
            } else {
                simd_replicate_element(instr.imm as u128, instr.cond as usize, instr.size as usize)
            };
        }
        op if super::simd_data_ops::is_simd_data_opcode(op) => exec_simd_data(cpu, instr),
        Opcode::FpAdd
        | Opcode::FpSub
        | Opcode::FpMul
        | Opcode::FpFnmul
        | Opcode::FpDiv
        | Opcode::FpMaxnm
        | Opcode::FpMinnm
        | Opcode::FpNeg
        | Opcode::FpAbs
        | Opcode::FpSqrt
        | Opcode::FpFcvt
        | Opcode::FpFrintm
        | Opcode::FpFrintn
        | Opcode::FpFrinta
        | Opcode::FpFrintx
        | Opcode::FpFrintz
        | Opcode::FpFrintp
        | Opcode::FpFrinti
        | Opcode::FpMovImm
        | Opcode::Fmadd
        | Opcode::Fmsub
        | Opcode::Fnmsub
        | Opcode::Scvtf
        | Opcode::Ucvtf
        | Opcode::Fcvtns
        | Opcode::Fcvtms
        | Opcode::Fcvtzs
        | Opcode::Fcvtzu
        | Opcode::Fcvtas
        | Opcode::Fcmp
        | Opcode::Fcmpe
        | Opcode::Fccmp
        | Opcode::Fccmpe
        | Opcode::Fcsel => exec_fp_scalar(cpu, instr),
        _ => return Ok(None),
    }
    Ok(Some(Flow::Advance))
}
