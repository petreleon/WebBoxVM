use super::*;

pub(in crate::arm64::execute) fn exec_sve_addsub_imm(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.size as usize;
    let elements = sve_vl_bytes(cpu) / element_size;
    let source = sve_read_z(cpu, instr.rn as usize);
    let mask = sve_element_mask(element_size);
    let imm = instr.imm & mask;
    let mut result = [0; 256];

    for element in 0..elements {
        let left = sve_element(&source, element, element_size);
        let value = addsub_value(instr.op, left, imm) & mask;
        sve_set_element(&mut result, element, element_size, value);
    }

    sve_write_z(cpu, instr.rd as usize, result);
}

pub(in crate::arm64::execute) fn exec_sve_addsub_pred(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.size as usize;
    let elements = sve_vl_bytes(cpu) / element_size;
    let lhs = sve_read_z(cpu, instr.rn as usize);
    let rhs = sve_read_z(cpu, instr.rm as usize);
    let mask = sve_element_mask(element_size);
    let pred = cpu.sve_pred[instr.cond as usize];
    let mut result = lhs;

    for element in 0..elements {
        if predicate_element(&pred, element, element_size) {
            let left = sve_element(&lhs, element, element_size);
            let right = sve_element(&rhs, element, element_size);
            let value = addsub_value(instr.op, left, right) & mask;
            sve_set_element(&mut result, element, element_size, value);
        }
    }

    sve_write_z(cpu, instr.rd as usize, result);
}

fn addsub_value(op: Opcode, left: u64, right: u64) -> u64 {
    match op {
        Opcode::SveAddImm | Opcode::SveAddPred => left.wrapping_add(right),
        Opcode::SveSubImm | Opcode::SveSubPred => left.wrapping_sub(right),
        _ => unreachable!(),
    }
}
