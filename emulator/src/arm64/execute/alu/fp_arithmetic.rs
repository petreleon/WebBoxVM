use super::*;

pub(in crate::arm64::execute) fn exec_fp_arithmetic(cpu: &mut Armv8Cpu, instr: Instr) {
    match instr.op {
        Opcode::FpAdd => exec_fp_binary(cpu, instr, |a, b| a + b, |a, b| a + b),
        Opcode::FpSub => exec_fp_binary(cpu, instr, |a, b| a - b, |a, b| a - b),
        Opcode::FpMul => exec_fp_binary(cpu, instr, |a, b| a * b, |a, b| a * b),
        Opcode::FpFnmul => exec_fp_binary(cpu, instr, |a, b| -(a * b), |a, b| -(a * b)),
        Opcode::FpDiv => exec_fp_binary(cpu, instr, |a, b| a / b, |a, b| a / b),
        Opcode::FpMax => exec_fp_binary(cpu, instr, fp_max, fp_max),
        Opcode::FpMin => exec_fp_binary(cpu, instr, fp_min, fp_min),
        Opcode::FpMaxnm => exec_fp_binary(cpu, instr, f32::max, f64::max),
        Opcode::FpMinnm => exec_fp_binary(cpu, instr, f32::min, f64::min),
        Opcode::Fmadd => exec_fp_fused(cpu, instr, false),
        Opcode::Fmsub => exec_fp_fused(cpu, instr, true),
        Opcode::Fnmsub => exec_fp_fnmsub(cpu, instr),
        _ => unreachable!(),
    }
}

pub(in crate::arm64::execute) fn exec_fp_fnmsub(cpu: &mut Armv8Cpu, instr: Instr) {
    if instr.size == 4 {
        let n = f32::from_bits(read_fp_bits(cpu, instr.rn, 4) as u32);
        let m = f32::from_bits(read_fp_bits(cpu, instr.rm, 4) as u32);
        let a = f32::from_bits(read_fp_bits(cpu, instr.cond, 4) as u32);
        write_fp_bits(cpu, instr.rd, n.mul_add(m, -a).to_bits() as u64, 4);
    } else {
        let n = f64::from_bits(read_fp_bits(cpu, instr.rn, 8));
        let m = f64::from_bits(read_fp_bits(cpu, instr.rm, 8));
        let a = f64::from_bits(read_fp_bits(cpu, instr.cond, 8));
        write_fp_bits(cpu, instr.rd, n.mul_add(m, -a).to_bits(), 8);
    }
}

pub(in crate::arm64::execute) fn exec_fp_fused(
    cpu: &mut Armv8Cpu,
    instr: Instr,
    subtract_product: bool,
) {
    if instr.size == 4 {
        let n = f32::from_bits(read_fp_bits(cpu, instr.rn, 4) as u32);
        let m = f32::from_bits(read_fp_bits(cpu, instr.rm, 4) as u32);
        let a = f32::from_bits(read_fp_bits(cpu, instr.cond, 4) as u32);
        let value = if subtract_product {
            (-n).mul_add(m, a)
        } else {
            n.mul_add(m, a)
        };
        write_fp_bits(cpu, instr.rd, value.to_bits() as u64, 4);
    } else {
        let n = f64::from_bits(read_fp_bits(cpu, instr.rn, 8));
        let m = f64::from_bits(read_fp_bits(cpu, instr.rm, 8));
        let a = f64::from_bits(read_fp_bits(cpu, instr.cond, 8));
        let value = if subtract_product {
            (-n).mul_add(m, a)
        } else {
            n.mul_add(m, a)
        };
        write_fp_bits(cpu, instr.rd, value.to_bits(), 8);
    }
}

pub(in crate::arm64::execute) fn exec_fp_binary<F32, F64>(
    cpu: &mut Armv8Cpu,
    instr: Instr,
    op32: F32,
    op64: F64,
) where
    F32: FnOnce(f32, f32) -> f32,
    F64: FnOnce(f64, f64) -> f64,
{
    if instr.size == 4 {
        let lhs = f32::from_bits(read_fp_bits(cpu, instr.rn, 4) as u32);
        let rhs = f32::from_bits(read_fp_bits(cpu, instr.rm, 4) as u32);
        write_fp_bits(cpu, instr.rd, op32(lhs, rhs).to_bits() as u64, 4);
    } else {
        let lhs = f64::from_bits(read_fp_bits(cpu, instr.rn, 8));
        let rhs = f64::from_bits(read_fp_bits(cpu, instr.rm, 8));
        write_fp_bits(cpu, instr.rd, op64(lhs, rhs).to_bits(), 8);
    }
}
