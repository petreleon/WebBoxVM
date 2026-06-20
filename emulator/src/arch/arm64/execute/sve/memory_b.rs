use super::*;

pub(in crate::arch::arm64::execute) fn exec_sve_st1b(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    instr: Instr,
) -> Result<(), &'static str> {
    let element_size = instr.size as usize;
    let vl_bytes = sve_vl_bytes(cpu);
    let elements = vl_bytes / element_size;
    let base = st1b_base(cpu, instr, elements as i64);
    let mask = cpu.sve_pred[instr.cond as usize];
    let source = sve_read_z(cpu, instr.rd as usize);

    for element in 0..elements {
        if predicate_element(&mask, element, element_size) {
            let source_offset = element * element_size;
            write_sve_bytes(
                cpu,
                bus,
                base.wrapping_add(element as u64),
                &source[source_offset..source_offset + 1],
                "SVE store fault",
            )?;
        }
    }

    Ok(())
}

fn st1b_base(cpu: &Armv8Cpu, instr: Instr, in_memory_bytes: i64) -> u64 {
    if instr.rm == 0xFF {
        let offset = (instr.imm as i64).wrapping_mul(in_memory_bytes) as u64;
        read_base(cpu, instr.rn, true).wrapping_add(offset)
    } else {
        read_base(cpu, instr.rn, true).wrapping_add(read_reg(cpu, instr.rm, true))
    }
}
