use super::*;

pub(in crate::arm64::execute) fn exec_sve_ld1_st1_w(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    instr: Instr,
) -> Result<(), &'static str> {
    let element_size = instr.size as usize;
    let elements = sve_vl_bytes(cpu) / element_size;
    let mask = cpu.sve_pred[instr.cond as usize];
    let base = read_base(cpu, instr.rn, true);

    match instr.op {
        Opcode::SveLd1w => load_words(cpu, bus, instr, base, &mask, elements, element_size),
        Opcode::SveSt1w => store_words(cpu, bus, instr, base, &mask, elements, element_size),
        _ => unreachable!(),
    }
}

fn load_words(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    instr: Instr,
    base: u64,
    mask: &[u64; 4],
    elements: usize,
    element_size: usize,
) -> Result<(), &'static str> {
    let mut result = [0; 256];
    if instr.rm == 0xFF {
        let vector_offset = (instr.imm as i64).wrapping_mul((elements * 4) as i64) as u64;
        load_contiguous_words(
            cpu,
            bus,
            base.wrapping_add(vector_offset),
            mask,
            &mut result,
            elements,
            element_size,
        )?;
    } else {
        let offsets = sve_read_z(cpu, instr.rm as usize);
        for element in 0..elements {
            if predicate_element(mask, element, element_size) {
                let offset = scaled_word_offset(
                    sve_element(&offsets, element, element_size),
                    instr.sf,
                    element_size,
                );
                let bytes =
                    read_sve_bytes(cpu, bus, base.wrapping_add(offset), 4, "SVE load fault")?;
                sve_set_element(
                    &mut result,
                    element,
                    element_size,
                    sve_element(&bytes, 0, 4),
                );
            }
        }
    }
    sve_write_z(cpu, instr.rd as usize, result);
    Ok(())
}

fn load_contiguous_words(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    va: u64,
    mask: &[u64; 4],
    result: &mut [u8; 256],
    elements: usize,
    element_size: usize,
) -> Result<(), &'static str> {
    for element in 0..elements {
        if predicate_element(mask, element, element_size) {
            let bytes = read_sve_bytes(
                cpu,
                bus,
                va.wrapping_add((element * 4) as u64),
                4,
                "SVE load fault",
            )?;
            sve_set_element(result, element, element_size, sve_element(&bytes, 0, 4));
        }
    }
    Ok(())
}

fn store_words(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    instr: Instr,
    base: u64,
    mask: &[u64; 4],
    elements: usize,
    element_size: usize,
) -> Result<(), &'static str> {
    let source = sve_read_z(cpu, instr.rd as usize);
    let vector_offset = (instr.imm as i64).wrapping_mul((elements * 4) as i64) as u64;
    let va = base.wrapping_add(vector_offset);
    for element in 0..elements {
        if predicate_element(mask, element, element_size) {
            let offset = element * element_size;
            write_sve_bytes(
                cpu,
                bus,
                va.wrapping_add((element * 4) as u64),
                &source[offset..offset + 4],
                "SVE store fault",
            )?;
        }
    }
    Ok(())
}

fn scaled_word_offset(raw: u64, signed: bool, element_size: usize) -> u64 {
    if signed && element_size == 4 {
        ((raw as u32 as i32 as i64) as u64).wrapping_shl(2)
    } else {
        raw.wrapping_shl(2)
    }
}
