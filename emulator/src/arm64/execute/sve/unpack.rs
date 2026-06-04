use super::*;

pub(in crate::arm64::execute) fn exec_sve_unpack(cpu: &mut Armv8Cpu, instr: Instr) {
    match instr.op {
        Opcode::SvePunpklo | Opcode::SvePunpkhi => exec_predicate_unpack(cpu, instr),
        _ => exec_vector_unpack(cpu, instr),
    }
}

fn exec_vector_unpack(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.size as usize;
    let half_size = element_size / 2;
    let elements = sve_vl_bytes(cpu) / element_size;
    let offset = if unpack_high(instr.op) { elements } else { 0 };
    let source = sve_read_z(cpu, instr.rn as usize);
    let mut result = [0; 256];

    for element in 0..elements {
        let value = sve_element(&source, element + offset, half_size);
        let widened = widen(value, half_size, unpack_signed(instr.op));
        sve_set_element(&mut result, element, element_size, widened);
    }

    sve_write_z(cpu, instr.rd as usize, result);
}

fn exec_predicate_unpack(cpu: &mut Armv8Cpu, instr: Instr) {
    let vl_bytes = sve_vl_bytes(cpu);
    let elements = vl_bytes / 2;
    let offset = if unpack_high(instr.op) { elements } else { 0 };
    let source = cpu.sve_pred[instr.rn as usize];
    let mut result = [0; 4];

    for element in 0..elements {
        if predicate_element(&source, element + offset, 1) {
            set_predicate_bit(&mut result, element * 2, true);
        }
    }

    cpu.sve_pred[instr.rd as usize] = result;
}

fn widen(value: u64, size: usize, signed: bool) -> u64 {
    let bits = (size * 8) as u32;
    let mask = (1u64 << bits) - 1;
    if signed && (value & (1u64 << (bits - 1))) != 0 {
        value | !mask
    } else {
        value & mask
    }
}

fn unpack_high(op: Opcode) -> bool {
    matches!(
        op,
        Opcode::SveUunpkhi | Opcode::SveSunpkhi | Opcode::SvePunpkhi
    )
}

fn unpack_signed(op: Opcode) -> bool {
    matches!(op, Opcode::SveSunpklo | Opcode::SveSunpkhi)
}
