use super::*;

pub(in crate::arch::arm64::execute) fn exec_sve_dup_imm(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.size as usize;
    let elements = sve_vl_bytes(cpu) / element_size;
    let bytes = instr.imm.to_le_bytes();
    let mut result = [0; 256];

    for element in 0..elements {
        copy_bytes(&mut result, element, element_size, &bytes);
    }

    sve_write_z(cpu, instr.rd as usize, result);
}

pub(in crate::arch::arm64::execute) fn exec_sve_dup_elem(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.size as usize;
    let elements = sve_vl_bytes(cpu) / element_size;
    let index = instr.imm as usize;
    let source = sve_read_z(cpu, instr.rn as usize);
    let mut result = [0; 256];
    let mut element_bytes = [0; 16];

    if index < elements {
        let offset = index * element_size;
        element_bytes[..element_size].copy_from_slice(&source[offset..offset + element_size]);
    }
    for element in 0..elements {
        copy_bytes(&mut result, element, element_size, &element_bytes);
    }

    sve_write_z(cpu, instr.rd as usize, result);
}

pub(in crate::arch::arm64::execute) fn exec_sve_cpy_imm(cpu: &mut Armv8Cpu, instr: Instr) {
    exec_sve_cpy_value(cpu, instr, instr.imm);
}

pub(in crate::arch::arm64::execute) fn exec_sve_cpy_gpr(cpu: &mut Armv8Cpu, instr: Instr) {
    let value = read_base(cpu, instr.rn, true);
    exec_sve_cpy_value(cpu, instr, value);
}

fn exec_sve_cpy_value(cpu: &mut Armv8Cpu, instr: Instr, value: u64) {
    let element_size = instr.size as usize;
    let elements = sve_vl_bytes(cpu) / element_size;
    let mask = cpu.sve_pred[instr.cond as usize];
    let mut result = if instr.sf {
        sve_read_z(cpu, instr.rd as usize)
    } else {
        [0; 256]
    };

    for element in 0..elements {
        if predicate_element(&mask, element, element_size) {
            sve_set_element(&mut result, element, element_size, value);
        }
    }

    sve_write_z(cpu, instr.rd as usize, result);
}

fn copy_bytes(result: &mut [u8; 256], element: usize, element_size: usize, source: &[u8]) {
    let offset = element * element_size;
    result[offset..offset + element_size].copy_from_slice(&source[..element_size]);
}
