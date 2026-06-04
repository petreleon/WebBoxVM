use super::*;

pub(in crate::arm64::execute) fn exec_sve_int_compare(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.size as usize;
    let vl_bytes = sve_vl_bytes(cpu);
    let elements = vl_bytes / element_size;
    let mask = cpu.sve_pred[instr.cond as usize];
    let lhs = sve_read_z(cpu, instr.rn as usize);
    let rhs = if instr.op == Opcode::SveCmpHs {
        Some(sve_read_z(cpu, instr.rm as usize))
    } else {
        None
    };
    let mut result = [0; 4];

    for element in 0..elements {
        if predicate_element(&mask, element, element_size) {
            let left = sve_element(&lhs, element, element_size);
            let right = rhs
                .as_ref()
                .map_or(instr.imm, |vec| sve_element(vec, element, element_size));
            set_predicate_bit(&mut result, element * element_size, left >= right);
        }
    }

    let flags = sve_pred_test(&mask, &result, element_size, vl_bytes);
    cpu.pstate.set_nzcv(flags.0, flags.1, flags.2, false);
    cpu.sve_pred[instr.rd as usize] = result;
}
