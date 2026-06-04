use super::*;

pub(in crate::arm64::execute) fn exec_sve_int_compare(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.size as usize;
    let vl_bytes = sve_vl_bytes(cpu);
    let elements = vl_bytes / element_size;
    let mask = cpu.sve_pred[instr.cond as usize];
    let lhs = sve_read_z(cpu, instr.rn as usize);
    let rhs = if is_immediate_compare(instr.op) {
        None
    } else {
        Some(sve_read_z(cpu, instr.rm as usize))
    };
    let mut result = [0; 4];

    for element in 0..elements {
        if predicate_element(&mask, element, element_size) {
            let left = sve_element(&lhs, element, element_size);
            let right = rhs
                .as_ref()
                .map_or(instr.imm, |vec| sve_element(vec, element, element_size));
            set_predicate_bit(
                &mut result,
                element * element_size,
                compare_elements(instr.op, left, right, element_size),
            );
        }
    }

    let flags = sve_pred_test(&mask, &result, element_size, vl_bytes);
    cpu.pstate.set_nzcv(flags.0, flags.1, flags.2, false);
    cpu.sve_pred[instr.rd as usize] = result;
}

fn is_immediate_compare(op: Opcode) -> bool {
    matches!(
        op,
        Opcode::SveCmpHsImm | Opcode::SveCmpHiImm | Opcode::SveCmpEqImm | Opcode::SveCmpNeImm
    )
}

fn compare_elements(op: Opcode, left: u64, right: u64, element_size: usize) -> bool {
    match op {
        Opcode::SveCmpHs | Opcode::SveCmpHsImm => left >= right,
        Opcode::SveCmpHi | Opcode::SveCmpHiImm => left > right,
        Opcode::SveCmpEq | Opcode::SveCmpEqImm => {
            sign_extend(left, element_size) == sign_extend(right, element_size)
        }
        Opcode::SveCmpNe | Opcode::SveCmpNeImm => {
            sign_extend(left, element_size) != sign_extend(right, element_size)
        }
        _ => false,
    }
}

fn sign_extend(value: u64, element_size: usize) -> i64 {
    match element_size {
        1 => value as u8 as i8 as i64,
        2 => value as u16 as i16 as i64,
        4 => value as u32 as i32 as i64,
        8 => value as i64,
        _ => 0,
    }
}

pub(in crate::arm64::execute) fn exec_sve_whilelo(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.size as usize;
    let vl_bytes = sve_vl_bytes(cpu);
    let elements = vl_bytes / element_size;
    let width_mask = if instr.sf { u64::MAX } else { u32::MAX as u64 };
    let limit = read_reg(cpu, instr.rm, instr.sf);
    let mut value = read_reg(cpu, instr.rn, instr.sf);
    let mut still_lower = true;
    let mut result = [0; 4];

    for element in 0..elements {
        still_lower &= value < limit;
        set_predicate_bit(&mut result, element * element_size, still_lower);
        value = value.wrapping_add(1) & width_mask;
    }

    let flags = sve_pred_test(&[u64::MAX; 4], &result, element_size, vl_bytes);
    cpu.pstate.set_nzcv(flags.0, flags.1, flags.2, false);
    cpu.sve_pred[instr.rd as usize] = result;
}
