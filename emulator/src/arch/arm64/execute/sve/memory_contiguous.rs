use super::*;

pub(in crate::arch::arm64::execute) fn exec_sve_contiguous_load(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    instr: Instr,
) -> Result<(), &'static str> {
    match instr.op {
        Opcode::SveLd1b => exec_ld1b(cpu, bus, instr),
        Opcode::SveLd1rw => exec_ld1rw(cpu, bus, instr),
        Opcode::SveLd1rqw => exec_ld1rqw(cpu, bus, instr),
        _ => unreachable!(),
    }
}

fn exec_ld1b(cpu: &mut Armv8Cpu, bus: &mut SystemBus, instr: Instr) -> Result<(), &'static str> {
    let element_size = instr.size as usize;
    let elements = sve_vl_bytes(cpu) / element_size;
    let base = byte_base(cpu, instr, elements as i64);
    let mask = cpu.sve_pred[instr.cond as usize];
    let mut result = [0; 256];
    for element in 0..elements {
        if predicate_element(&mask, element, element_size) {
            let bytes = read_sve_bytes(
                cpu,
                bus,
                base.wrapping_add(element as u64),
                1,
                "SVE load fault",
            )?;
            sve_set_element(&mut result, element, element_size, bytes[0] as u64);
        }
    }
    sve_write_z(cpu, instr.rd as usize, result);
    Ok(())
}

fn exec_ld1rw(cpu: &mut Armv8Cpu, bus: &mut SystemBus, instr: Instr) -> Result<(), &'static str> {
    let mask = cpu.sve_pred[instr.cond as usize];
    let elements = sve_vl_bytes(cpu) / 4;
    let mut result = [0; 256];
    if (0..elements).any(|element| predicate_element(&mask, element, 4)) {
        let va = read_base(cpu, instr.rn, true).wrapping_add(instr.imm);
        let bytes = read_sve_bytes(cpu, bus, va, 4, "SVE load fault")?;
        let value = sve_element(&bytes, 0, 4);
        for element in 0..elements {
            if predicate_element(&mask, element, 4) {
                sve_set_element(&mut result, element, 4, value);
            }
        }
    }
    sve_write_z(cpu, instr.rd as usize, result);
    Ok(())
}

fn exec_ld1rqw(cpu: &mut Armv8Cpu, bus: &mut SystemBus, instr: Instr) -> Result<(), &'static str> {
    let mask = cpu.sve_pred[instr.cond as usize];
    let elements = sve_vl_bytes(cpu) / 4;
    let va = read_base(cpu, instr.rn, true).wrapping_add(instr.imm);
    let mut pattern = [0; 4];
    for (element, slot) in pattern.iter_mut().enumerate() {
        if predicate_element(&mask, element, 4) {
            let bytes = read_sve_bytes(
                cpu,
                bus,
                va.wrapping_add((element * 4) as u64),
                4,
                "SVE load fault",
            )?;
            *slot = sve_element(&bytes, 0, 4);
        }
    }
    let mut result = [0; 256];
    for element in 0..elements {
        sve_set_element(&mut result, element, 4, pattern[element % 4]);
    }
    sve_write_z(cpu, instr.rd as usize, result);
    Ok(())
}

fn vector_base(cpu: &Armv8Cpu, instr: Instr, in_memory_bytes: i64) -> u64 {
    let offset = (instr.imm as i64).wrapping_mul(in_memory_bytes) as u64;
    read_base(cpu, instr.rn, true).wrapping_add(offset)
}

fn byte_base(cpu: &Armv8Cpu, instr: Instr, in_memory_bytes: i64) -> u64 {
    if instr.rm == 0xFF {
        vector_base(cpu, instr, in_memory_bytes)
    } else {
        read_base(cpu, instr.rn, true).wrapping_add(read_reg(cpu, instr.rm, true))
    }
}
