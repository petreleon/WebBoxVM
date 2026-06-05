use super::*;

const OFFSET_32: u64 = 2;

pub(in crate::arm64::execute) fn exec_sve_st1h(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    instr: Instr,
) -> Result<(), &'static str> {
    let element_size = instr.size as usize;
    let elements = sve_vl_bytes(cpu) / element_size;
    let mask = cpu.sve_pred[instr.cond as usize];
    let source = sve_read_z(cpu, instr.rd as usize);
    let base = st1h_base(cpu, instr, (elements * 2) as i64);

    for element in 0..elements {
        if predicate_element(&mask, element, element_size) {
            let source_offset = element * element_size;
            write_sve_bytes(
                cpu,
                bus,
                base.wrapping_add((element * 2) as u64),
                &source[source_offset..source_offset + 2],
                "SVE store fault",
            )?;
        }
    }
    Ok(())
}

pub(in crate::arm64::execute) fn exec_sve_st1h_scatter(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    instr: Instr,
) -> Result<(), &'static str> {
    let element_size = instr.size as usize;
    let elements = sve_vl_bytes(cpu) / element_size;
    let mask = cpu.sve_pred[instr.cond as usize];
    let source = sve_read_z(cpu, instr.rd as usize);
    let offsets = sve_read_z(cpu, instr.rm as usize);
    let base = read_base(cpu, instr.rn, true);

    for element in 0..elements {
        if predicate_element(&mask, element, element_size) {
            let source_offset = element * element_size;
            let offset = st1h_scatter_offset(&offsets, element, element_size, instr);
            write_sve_bytes(
                cpu,
                bus,
                base.wrapping_add(offset),
                &source[source_offset..source_offset + 2],
                "SVE store fault",
            )?;
        }
    }
    Ok(())
}

fn st1h_base(cpu: &Armv8Cpu, instr: Instr, in_memory_bytes: i64) -> u64 {
    if instr.rm == 0xFF {
        let offset = (instr.imm as i64).wrapping_mul(in_memory_bytes) as u64;
        read_base(cpu, instr.rn, true).wrapping_add(offset)
    } else {
        let offset = read_reg(cpu, instr.rm, true).wrapping_shl(1);
        read_base(cpu, instr.rn, true).wrapping_add(offset)
    }
}

fn st1h_scatter_offset(
    offsets: &[u8; 256],
    element: usize,
    element_size: usize,
    instr: Instr,
) -> u64 {
    let raw = sve_element(offsets, element, element_size);
    let offset = if (instr.imm & OFFSET_32) != 0 {
        if instr.sf {
            raw as u32 as i32 as i64 as u64
        } else {
            raw as u32 as u64
        }
    } else {
        raw
    };
    offset.wrapping_shl((instr.imm & 1) as u32)
}
