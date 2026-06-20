use super::super::alu::{f16_to_f32, f32_to_f16_bits};
use super::*;

const COEFF_H: [u16; 16] = [
    0x3C00, 0xB155, 0x2030, 0x0000, 0x0000, 0x0000, 0x0000, 0x0000, 0x3C00, 0xB800, 0x293A, 0x0000,
    0x0000, 0x0000, 0x0000, 0x0000,
];

const COEFF_S: [u32; 16] = [
    0x3F80_0000,
    0xBE2A_AAAB,
    0x3C08_8886,
    0xB950_08B9,
    0x3636_9D6D,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x3F80_0000,
    0xBF00_0000,
    0x3D2A_AAA6,
    0xBAB6_0705,
    0x37CD_37CC,
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
];

const COEFF_D: [u64; 16] = [
    0x3FF0_0000_0000_0000,
    0xBFC5_5555_5555_5543,
    0x3F81_1111_1110_F30C,
    0xBF2A_01A0_19B9_2FC6,
    0x3EC7_1DE3_51F3_D22B,
    0xBE5A_E5E2_B60F_7B91,
    0x3DE5_D840_8868_552F,
    0x0000_0000_0000_0000,
    0x3FF0_0000_0000_0000,
    0xBFE0_0000_0000_0000,
    0x3FA5_5555_5555_5536,
    0xBF56_C16C_16C1_3A0B,
    0x3EFA_01A0_19B1_E8D8,
    0xBE92_7E4F_7282_F468,
    0x3E21_EE96_D264_1B13,
    0xBDA8_F763_80FB_B401,
];

pub(in crate::arch::arm64::execute) fn exec_sve_fp_ftmad(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.size as usize;
    let elements = sve_vl_bytes(cpu) / element_size;
    let lhs = sve_read_z(cpu, instr.rn as usize);
    let rhs = sve_read_z(cpu, instr.rm as usize);
    let mut result = [0; 256];

    for element in 0..elements {
        let left = sve_element(&lhs, element, element_size);
        let right = sve_element(&rhs, element, element_size);
        let value = ftmad(instr.imm as usize, left, right, element_size);
        sve_set_element(&mut result, element, element_size, value);
    }

    sve_write_z(cpu, instr.rd as usize, result);
}

fn ftmad(imm: usize, left: u64, right: u64, element_size: usize) -> u64 {
    match element_size {
        2 => ftmad_h(imm, left as u16, right as u16) as u64,
        4 => ftmad_s(imm, left as u32, right as u32) as u64,
        8 => ftmad_d(imm, left, right),
        _ => unreachable!(),
    }
}

fn ftmad_h(imm: usize, left: u16, right: u16) -> u16 {
    let index = imm + (((right >> 15) & 1) as usize * 8);
    let product = f16_to_f32(left).mul_add(f16_to_f32(right & 0x7FFF), f16_to_f32(COEFF_H[index]));
    f32_to_f16_bits(product)
}

fn ftmad_s(imm: usize, left: u32, right: u32) -> u32 {
    let index = imm + (((right >> 31) & 1) as usize * 8);
    let product = f32::from_bits(left).mul_add(
        f32::from_bits(right & 0x7FFF_FFFF),
        f32::from_bits(COEFF_S[index]),
    );
    product.to_bits()
}

fn ftmad_d(imm: usize, left: u64, right: u64) -> u64 {
    let index = imm + (((right >> 63) & 1) as usize * 8);
    let product = f64::from_bits(left).mul_add(
        f64::from_bits(right & 0x7FFF_FFFF_FFFF_FFFF),
        f64::from_bits(COEFF_D[index]),
    );
    product.to_bits()
}
