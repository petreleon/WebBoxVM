use super::*;

pub(in crate::arm64::execute) fn exec_sve_ld1h_family(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    instr: Instr,
) -> Result<(), &'static str> {
    let element_size = instr.size as usize;
    let elements = sve_vl_bytes(cpu) / element_size;
    let mask = cpu.sve_pred[instr.cond as usize];
    let mut result = [0; 256];

    match instr.op {
        Opcode::SveLd1h if instr.rm == 0xFF => {
            let base = read_base(cpu, instr.rn, true);
            let offset = (instr.imm as i64).wrapping_mul((elements * 2) as i64) as u64;
            load_scalar_base(
                cpu,
                bus,
                base.wrapping_add(offset),
                &mask,
                &mut result,
                elements,
                element_size,
            )?;
        }
        Opcode::SveLd1h => {
            let base = read_base(cpu, instr.rn, true);
            let offsets = sve_read_z(cpu, instr.rm as usize);
            for element in 0..elements {
                if predicate_element(&mask, element, element_size) {
                    let offset = sve_element(&offsets, element, element_size);
                    let offset = if instr.sf {
                        offset.wrapping_shl(1)
                    } else {
                        offset
                    };
                    let value =
                        load_half(cpu, bus, base.wrapping_add(offset), false, element_size)?;
                    sve_set_element(&mut result, element, element_size, value);
                }
            }
        }
        Opcode::SveLdnt1sh => {
            let bases = sve_read_z(cpu, instr.rn as usize);
            let offset = read_reg(cpu, instr.rm, true);
            for element in 0..elements {
                if predicate_element(&mask, element, element_size) {
                    let base = sve_element(&bases, element, element_size);
                    let value = load_half(cpu, bus, base.wrapping_add(offset), true, element_size)?;
                    sve_set_element(&mut result, element, element_size, value);
                }
            }
        }
        _ => unreachable!(),
    }

    sve_write_z(cpu, instr.rd as usize, result);
    Ok(())
}

fn load_scalar_base(
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
            let value = load_half(
                cpu,
                bus,
                va.wrapping_add((element * 2) as u64),
                false,
                element_size,
            )?;
            sve_set_element(result, element, element_size, value);
        }
    }
    Ok(())
}

fn load_half(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    va: u64,
    signed: bool,
    element_size: usize,
) -> Result<u64, &'static str> {
    let bytes = read_sve_bytes(cpu, bus, va, 2, "SVE load fault")?;
    let raw = u16::from_le_bytes([bytes[0], bytes[1]]);
    Ok(if signed {
        sign_extend_half(raw, element_size)
    } else {
        raw as u64
    })
}

fn sign_extend_half(raw: u16, element_size: usize) -> u64 {
    match element_size {
        4 => raw as i16 as i32 as u32 as u64,
        8 => raw as i16 as i64 as u64,
        _ => raw as u64,
    }
}
