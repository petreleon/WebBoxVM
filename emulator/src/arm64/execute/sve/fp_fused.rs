use super::super::alu::{f16_to_f32, f32_to_f16_bits};
use super::*;

pub(in crate::arm64::execute) fn exec_sve_fp_fused(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.size as usize;
    let elements = sve_vl_bytes(cpu) / element_size;
    let mask = cpu.sve_pred[instr.cond as usize];
    let dest = sve_read_z(cpu, instr.rd as usize);
    let mut result = dest;
    let left = fused_left_operand(cpu, instr, dest);
    let rhs = sve_read_z(cpu, instr.rm as usize);
    let addend = fused_addend_operand(cpu, instr, dest);

    for element in 0..elements {
        if predicate_element(&mask, element, element_size) {
            let addend = sve_element(&addend, element, element_size);
            let left = sve_element(&left, element, element_size);
            let right = sve_element(&rhs, element, element_size);
            let value = fp_fused(instr.op, addend, left, right, element_size);
            sve_set_element(&mut result, element, element_size, value);
        }
    }

    sve_write_z(cpu, instr.rd as usize, result);
}

fn fused_left_operand(cpu: &mut Armv8Cpu, instr: Instr, dest: [u8; 256]) -> [u8; 256] {
    match instr.op {
        Opcode::SveFpFmla | Opcode::SveFpFmls => sve_read_z(cpu, instr.rn as usize),
        Opcode::SveFpFmad | Opcode::SveFpFmsb => dest,
        _ => unreachable!(),
    }
}

fn fused_addend_operand(cpu: &mut Armv8Cpu, instr: Instr, dest: [u8; 256]) -> [u8; 256] {
    match instr.op {
        Opcode::SveFpFmla | Opcode::SveFpFmls => dest,
        Opcode::SveFpFmad | Opcode::SveFpFmsb => sve_read_z(cpu, instr.imm as usize),
        _ => unreachable!(),
    }
}

fn fp_fused(op: Opcode, addend: u64, left: u64, right: u64, element_size: usize) -> u64 {
    match element_size {
        2 => fp_fused_f16(op, addend as u16, left as u16, right as u16) as u64,
        4 => fp_fused_f32(op, addend as u32, left as u32, right as u32) as u64,
        8 => fp_fused_f64(op, addend, left, right),
        _ => 0,
    }
}

fn fp_fused_f16(op: Opcode, addend: u16, left: u16, right: u16) -> u16 {
    let addend = f16_to_f32(addend);
    let left = f16_to_f32(left);
    let right = f16_to_f32(right);
    f32_to_f16_bits(fused_value(op, addend, left, right))
}

fn fp_fused_f32(op: Opcode, addend: u32, left: u32, right: u32) -> u32 {
    let addend = f32::from_bits(addend);
    let left = f32::from_bits(left);
    let right = f32::from_bits(right);
    fused_value(op, addend, left, right).to_bits()
}

fn fp_fused_f64(op: Opcode, addend: u64, left: u64, right: u64) -> u64 {
    let addend = f64::from_bits(addend);
    let left = f64::from_bits(left);
    let right = f64::from_bits(right);
    match op {
        Opcode::SveFpFmla | Opcode::SveFpFmad => left.mul_add(right, addend),
        Opcode::SveFpFmls | Opcode::SveFpFmsb => (-left).mul_add(right, addend),
        _ => unreachable!(),
    }
    .to_bits()
}

fn fused_value(op: Opcode, addend: f32, left: f32, right: f32) -> f32 {
    match op {
        Opcode::SveFpFmla | Opcode::SveFpFmad => left.mul_add(right, addend),
        Opcode::SveFpFmls | Opcode::SveFpFmsb => (-left).mul_add(right, addend),
        _ => unreachable!(),
    }
}
