use super::*;

pub(in crate::arm64::execute) fn exec_sve_movprfx(cpu: &mut Armv8Cpu, instr: Instr) {
    let vl_bytes = sve_vl_bytes(cpu);
    let source = sve_read_z(cpu, instr.rn as usize);
    let mut result = if instr.cond == 0xFF || !instr.sf {
        [0; 256]
    } else {
        sve_read_z(cpu, instr.rd as usize)
    };

    if instr.cond == 0xFF {
        result[..vl_bytes].copy_from_slice(&source[..vl_bytes]);
    } else {
        let element_size = instr.size as usize;
        let elements = vl_bytes / element_size;
        let mask = cpu.sve_pred[instr.cond as usize];
        for element in 0..elements {
            if predicate_element(&mask, element, element_size) {
                copy_sve_element(&mut result, &source, element, element_size);
            }
        }
    }

    sve_write_z(cpu, instr.rd as usize, result);
}

pub(in crate::arm64::execute) fn exec_sve_dup_gpr(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.size as usize;
    let elements = sve_vl_bytes(cpu) / element_size;
    let value = read_base(cpu, instr.rn, true) & sve_element_mask(element_size);
    let mut result = [0; 256];

    for element in 0..elements {
        sve_set_element(&mut result, element, element_size, value);
    }

    sve_write_z(cpu, instr.rd as usize, result);
}

pub(in crate::arm64::execute) fn exec_sve_int_binary(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.size as usize;
    let elements = sve_vl_bytes(cpu) / element_size;
    let lhs = sve_read_z(cpu, instr.rn as usize);
    let rhs = sve_read_z(cpu, instr.rm as usize);
    let mask = sve_element_mask(element_size);
    let mut result = [0; 256];

    for element in 0..elements {
        let left = sve_element(&lhs, element, element_size);
        let right = sve_element(&rhs, element, element_size);
        let value = match instr.op {
            Opcode::SveAddVec => left.wrapping_add(right),
            Opcode::SveSubVec => left.wrapping_sub(right),
            _ => unreachable!(),
        } & mask;
        sve_set_element(&mut result, element, element_size, value);
    }

    sve_write_z(cpu, instr.rd as usize, result);
}

pub(in crate::arm64::execute) fn exec_sve_logical_binary(cpu: &mut Armv8Cpu, instr: Instr) {
    let vl_bytes = sve_vl_bytes(cpu);
    let lhs = sve_read_z(cpu, instr.rn as usize);
    let rhs = sve_read_z(cpu, instr.rm as usize);
    let mut result = [0; 256];

    for byte in 0..vl_bytes {
        result[byte] = match instr.op {
            Opcode::SveOrrVec => lhs[byte] | rhs[byte],
            Opcode::SveEorVec => lhs[byte] ^ rhs[byte],
            _ => unreachable!(),
        };
    }

    sve_write_z(cpu, instr.rd as usize, result);
}

pub(in crate::arm64::execute) fn exec_sve_sel(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.size as usize;
    let vl_bytes = sve_vl_bytes(cpu);
    let elements = vl_bytes / element_size;
    let mask = cpu.sve_pred[instr.cond as usize];
    let true_source = sve_read_z(cpu, instr.rn as usize);
    let false_source = sve_read_z(cpu, instr.rm as usize);
    let mut result = [0; 256];

    for element in 0..elements {
        if predicate_element(&mask, element, element_size) {
            copy_sve_element(&mut result, &true_source, element, element_size);
        } else {
            copy_sve_element(&mut result, &false_source, element, element_size);
        }
    }

    sve_write_z(cpu, instr.rd as usize, result);
}
