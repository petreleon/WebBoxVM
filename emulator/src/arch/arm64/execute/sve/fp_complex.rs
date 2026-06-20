use super::super::alu::{f16_to_f32, f32_to_f16_bits};
use super::*;

pub(in crate::arch::arm64::execute) fn exec_sve_fp_complex(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.size as usize;
    let pairs = sve_vl_bytes(cpu) / (element_size * 2);
    let mask = cpu.sve_pred[instr.cond as usize];
    let left = sve_read_z(cpu, instr.rn as usize);
    let right = sve_read_z(cpu, instr.rm as usize);
    let mut result = sve_read_z(cpu, instr.rd as usize);
    let rot = (instr.imm / 90) as usize;
    let sel_a = rot & 1;
    let sel_b = sel_a ^ 1;
    let neg_i = (rot & 2) != 0;
    let neg_r = ((rot & 1) != 0) != neg_i;

    for pair in 0..pairs {
        let real = pair * 2;
        let imag = real + 1;
        if predicate_element(&mask, real, element_size) {
            fcmla_lane(
                &mut result,
                &left,
                &right,
                real,
                sel_a,
                sel_a,
                neg_r,
                element_size,
            );
        }
        if predicate_element(&mask, imag, element_size) {
            fcmla_lane(
                &mut result,
                &left,
                &right,
                imag,
                sel_a,
                sel_b,
                neg_i,
                element_size,
            );
        }
    }

    sve_write_z(cpu, instr.rd as usize, result);
}

fn fcmla_lane(
    result: &mut [u8; 256],
    left: &[u8; 256],
    right: &[u8; 256],
    lane: usize,
    lhs_select: usize,
    rhs_select: usize,
    negate_rhs: bool,
    element_size: usize,
) {
    let pair_base = lane & !1;
    let lhs_index = pair_base + lhs_select;
    let rhs_index = pair_base + rhs_select;
    let addend = sve_element(result, lane, element_size);
    let lhs = sve_element(left, lhs_index, element_size);
    let rhs = sve_element(right, rhs_index, element_size);
    let value = complex_fma(addend, lhs, rhs, negate_rhs, element_size);
    sve_set_element(result, lane, element_size, value);
}

fn complex_fma(addend: u64, left: u64, right: u64, negate_right: bool, element_size: usize) -> u64 {
    match element_size {
        2 => complex_fma_f16(addend as u16, left as u16, right as u16, negate_right) as u64,
        4 => complex_fma_f32(addend as u32, left as u32, right as u32, negate_right) as u64,
        8 => complex_fma_f64(addend, left, right, negate_right),
        _ => unreachable!(),
    }
}

fn complex_fma_f16(addend: u16, left: u16, right: u16, negate_right: bool) -> u16 {
    let addend = f16_to_f32(addend);
    let left = f16_to_f32(left);
    let right = f16_to_f32(right);
    f32_to_f16_bits(left.mul_add(if negate_right { -right } else { right }, addend))
}

fn complex_fma_f32(addend: u32, left: u32, right: u32, negate_right: bool) -> u32 {
    let addend = f32::from_bits(addend);
    let left = f32::from_bits(left);
    let right = f32::from_bits(right);
    left.mul_add(if negate_right { -right } else { right }, addend)
        .to_bits()
}

fn complex_fma_f64(addend: u64, left: u64, right: u64, negate_right: bool) -> u64 {
    let addend = f64::from_bits(addend);
    let left = f64::from_bits(left);
    let right = f64::from_bits(right);
    left.mul_add(if negate_right { -right } else { right }, addend)
        .to_bits()
}
