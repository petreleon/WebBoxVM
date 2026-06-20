use super::super::alu::{f16_to_f32, f32_to_f16_bits};
use super::*;

pub(in crate::arch::arm64::execute) fn exec_sve_fp_trig_pair(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.size as usize;
    let elements = sve_vl_bytes(cpu) / element_size;
    let lhs = sve_read_z(cpu, instr.rn as usize);
    let rhs = sve_read_z(cpu, instr.rm as usize);
    let mut result = [0; 256];

    for element in 0..elements {
        let left = sve_element(&lhs, element, element_size);
        let right = sve_element(&rhs, element, element_size);
        let value = trig_value(instr.op, left, right, element_size);
        sve_set_element(&mut result, element, element_size, value);
    }

    sve_write_z(cpu, instr.rd as usize, result);
}

fn trig_value(op: Opcode, left: u64, right: u64, element_size: usize) -> u64 {
    match op {
        Opcode::SveFpFtsmul => ftsmul(left, right, element_size),
        Opcode::SveFpFtssel => ftssel(left, right, element_size),
        _ => unreachable!(),
    }
}

fn ftsmul(left: u64, right: u64, element_size: usize) -> u64 {
    let value = match element_size {
        2 => f32_to_f16_bits(f16_to_f32(left as u16) * f16_to_f32(left as u16)) as u64,
        4 => (f32::from_bits(left as u32) * f32::from_bits(left as u32)).to_bits() as u64,
        8 => (f64::from_bits(left) * f64::from_bits(left)).to_bits(),
        _ => unreachable!(),
    };
    set_sign_unless_nan(value, right & 1 != 0, element_size)
}

fn ftssel(left: u64, right: u64, element_size: usize) -> u64 {
    match (right & 1 != 0, right & 2 != 0) {
        (true, sign) => fp_one(sign, element_size),
        (false, true) => left ^ sign_mask(element_size),
        (false, false) => left,
    }
}

fn set_sign_unless_nan(value: u64, sign: bool, element_size: usize) -> u64 {
    if fp_is_nan(value, element_size) {
        value
    } else if sign {
        value | sign_mask(element_size)
    } else {
        value & !sign_mask(element_size)
    }
}

fn fp_one(sign: bool, element_size: usize) -> u64 {
    match (element_size, sign) {
        (2, false) => 0x3C00,
        (2, true) => 0xBC00,
        (4, false) => 0x3F80_0000,
        (4, true) => 0xBF80_0000,
        (8, false) => 0x3FF0_0000_0000_0000,
        (8, true) => 0xBFF0_0000_0000_0000,
        _ => unreachable!(),
    }
}

fn sign_mask(element_size: usize) -> u64 {
    1u64 << (element_size * 8 - 1)
}

fn fp_is_nan(value: u64, element_size: usize) -> bool {
    match element_size {
        2 => (value & 0x7C00) == 0x7C00 && (value & 0x03FF) != 0,
        4 => (value & 0x7F80_0000) == 0x7F80_0000 && (value & 0x007F_FFFF) != 0,
        8 => {
            (value & 0x7FF0_0000_0000_0000) == 0x7FF0_0000_0000_0000
                && (value & 0x000F_FFFF_FFFF_FFFF) != 0
        }
        _ => unreachable!(),
    }
}
