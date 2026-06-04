use super::*;

pub(in crate::arm64::execute) fn compute_ldst_va(
    cpu: &Armv8Cpu,
    instr: &Instr,
) -> (u64, Option<u64>) {
    if matches!(instr.op, Opcode::SimdLd1Lane | Opcode::SimdSt1Lane) {
        let base = base_addr(cpu, instr.rn);
        let writeback = if instr.rm == 0xFF {
            None
        } else if instr.rm == SIMD_MULTI_POST_INDEX {
            Some(base.wrapping_add(instr.cond.max(1) as u64))
        } else {
            Some(base.wrapping_add(read_reg(cpu, instr.rm, true)))
        };
        return (base, writeback);
    }

    if matches!(
        instr.op,
        Opcode::SimdLd1
            | Opcode::SimdLd1Multi
            | Opcode::SimdLd2
            | Opcode::SimdLd3
            | Opcode::SimdLd4
            | Opcode::SimdLd1r
            | Opcode::SimdSt1Multi
            | Opcode::SimdSt4
    ) && instr.rm == 0xFF
    {
        return (base_addr(cpu, instr.rn), None);
    }

    if matches!(
        instr.op,
        Opcode::SimdLd1
            | Opcode::SimdLd1Multi
            | Opcode::SimdLd2
            | Opcode::SimdLd3
            | Opcode::SimdLd4
            | Opcode::SimdLd1r
            | Opcode::SimdSt1Multi
            | Opcode::SimdSt4
    ) && instr.rm != 0xFF
        && instr.rm != SIMD_MULTI_POST_INDEX
    {
        let base = base_addr(cpu, instr.rn);
        return (base, Some(base.wrapping_add(read_reg(cpu, instr.rm, true))));
    }

    if instr.rm == SIMD_MULTI_POST_INDEX {
        let base = base_addr(cpu, instr.rn);
        (base, Some(base.wrapping_add(instr.imm)))
    } else if instr.rm != 0xFF {
        let base = base_addr(cpu, instr.rn);
        let offset_val = read_reg(cpu, instr.rm, true);
        let extended = apply_extension(offset_val, instr.cond);
        let shift = if instr.imm == 1 {
            instr.size.trailing_zeros() as u8
        } else {
            0
        };
        (base.wrapping_add(extended << shift), None)
    } else {
        let base = base_addr(cpu, instr.rn);
        let (va, wb) = match instr.cond {
            1 => (base, Some(base.wrapping_add(instr.imm))),
            3 => {
                let b = base.wrapping_add(instr.imm);
                (b, Some(b))
            }
            _ => (base.wrapping_add(instr.imm), None),
        };
        (va, wb)
    }
}

pub(in crate::arm64::execute) fn base_addr(cpu: &Armv8Cpu, rn: u8) -> u64 {
    if rn == SP_REGISTER_INDEX {
        cpu.regs.sp
    } else {
        cpu.regs.x(rn)
    }
}

pub(in crate::arm64::execute) fn apply_extension(val: u64, option: u8) -> u64 {
    match option {
        0b010 => (val as u32) as u64,
        0b110 => (val as i32) as i64 as u64,
        0b011 => val,
        0b111 => val,
        _ => val,
    }
}

pub(in crate::arm64::execute) fn ldst_size(instr: &Instr) -> u8 {
    if instr.size != 0 {
        instr.size
    } else if instr.sf {
        8
    } else {
        4
    }
}

pub(in crate::arm64::execute) fn sign_extend_load(val: u64, size: u8, sf: bool) -> u64 {
    match (size, sf) {
        (1, false) => (val as i8 as i32) as u32 as u64,
        (1, true) => val as i8 as i64 as u64,
        (2, false) => (val as i16 as i32) as u32 as u64,
        (2, true) => val as i16 as i64 as u64,
        (4, true) => val as u32 as i32 as i64 as u64,
        _ => val,
    }
}

pub(in crate::arm64::execute) fn access_crosses_page(va: u64, size: u8) -> bool {
    (va & PAGE_OFFSET_MASK) + size as u64 > PAGE_SIZE
}

pub(in crate::arm64::execute) fn access_mask(size: u8) -> u64 {
    match size {
        1 => 0xFF,
        2 => 0xFFFF,
        4 => 0xFFFF_FFFF,
        8 => u64::MAX,
        _ => 0,
    }
}

pub(in crate::arm64::execute) fn signed_ext(val: u64, size: u8) -> i64 {
    match size {
        1 => val as i8 as i64,
        2 => val as i16 as i64,
        4 => val as u32 as i32 as i64,
        8 => val as i64,
        _ => val as i64,
    }
}
