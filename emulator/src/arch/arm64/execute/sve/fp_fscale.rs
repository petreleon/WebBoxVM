use super::super::alu::{f16_to_f32, f32_to_f16_bits};
use super::*;

pub(in crate::arch::arm64::execute) fn exec_sve_fp_fscale(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.size as usize;
    let elements = sve_vl_bytes(cpu) / element_size;
    let mask = cpu.sve_pred[instr.cond as usize];
    let lhs = sve_read_z(cpu, instr.rn as usize);
    let rhs = sve_read_z(cpu, instr.rm as usize);
    let mut result = lhs;

    for element in 0..elements {
        if predicate_element(&mask, element, element_size) {
            let left = sve_element(&lhs, element, element_size);
            let right = sve_element(&rhs, element, element_size);
            let value = fscale(left, right, element_size);
            sve_set_element(&mut result, element, element_size, value);
        }
    }

    sve_write_z(cpu, instr.rd as usize, result);
}

fn fscale(left: u64, right: u64, element_size: usize) -> u64 {
    match element_size {
        2 => fscale_h(left as u16, signed_scale(right, 16)) as u64,
        4 => fscale_s(left as u32, signed_scale(right, 32)) as u64,
        8 => fscale_d(left, signed_scale(right, 64)),
        _ => unreachable!(),
    }
}

fn fscale_h(left: u16, scale: i64) -> u16 {
    let scale = clamp_scale(left as u64, 16, scale);
    f32_to_f16_bits(f16_to_f32(left) * 2.0f32.powi(scale))
}

fn fscale_s(left: u32, scale: i64) -> u32 {
    let scale = clamp_scale(left as u64, 32, scale);
    (f32::from_bits(left) * 2.0f32.powi(scale)).to_bits()
}

fn fscale_d(left: u64, scale: i64) -> u64 {
    let scale = clamp_scale(left, 64, scale);
    (f64::from_bits(left) * 2.0f64.powi(scale)).to_bits()
}

fn signed_scale(value: u64, bits: u32) -> i64 {
    match bits {
        16 => (value as u16 as i16) as i64,
        32 => (value as u32 as i32) as i64,
        64 => value as i64,
        _ => unreachable!(),
    }
}

fn clamp_scale(value: u64, bits: u32, scale: i64) -> i32 {
    let (exp_bits, frac_bits, exp) = match bits {
        16 => (5, 10, ((value >> 10) & 0x1F) as i64),
        32 => (8, 23, ((value >> 23) & 0xFF) as i64),
        64 => (11, 52, ((value >> 52) & 0x7FF) as i64),
        _ => unreachable!(),
    };
    let emax = (1i64 << exp_bits) - 1;
    let min_scale = -(frac_bits + 1);
    let max_scale = emax + frac_bits + 1;
    scale.clamp(min_scale - exp, max_scale - exp) as i32
}
