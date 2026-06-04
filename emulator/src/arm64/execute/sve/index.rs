use super::*;

const START_IMM: u8 = 1;
const STEP_IMM: u8 = 2;

pub(in crate::arm64::execute) fn exec_sve_index(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.size as usize;
    let elements = sve_vl_bytes(cpu) / element_size;
    let start = operand(
        cpu,
        instr.rn,
        instr.imm as u32,
        instr.cond & START_IMM != 0,
        instr,
    );
    let step = operand(
        cpu,
        instr.rm,
        (instr.imm >> 32) as u32,
        instr.cond & STEP_IMM != 0,
        instr,
    );
    let mut result = [0; 256];

    for element in 0..elements {
        let index = start.wrapping_add((element as i64).wrapping_mul(step));
        sve_set_element(&mut result, element, element_size, index as u64);
    }

    sve_write_z(cpu, instr.rd as usize, result);
}

fn operand(cpu: &Armv8Cpu, reg: u8, imm: u32, is_imm: bool, instr: Instr) -> i64 {
    if is_imm {
        return imm as i32 as i64;
    }
    sign_extend_element(read_reg(cpu, reg, instr.sf), instr.size as usize)
}

fn sign_extend_element(value: u64, element_size: usize) -> i64 {
    match element_size {
        1 => value as u8 as i8 as i64,
        2 => value as u16 as i16 as i64,
        4 => value as u32 as i32 as i64,
        8 => value as i64,
        _ => 0,
    }
}
