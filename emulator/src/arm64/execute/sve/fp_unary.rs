use super::*;
use crate::arm64::execute::round_ties_even;

pub(in crate::arm64::execute) fn exec_sve_fp_unary(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.size as usize;
    let elements = sve_vl_bytes(cpu) / element_size;
    let mask = cpu.sve_pred[instr.cond as usize];
    let source = sve_read_z(cpu, instr.rn as usize);
    let mut result = sve_read_z(cpu, instr.rd as usize);

    for element in 0..elements {
        if predicate_element(&mask, element, element_size) {
            let value = sve_element(&source, element, element_size);
            sve_set_element(
                &mut result,
                element,
                element_size,
                unary_value(instr.op, value, element_size),
            );
        }
    }

    sve_write_z(cpu, instr.rd as usize, result);
}

fn unary_value(op: Opcode, value: u64, element_size: usize) -> u64 {
    let sign = 1u64 << (element_size * 8 - 1);
    match op {
        Opcode::SveFpAbs => value & !sign,
        Opcode::SveFpNeg => value ^ sign,
        Opcode::SveFpSqrt => map_fp(value, element_size, f32::sqrt, f64::sqrt),
        Opcode::SveFpFrintn => map_fp(
            value,
            element_size,
            |v| round_ties_even(v as f64) as f32,
            round_ties_even,
        ),
        Opcode::SveFpFrinta => map_fp(value, element_size, f32::round, f64::round),
        Opcode::SveFpFrintz => map_fp(value, element_size, f32::trunc, f64::trunc),
        _ => unreachable!(),
    }
}

fn map_fp(
    value: u64,
    element_size: usize,
    op32: impl FnOnce(f32) -> f32,
    op64: impl FnOnce(f64) -> f64,
) -> u64 {
    match element_size {
        4 => op32(f32::from_bits(value as u32)).to_bits() as u64,
        8 => op64(f64::from_bits(value)).to_bits(),
        _ => unreachable!(),
    }
}
