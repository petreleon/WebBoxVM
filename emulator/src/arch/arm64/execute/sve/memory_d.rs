use super::*;

pub(in crate::arch::arm64::execute) fn exec_sve_ld1_st1_d(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    instr: Instr,
) -> Result<(), &'static str> {
    let element_size = instr.size as usize;
    let vl_bytes = sve_vl_bytes(cpu);
    let elements = vl_bytes / element_size;
    let mask = cpu.sve_pred[instr.cond as usize];
    let base = read_base(cpu, instr.rn, true);

    match instr.op {
        Opcode::SveLd1rd => {
            let active =
                (0..elements).any(|element| predicate_element(&mask, element, element_size));
            let mut result = [0; 256];
            if active {
                let va = base.wrapping_add(instr.imm);
                let bytes = read_sve_bytes(cpu, bus, va, element_size, "SVE load fault")?;
                let value = sve_element(&bytes, 0, element_size);
                for element in 0..elements {
                    if predicate_element(&mask, element, element_size) {
                        sve_set_element(&mut result, element, element_size, value);
                    }
                }
            }
            sve_write_z(cpu, instr.rd as usize, result);
        }
        Opcode::SveLd1rqd => {
            let va = base.wrapping_add(instr.imm);
            let mut pattern = [0; 2];
            for (element, slot) in pattern.iter_mut().enumerate() {
                if predicate_element(&mask, element, element_size) {
                    let bytes = read_sve_bytes(
                        cpu,
                        bus,
                        va.wrapping_add((element * element_size) as u64),
                        element_size,
                        "SVE load fault",
                    )?;
                    *slot = sve_element(&bytes, 0, element_size);
                }
            }

            let mut result = [0; 256];
            for element in 0..elements {
                sve_set_element(&mut result, element, element_size, pattern[element % 2]);
            }
            sve_write_z(cpu, instr.rd as usize, result);
        }
        Opcode::SveLd1d => {
            let mut result = [0; 256];
            if instr.rm == 0xFF {
                let vector_offset = (instr.imm as i64).wrapping_mul(vl_bytes as i64) as u64;
                let va = base.wrapping_add(vector_offset);
                for element in 0..elements {
                    if predicate_element(&mask, element, element_size) {
                        let bytes = read_sve_bytes(
                            cpu,
                            bus,
                            va.wrapping_add((element * element_size) as u64),
                            element_size,
                            "SVE load fault",
                        )?;
                        let value = sve_element(&bytes, 0, element_size);
                        sve_set_element(&mut result, element, element_size, value);
                    }
                }
            } else {
                let offsets = sve_read_z(cpu, instr.rm as usize);
                for element in 0..elements {
                    if predicate_element(&mask, element, element_size) {
                        let scaled = sve_element(&offsets, element, element_size).wrapping_mul(8);
                        let bytes = read_sve_bytes(
                            cpu,
                            bus,
                            base.wrapping_add(scaled),
                            element_size,
                            "SVE load fault",
                        )?;
                        let value = sve_element(&bytes, 0, element_size);
                        sve_set_element(&mut result, element, element_size, value);
                    }
                }
            }
            sve_write_z(cpu, instr.rd as usize, result);
        }
        Opcode::SveSt1d => {
            let vector_offset = (instr.imm as i64).wrapping_mul(vl_bytes as i64) as u64;
            let va = base.wrapping_add(vector_offset);
            let source = sve_read_z(cpu, instr.rd as usize);
            for element in 0..elements {
                if predicate_element(&mask, element, element_size) {
                    let offset = element * element_size;
                    write_sve_bytes(
                        cpu,
                        bus,
                        va.wrapping_add(offset as u64),
                        &source[offset..offset + element_size],
                        "SVE store fault",
                    )?;
                }
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}
