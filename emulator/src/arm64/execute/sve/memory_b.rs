use super::*;

pub(in crate::arm64::execute) fn exec_sve_st1b(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    instr: Instr,
) -> Result<(), &'static str> {
    let element_size = instr.size as usize;
    let vl_bytes = sve_vl_bytes(cpu);
    let elements = vl_bytes / element_size;
    let in_memory_bytes = elements as i64;
    let vector_offset = (instr.imm as i64).wrapping_mul(in_memory_bytes) as u64;
    let base = read_base(cpu, instr.rn, true).wrapping_add(vector_offset);
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
