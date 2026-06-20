use super::*;

pub(in crate::arch::arm64::execute) fn exec_sve_ldr_str(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    instr: Instr,
) -> Result<(), &'static str> {
    let is_vector = instr.cond == 1;
    let transfer_bytes = if is_vector {
        sve_vl_bytes(cpu)
    } else {
        sve_pl_bytes(cpu)
    };
    let offset = (instr.imm as i64).wrapping_mul(transfer_bytes as i64) as u64;
    let va = read_base(cpu, instr.rn, true).wrapping_add(offset);

    match instr.op {
        Opcode::SveLdr => {
            let bytes = read_sve_bytes(cpu, bus, va, transfer_bytes, "SVE load fault")?;
            if is_vector {
                let mut value = [0; 256];
                value[..transfer_bytes].copy_from_slice(&bytes[..transfer_bytes]);
                sve_write_z(cpu, instr.rd as usize, value);
            } else {
                cpu.sve_pred[(instr.rd & 0xF) as usize] = predicate_from_bytes(&bytes);
            }
        }
        Opcode::SveStr => {
            if is_vector {
                let value = sve_read_z(cpu, instr.rd as usize);
                write_sve_bytes(cpu, bus, va, &value[..transfer_bytes], "SVE store fault")?;
            } else {
                let bytes = predicate_to_bytes(cpu.sve_pred[(instr.rd & 0xF) as usize]);
                write_sve_bytes(cpu, bus, va, &bytes[..transfer_bytes], "SVE store fault")?;
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}
