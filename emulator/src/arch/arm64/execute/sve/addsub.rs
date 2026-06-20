use super::*;

pub(in crate::arch::arm64::execute) fn exec_sve_addsub_imm(cpu: &mut Armv8Cpu, instr: Instr) {
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

pub(in crate::arch::arm64::execute) fn exec_sve_addsub_pred(cpu: &mut Armv8Cpu, instr: Instr) {
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

pub(in crate::arch::arm64::execute) fn exec_sve_muladd_pred(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.size as usize;
    let elements = sve_vl_bytes(cpu) / element_size;
    let dest = sve_read_z(cpu, instr.rd as usize);
    let left = match instr.op {
        Opcode::SveMla | Opcode::SveMls => sve_read_z(cpu, instr.rn as usize),
        Opcode::SveMad | Opcode::SveMsb => dest,
        _ => unreachable!(),
    };
    let right = sve_read_z(cpu, instr.rm as usize);
    let addend = match instr.op {
        Opcode::SveMla | Opcode::SveMls => dest,
        Opcode::SveMad | Opcode::SveMsb => sve_read_z(cpu, instr.imm as usize),
        _ => unreachable!(),
    };
    let mask = sve_element_mask(element_size);
    let pred = cpu.sve_pred[instr.cond as usize];
    let mut result = dest;

    for element in 0..elements {
        if predicate_element(&pred, element, element_size) {
            let product = sve_element(&left, element, element_size).wrapping_mul(sve_element(
                &right,
                element,
                element_size,
            ));
            let base = sve_element(&addend, element, element_size);
            let value = match instr.op {
                Opcode::SveMla | Opcode::SveMad => base.wrapping_add(product),
                Opcode::SveMls | Opcode::SveMsb => base.wrapping_sub(product),
                _ => unreachable!(),
            } & mask;
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
