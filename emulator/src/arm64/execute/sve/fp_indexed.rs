use super::super::alu::{f16_to_f32, f32_to_f16_bits};
use super::*;

pub(in crate::arm64::execute) fn exec_sve_fp_indexed(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.size as usize;
    let elements = sve_vl_bytes(cpu) / element_size;
    let segment_elements = 16 / element_size;
    let left = sve_read_z(cpu, instr.rn as usize);
    let indexed = sve_read_z(cpu, instr.rm as usize);
    let mut result = indexed_initial_result(cpu, instr);

    for element in 0..elements {
        let indexed_element = element - (element % segment_elements) + instr.imm as usize;
        let addend = sve_element(&result, element, element_size);
        let left = sve_element(&left, element, element_size);
        let right = sve_element(&indexed, indexed_element, element_size);
        let value = indexed_value(instr.op, addend, left, right, element_size);
        sve_set_element(&mut result, element, element_size, value);
    }

    sve_write_z(cpu, instr.rd as usize, result);
}

fn indexed_initial_result(cpu: &mut Armv8Cpu, instr: Instr) -> [u8; 256] {
    match instr.op {
        Opcode::SveFpFmlaIndex | Opcode::SveFpFmlsIndex => sve_read_z(cpu, instr.rd as usize),
        Opcode::SveFpMulIndex => [0; 256],
        _ => unreachable!(),
    }
}

fn indexed_value(op: Opcode, addend: u64, left: u64, right: u64, element_size: usize) -> u64 {
    match element_size {
        2 => indexed_f16(op, addend as u16, left as u16, right as u16) as u64,
        4 => indexed_f32(op, addend as u32, left as u32, right as u32) as u64,
        8 => indexed_f64(op, addend, left, right),
        _ => 0,
    }
}

fn indexed_f16(op: Opcode, addend: u16, left: u16, right: u16) -> u16 {
    let addend = f16_to_f32(addend);
    let left = f16_to_f32(left);
    let right = f16_to_f32(right);
    f32_to_f16_bits(indexed_f32_value(op, addend, left, right))
}

fn indexed_f32(op: Opcode, addend: u32, left: u32, right: u32) -> u32 {
    let addend = f32::from_bits(addend);
    let left = f32::from_bits(left);
    let right = f32::from_bits(right);
    indexed_f32_value(op, addend, left, right).to_bits()
}

fn indexed_f64(op: Opcode, addend: u64, left: u64, right: u64) -> u64 {
    let addend = f64::from_bits(addend);
    let left = f64::from_bits(left);
    let right = f64::from_bits(right);
    match op {
        Opcode::SveFpFmlaIndex => left.mul_add(right, addend),
        Opcode::SveFpFmlsIndex => (-left).mul_add(right, addend),
        Opcode::SveFpMulIndex => left * right,
        _ => unreachable!(),
    }
    .to_bits()
}

fn indexed_f32_value(op: Opcode, addend: f32, left: f32, right: f32) -> f32 {
    match op {
        Opcode::SveFpFmlaIndex => left.mul_add(right, addend),
        Opcode::SveFpFmlsIndex => (-left).mul_add(right, addend),
        Opcode::SveFpMulIndex => left * right,
        _ => unreachable!(),
    }
}
