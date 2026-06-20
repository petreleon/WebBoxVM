use super::*;

pub(in crate::arch::arm64::execute) fn exec_fp_scalar(cpu: &mut Armv8Cpu, instr: Instr) {
    match instr.op {
        Opcode::FpAdd
        | Opcode::FpSub
        | Opcode::FpMul
        | Opcode::FpFnmul
        | Opcode::FpDiv
        | Opcode::FpMax
        | Opcode::FpMin
        | Opcode::FpMaxnm
        | Opcode::FpMinnm
        | Opcode::Fmadd
        | Opcode::Fmsub
        | Opcode::Fnmadd
        | Opcode::Fnmsub => exec_fp_arithmetic(cpu, instr),
        Opcode::FpNeg | Opcode::FpAbs | Opcode::FpSqrt | Opcode::FpFcvt | Opcode::FpMovImm => {
            exec_fp_unary(cpu, instr)
        }
        Opcode::FpFrintm
        | Opcode::FpFrintn
        | Opcode::FpFrinta
        | Opcode::FpFrintx
        | Opcode::FpFrintz
        | Opcode::FpFrintp
        | Opcode::FpFrinti => exec_fp_rounding(cpu, instr),
        Opcode::Scvtf
        | Opcode::Ucvtf
        | Opcode::Fcvtns
        | Opcode::Fcvtms
        | Opcode::Fcvtzs
        | Opcode::Fcvtzu
        | Opcode::Fcvtas => exec_fp_convert(cpu, instr),
        Opcode::Fcmp | Opcode::Fcmpe | Opcode::Fccmp | Opcode::Fccmpe | Opcode::Fcsel => {
            exec_fp_compare(cpu, instr)
        }
        _ => unreachable!(),
    }
}
