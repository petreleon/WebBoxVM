use super::*;

const TAG_GRANULE: u64 = 16;
const LOGICAL_TAG_SHIFT: u64 = 56;
const LOGICAL_TAG_MASK: u64 = 0x0F00_0000_0000_0000;
const USER_TOP_BYTE_MASK: u64 = 0xFF00_0000_0000_0000;
const TAGLESS_EXCLUDE_MASK: u16 = 0xFFFE;

pub(in crate::arch::arm64::execute) fn exec_mte_gpr(cpu: &mut Armv8Cpu, instr: Instr) {
    match instr.op {
        Opcode::MteIrg => {
            let exclude = read_reg(cpu, instr.rm, true) as u16;
            let tagged = with_logical_tag(read_base(cpu, instr.rn, true), choose_tag(exclude));
            write_reg_sp(cpu, instr.rd, tagged, true);
        }
        Opcode::MteGmi => {
            let tag = logical_tag(read_base(cpu, instr.rn, true));
            let mask = read_reg(cpu, instr.rm, true) | (1u64 << tag);
            write_reg(cpu, instr.rd, mask, true);
        }
        Opcode::MteAddg | Opcode::MteSubg => {
            let base = read_base(cpu, instr.rn, true);
            let address = if instr.op == Opcode::MteAddg {
                base.wrapping_add(instr.imm)
            } else {
                base.wrapping_sub(instr.imm)
            };
            let tag = choose_tag_from(logical_tag(base), instr.cond, TAGLESS_EXCLUDE_MASK);
            write_reg_sp(cpu, instr.rd, with_logical_tag(address, tag), true);
        }
        _ => unreachable!(),
    }
}

pub(in crate::arch::arm64::execute) fn exec_mte_mem(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    instr: Instr,
) -> Result<(), &'static str> {
    let (address, writeback) = mte_address(cpu, instr);
    let va = untag_user_address(address);

    match instr.op {
        Opcode::MteLdg => exec_ldg(cpu, bus, va, instr)?,
        Opcode::MteStg | Opcode::MteSt2g => check_tag_store(cpu, bus, va, instr.size)?,
        Opcode::MteStzg | Opcode::MteStz2g => zero_tag_granules(cpu, bus, va, instr.size)?,
        _ => unreachable!(),
    }

    if let Some(new_base) = writeback {
        write_reg_sp(cpu, instr.rn, new_base, true);
    }
    Ok(())
}

fn exec_ldg(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    va: u64,
    instr: Instr,
) -> Result<(), &'static str> {
    let granule = va & !(TAG_GRANULE - 1);
    translate_or_data_fault(cpu, &mut bus.mem, granule, false, "LDG translation fault")?;
    let tagged = with_logical_tag(read_reg(cpu, instr.rd, true), 0);
    write_reg(cpu, instr.rd, tagged, true);
    Ok(())
}

fn check_tag_store(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    va: u64,
    size: u8,
) -> Result<(), &'static str> {
    for offset in (0..size as u64).step_by(TAG_GRANULE as usize) {
        let granule = (va.wrapping_add(offset)) & !(TAG_GRANULE - 1);
        translate_or_data_fault(cpu, &mut bus.mem, granule, true, "STG translation fault")?;
    }
    Ok(())
}

fn zero_tag_granules(
    cpu: &mut Armv8Cpu,
    bus: &mut SystemBus,
    va: u64,
    size: u8,
) -> Result<(), &'static str> {
    if (va & (TAG_GRANULE - 1)) != 0 {
        return Err("MTE tag granule alignment fault");
    }
    for offset in (0..size as u64).step_by(TAG_GRANULE as usize) {
        write_simd_guest(cpu, bus, va.wrapping_add(offset), 16, 0, "STZG bus fault")?;
    }
    Ok(())
}

fn mte_address(cpu: &Armv8Cpu, instr: Instr) -> (u64, Option<u64>) {
    let base = read_base(cpu, instr.rn, true);
    match instr.cond {
        1 => (base, Some(base.wrapping_add(instr.imm))),
        3 => {
            let address = base.wrapping_add(instr.imm);
            (address, Some(address))
        }
        _ => (base.wrapping_add(instr.imm), None),
    }
}

fn choose_tag(exclude: u16) -> u8 {
    choose_tag_from(0, 0, exclude)
}

fn choose_tag_from(start: u8, offset: u8, exclude: u16) -> u8 {
    let first = start.wrapping_add(offset) & 0xF;
    (0..16)
        .map(|step| first.wrapping_add(step) & 0xF)
        .find(|tag| (exclude & (1 << tag)) == 0)
        .unwrap_or(0)
}

fn logical_tag(address: u64) -> u8 {
    ((address & LOGICAL_TAG_MASK) >> LOGICAL_TAG_SHIFT) as u8
}

fn with_logical_tag(address: u64, tag: u8) -> u64 {
    (address & !LOGICAL_TAG_MASK) | (((tag & 0xF) as u64) << LOGICAL_TAG_SHIFT)
}

fn untag_user_address(address: u64) -> u64 {
    if (address & (1 << 55)) == 0 {
        address & !USER_TOP_BYTE_MASK
    } else {
        address
    }
}
