use super::*;

pub(in crate::arm64::execute) fn exec_sve_shift_imm(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.size as usize;
    let elements = sve_vl_bytes(cpu) / element_size;
    let source = sve_read_z(cpu, instr.rn as usize);
    let mut result = if instr.cond == 0xFF { [0; 256] } else { source };
    let shift = instr.imm as u32;
    let pred = if instr.cond == 0xFF {
        [u64::MAX; 4]
    } else {
        cpu.sve_pred[instr.cond as usize]
    };

    for element in 0..elements {
        if predicate_element(&pred, element, element_size) {
            let value = sve_element(&source, element, element_size);
            sve_set_element(
                &mut result,
                element,
                element_size,
                shifted(instr, value, shift),
            );
        }
    }

    sve_write_z(cpu, instr.rd as usize, result);
}

fn shifted(instr: Instr, value: u64, shift: u32) -> u64 {
    let mask = sve_element_mask(instr.size as usize);
    match instr.op {
        Opcode::SveLslImm => value.wrapping_shl(shift) & mask,
        Opcode::SveLsrImm => logical_right(value, shift, instr.size as usize),
        Opcode::SveAsrImm => arithmetic_right(value, shift, instr.size as usize),
        _ => unreachable!(),
    }
}

fn logical_right(value: u64, shift: u32, element_size: usize) -> u64 {
    let bits = (element_size * 8) as u32;
    if shift >= bits {
        0
    } else {
        (value & sve_element_mask(element_size)) >> shift
    }
}

fn arithmetic_right(value: u64, shift: u32, element_size: usize) -> u64 {
    let bits = (element_size * 8) as u32;
    let mask = sve_element_mask(element_size);
    let sign = 1u64 << (bits - 1);
    if shift >= bits {
        if (value & sign) != 0 { mask } else { 0 }
    } else if (value & sign) == 0 {
        (value & mask) >> shift
    } else {
        ((value | !mask) as i64 >> shift) as u64 & mask
    }
}
