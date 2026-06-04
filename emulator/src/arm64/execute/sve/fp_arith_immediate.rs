use super::*;

pub(in crate::arm64::execute) fn sve_fp_arith_is_immediate(instr: Instr) -> bool {
    matches!(instr.op, Opcode::SveFpAddImm | Opcode::SveFpMulImm)
        || (matches!(instr.op, Opcode::SveFpSub | Opcode::SveFpSubr) && instr.rm == 0xFF)
}

pub(in crate::arm64::execute) fn exec_sve_fp_imm(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.size as usize;
    let elements = sve_vl_bytes(cpu) / element_size;
    let mask = cpu.sve_pred[instr.cond as usize];
    let op = immediate_op(instr.op);
    let imm = fp_imm_bits(instr.op, instr.imm != 0, element_size);
    let mut result = sve_read_z(cpu, instr.rd as usize);

    for element in 0..elements {
        if predicate_element(&mask, element, element_size) {
            let left = sve_element(&result, element, element_size);
            let value = super::fp::fp_binary(op, left, imm, element_size);
            sve_set_element(&mut result, element, element_size, value);
        }
    }

    sve_write_z(cpu, instr.rd as usize, result);
}

fn immediate_op(op: Opcode) -> Opcode {
    match op {
        Opcode::SveFpAddImm => Opcode::SveFpAdd,
        Opcode::SveFpMulImm => Opcode::SveFpMul,
        Opcode::SveFpSub | Opcode::SveFpSubr => op,
        _ => unreachable!(),
    }
}

fn fp_imm_bits(op: Opcode, high: bool, element_size: usize) -> u64 {
    match op {
        Opcode::SveFpMulImm => fmul_imm_bits(high, element_size),
        Opcode::SveFpAddImm | Opcode::SveFpSub | Opcode::SveFpSubr => {
            fadd_imm_bits(high, element_size)
        }
        _ => unreachable!(),
    }
}

fn fadd_imm_bits(one: bool, element_size: usize) -> u64 {
    match (one, element_size) {
        (false, 2) => 0x3800,
        (true, 2) => 0x3C00,
        (false, 4) => 0.5f32.to_bits() as u64,
        (true, 4) => 1.0f32.to_bits() as u64,
        (false, 8) => 0.5f64.to_bits(),
        (true, 8) => 1.0f64.to_bits(),
        _ => 0,
    }
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
