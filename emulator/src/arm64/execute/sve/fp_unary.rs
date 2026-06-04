use super::*;

pub(in crate::arm64::execute) fn exec_sve_fp_unary(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.size as usize;
    let elements = sve_vl_bytes(cpu) / element_size;
    let mask = cpu.sve_pred[instr.cond as usize];
    let source = sve_read_z(cpu, instr.rn as usize);
    let mut result = sve_read_z(cpu, instr.rd as usize);

    for element in 0..elements {
        if predicate_element(&mask, element, element_size) {
            let value = sve_element(&source, element, element_size);
            sve_set_element(
                &mut result,
                element,
                element_size,
                unary_value(instr.op, value, element_size),
            );
        }
    }

    sve_write_z(cpu, instr.rd as usize, result);
}

fn unary_value(op: Opcode, value: u64, element_size: usize) -> u64 {
    let sign = 1u64 << (element_size * 8 - 1);
    match op {
        Opcode::SveFpAbs => value & !sign,
        Opcode::SveFpNeg => value ^ sign,
        _ => unreachable!(),
    }
}
