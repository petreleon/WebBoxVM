use super::*;

fn sign_bit(sf: bool) -> u32 {
    if sf { SIGN_BIT_64 } else { SIGN_BIT_32 }
}

pub(in crate::arch::arm64::execute) fn set_nz_flags(cpu: &mut Armv8Cpu, val: u64, sf: bool) {
    let sb = sign_bit(sf);
    let is_zero = if sf { val == 0 } else { (val as u32) == 0 };
    cpu.pstate
        .set_nzcv(((val >> sb) & 1) != 0, is_zero, false, false);
}

pub(in crate::arch::arm64::execute) fn add_flags(
    cpu: &mut Armv8Cpu,
    lhs: u64,
    rhs: u64,
    sf: bool,
) -> u64 {
    let val = lhs.wrapping_add(rhs);
    let sb = sign_bit(sf);
    let n = ((val >> sb) & 1) != 0;
    let z = if sf { val == 0 } else { (val as u32) == 0 };
    let c = if sf {
        val < lhs
    } else {
        (val as u32) < (lhs as u32)
    };
    let sign_mask = 1u64 << sb;
    let v = (lhs & sign_mask) == (rhs & sign_mask) && (lhs & sign_mask) != (val & sign_mask);
    cpu.pstate.set_nzcv(n, z, c, v);
    val
}

pub(in crate::arch::arm64::execute) fn sub_flags(
    cpu: &mut Armv8Cpu,
    lhs: u64,
    rhs: u64,
    sf: bool,
) -> u64 {
    let val = lhs.wrapping_sub(rhs);
    let sb = sign_bit(sf);
    let n = ((val >> sb) & 1) != 0;
    let z = if sf { val == 0 } else { (val as u32) == 0 };
    let c = if sf {
        lhs >= rhs
    } else {
        (lhs as u32) >= (rhs as u32)
    };
    let sign_mask = 1u64 << sb;
    let v = (lhs & sign_mask) != (rhs & sign_mask) && (lhs & sign_mask) != (val & sign_mask);
    cpu.pstate.set_nzcv(n, z, c, v);
    val
}

pub(in crate::arch::arm64::execute) fn exec_addsub_carry(cpu: &mut Armv8Cpu, instr: Instr) {
    let carry = u64::from(cpu.pstate.c());
    let mask = if instr.sf { u64::MAX } else { WORD_MASK };
    let lhs = read_reg(cpu, instr.rn, instr.sf) & mask;
    let rhs_raw = read_reg(cpu, instr.rm, instr.sf) & mask;
    let rhs = match instr.op {
        Opcode::Adc | Opcode::Adcs => rhs_raw,
        Opcode::Sbc | Opcode::Sbcs => !rhs_raw & mask,
        _ => unreachable!(),
    };
    let wide = lhs as u128 + rhs as u128 + carry as u128;
    let result = (wide & mask as u128) as u64;

    if matches!(instr.op, Opcode::Adcs | Opcode::Sbcs) {
        let sign_mask = 1u64 << sign_bit(instr.sf);
        let n = (result & sign_mask) != 0;
        let z = result == 0;
        let c = wide > mask as u128;
        let v = (lhs & sign_mask) == (rhs & sign_mask) && (lhs & sign_mask) != (result & sign_mask);
        cpu.pstate.set_nzcv(n, z, c, v);
    }

    write_reg(cpu, instr.rd, result, instr.sf);
}
