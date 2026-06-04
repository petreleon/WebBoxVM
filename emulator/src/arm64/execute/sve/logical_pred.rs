use super::*;

pub(in crate::arm64::execute) fn exec_sve_logical_pred(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.size as usize;
    let elements = sve_vl_bytes(cpu) / element_size;
    let lhs = sve_read_z(cpu, instr.rn as usize);
    let rhs = sve_read_z(cpu, instr.rm as usize);
    let pred = cpu.sve_pred[instr.cond as usize];
    let mut result = lhs;

    for element in 0..elements {
        if predicate_element(&pred, element, element_size) {
            apply_element(instr.op, &mut result, &lhs, &rhs, element, element_size);
        }
    }

    sve_write_z(cpu, instr.rd as usize, result);
}

fn apply_element(
    op: Opcode,
    result: &mut [u8; 256],
    lhs: &[u8; 256],
    rhs: &[u8; 256],
    element: usize,
    element_size: usize,
) {
    let start = element * element_size;
    for byte in start..start + element_size {
        result[byte] = match op {
            Opcode::SveAndPred => lhs[byte] & rhs[byte],
            Opcode::SveOrrPred => lhs[byte] | rhs[byte],
            Opcode::SveEorPred => lhs[byte] ^ rhs[byte],
            _ => unreachable!(),
        };
    }
}
