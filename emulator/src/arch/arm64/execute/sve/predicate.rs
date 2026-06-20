use super::*;

pub(in crate::arch::arm64::execute) fn exec_sve_ptrue(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.size as usize;
    let vl_bytes = sve_vl_bytes(cpu);
    let elements = vl_bytes / element_size;
    let count = sve_pred_count(instr.cond, elements as u64) as usize;
    let mut pred = [0; 4];

    for element in 0..count {
        set_predicate_bit(&mut pred, element * element_size, true);
    }

    if instr.op == Opcode::SvePtrues {
        let flags = sve_pred_test(&pred, &pred, element_size, vl_bytes);
        cpu.pstate.set_nzcv(flags.0, flags.1, flags.2, false);
    }
    cpu.sve_pred[instr.rd as usize] = pred;
}

pub(in crate::arch::arm64::execute) fn exec_sve_ptest(cpu: &mut Armv8Cpu, instr: Instr) {
    let flags = sve_pred_test(
        &cpu.sve_pred[instr.rd as usize],
        &cpu.sve_pred[instr.rn as usize],
        instr.size as usize,
        sve_vl_bytes(cpu),
    );
    cpu.pstate.set_nzcv(flags.0, flags.1, flags.2, false);
}

pub(in crate::arch::arm64::execute) fn exec_sve_pred_logical(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.size as usize;
    let vl_bytes = sve_vl_bytes(cpu);
    let elements = vl_bytes / element_size;
    let mask = cpu.sve_pred[instr.cond as usize];
    let operand1 = cpu.sve_pred[instr.rn as usize];
    let operand2 = cpu.sve_pred[instr.rm as usize];
    let mut result = [0; 4];

    for element in 0..elements {
        if predicate_element(&mask, element, element_size) {
            let left = predicate_element(&operand1, element, element_size);
            let right = predicate_element(&operand2, element, element_size);
            let bit = pred_logical_bit(instr.op, left, right);
            set_predicate_bit(&mut result, element * element_size, bit);
        }
    }

    if instr.sf {
        let flags = sve_pred_test(&mask, &result, element_size, vl_bytes);
        cpu.pstate.set_nzcv(flags.0, flags.1, flags.2, false);
    }
    cpu.sve_pred[instr.rd as usize] = result;
}

fn pred_logical_bit(op: Opcode, left: bool, right: bool) -> bool {
    match op {
        Opcode::SvePredAnd => left && right,
        Opcode::SvePredBic => left && !right,
        Opcode::SvePredOrr => left || right,
        Opcode::SvePredOrn => left || !right,
        Opcode::SvePredEor => left ^ right,
        Opcode::SvePredNor => !(left || right),
        Opcode::SvePredNand => !(left && right),
        _ => unreachable!(),
    }
}

pub(in crate::arch::arm64::execute) fn sve_pred_test(
    mask: &[u64; 4],
    result: &[u64; 4],
    element_size: usize,
    vl_bytes: usize,
) -> (bool, bool, bool) {
    let elements = vl_bytes / element_size;
    let mut first_active = None;
    let mut last_active = None;
    let mut any = false;

    for element in 0..elements {
        if predicate_element(mask, element, element_size) {
            first_active.get_or_insert(element);
            last_active = Some(element);
            any |= predicate_element(result, element, element_size);
        }
    }

    let first =
        first_active.is_some_and(|element| predicate_element(result, element, element_size));
    let last = last_active.is_some_and(|element| predicate_element(result, element, element_size));
    (first, !any, !last)
}
