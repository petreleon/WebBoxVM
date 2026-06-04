use super::super::alu::{f16_to_f32, f32_to_f16_bits};
use super::*;

pub(in crate::arm64::execute) fn exec_sve_fp_binary(cpu: &mut Armv8Cpu, instr: Instr) {
    if instr.op == Opcode::SveFpMulImm {
        exec_sve_fp_mul_imm(cpu, instr);
        return;
    }
    if instr.cond == 0xFF {
        exec_sve_fp_unpredicated(cpu, instr);
        return;
    }

    let element_size = instr.size as usize;
    let elements = sve_vl_bytes(cpu) / element_size;
    let mask = cpu.sve_pred[instr.cond as usize];
    let rhs = sve_read_z(cpu, instr.rm as usize);
    let mut result = sve_read_z(cpu, instr.rd as usize);

    for element in 0..elements {
        if predicate_element(&mask, element, element_size) {
            let left = sve_element(&result, element, element_size);
            let right = sve_element(&rhs, element, element_size);
            let value = fp_binary(instr.op, left, right, element_size);
            sve_set_element(&mut result, element, element_size, value);
        }
    }

    sve_write_z(cpu, instr.rd as usize, result);
}

fn exec_sve_fp_unpredicated(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.size as usize;
    let elements = sve_vl_bytes(cpu) / element_size;
    let lhs = sve_read_z(cpu, instr.rn as usize);
    let rhs = sve_read_z(cpu, instr.rm as usize);
    let mut result = [0; 256];

    for element in 0..elements {
        let left = sve_element(&lhs, element, element_size);
        let right = sve_element(&rhs, element, element_size);
        let value = fp_binary(instr.op, left, right, element_size);
        sve_set_element(&mut result, element, element_size, value);
    }

    sve_write_z(cpu, instr.rd as usize, result);
}

fn exec_sve_fp_mul_imm(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.size as usize;
    let elements = sve_vl_bytes(cpu) / element_size;
    let mask = cpu.sve_pred[instr.cond as usize];
    let imm = fmul_imm_bits(instr.imm != 0, element_size);
    let mut result = sve_read_z(cpu, instr.rd as usize);

    for element in 0..elements {
        if predicate_element(&mask, element, element_size) {
            let left = sve_element(&result, element, element_size);
            let value = fp_binary(Opcode::SveFpMul, left, imm, element_size);
            sve_set_element(&mut result, element, element_size, value);
        }
    }

    sve_write_z(cpu, instr.rd as usize, result);
}

fn fmul_imm_bits(two: bool, element_size: usize) -> u64 {
    match (two, element_size) {
        (false, 2) => 0x3800,
        (true, 2) => 0x4000,
        (false, 4) => 0.5f32.to_bits() as u64,
        (true, 4) => 2.0f32.to_bits() as u64,
        (false, 8) => 0.5f64.to_bits(),
        (true, 8) => 2.0f64.to_bits(),
        _ => 0,
    }
}

fn fp_binary(op: Opcode, left: u64, right: u64, element_size: usize) -> u64 {
    match element_size {
        2 => fp_binary_f16(op, left as u16, right as u16) as u64,
        4 => fp_binary_f32(op, left as u32, right as u32) as u64,
        8 => fp_binary_f64(op, left, right),
        _ => 0,
    }
}

fn fp_binary_f16(op: Opcode, left: u16, right: u16) -> u16 {
    let left = f16_to_f32(left);
    let right = f16_to_f32(right);
    f32_to_f16_bits(match op {
        Opcode::SveFpAdd => left + right,
        Opcode::SveFpSub => left - right,
        Opcode::SveFpMul => left * right,
        Opcode::SveFpSubr => right - left,
        _ => unreachable!(),
    })
}

fn fp_binary_f32(op: Opcode, left: u32, right: u32) -> u32 {
    let left = f32::from_bits(left);
    let right = f32::from_bits(right);
    match op {
        Opcode::SveFpAdd => left + right,
        Opcode::SveFpSub => left - right,
        Opcode::SveFpMul => left * right,
        Opcode::SveFpSubr => right - left,
        _ => unreachable!(),
    }
    .to_bits()
}

fn fp_binary_f64(op: Opcode, left: u64, right: u64) -> u64 {
    let left = f64::from_bits(left);
    let right = f64::from_bits(right);
    match op {
        Opcode::SveFpAdd => left + right,
        Opcode::SveFpSub => left - right,
        Opcode::SveFpMul => left * right,
        Opcode::SveFpSubr => right - left,
        _ => unreachable!(),
    }
    .to_bits()
}
