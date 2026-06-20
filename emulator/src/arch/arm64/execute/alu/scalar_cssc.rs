use super::*;

const IMM_RM: u8 = 0xFF;

pub(in crate::arch::arm64::execute) fn exec_abs(cpu: &mut Armv8Cpu, instr: Instr) {
    if instr.sf {
        let value = read_reg(cpu, instr.rn, true);
        let result = if (value >> 63) != 0 {
            value.wrapping_neg()
        } else {
            value
        };
        write_reg(cpu, instr.rd, result, true);
    } else {
        let value = read_reg(cpu, instr.rn, false) as u32;
        let result = if (value >> 31) != 0 {
            value.wrapping_neg()
        } else {
            value
        };
        write_reg(cpu, instr.rd, result as u64, false);
    }
}

pub(in crate::arch::arm64::execute) fn exec_smax(cpu: &mut Armv8Cpu, instr: Instr) {
    exec_signed_minmax(cpu, instr, true);
}

pub(in crate::arch::arm64::execute) fn exec_smin(cpu: &mut Armv8Cpu, instr: Instr) {
    exec_signed_minmax(cpu, instr, false);
}

pub(in crate::arch::arm64::execute) fn exec_umax(cpu: &mut Armv8Cpu, instr: Instr) {
    exec_unsigned_minmax(cpu, instr, true);
}

pub(in crate::arch::arm64::execute) fn exec_umin(cpu: &mut Armv8Cpu, instr: Instr) {
    exec_unsigned_minmax(cpu, instr, false);
}

pub(in crate::arch::arm64::execute) fn exec_ctz(cpu: &mut Armv8Cpu, instr: Instr) {
    let count = if instr.sf {
        read_reg(cpu, instr.rn, true).trailing_zeros()
    } else {
        (read_reg(cpu, instr.rn, false) as u32).trailing_zeros()
    };
    write_reg(cpu, instr.rd, count as u64, instr.sf);
}

pub(in crate::arch::arm64::execute) fn exec_cnt(cpu: &mut Armv8Cpu, instr: Instr) {
    let count = if instr.sf {
        read_reg(cpu, instr.rn, true).count_ones()
    } else {
        (read_reg(cpu, instr.rn, false) as u32).count_ones()
    };
    write_reg(cpu, instr.rd, count as u64, instr.sf);
}

fn exec_signed_minmax(cpu: &mut Armv8Cpu, instr: Instr, maximum: bool) {
    let lhs = signed_operand(cpu, instr.rn, instr.sf);
    let rhs = if instr.rm == IMM_RM {
        instr.imm as i64
    } else {
        signed_operand(cpu, instr.rm, instr.sf)
    };
    let result = if maximum { lhs.max(rhs) } else { lhs.min(rhs) };
    write_reg(cpu, instr.rd, result as u64, instr.sf);
}

fn exec_unsigned_minmax(cpu: &mut Armv8Cpu, instr: Instr, maximum: bool) {
    let lhs = unsigned_operand(cpu, instr.rn, instr.sf);
    let rhs = if instr.rm == IMM_RM {
        instr.imm
    } else {
        unsigned_operand(cpu, instr.rm, instr.sf)
    };
    let result = if maximum { lhs.max(rhs) } else { lhs.min(rhs) };
    write_reg(cpu, instr.rd, result, instr.sf);
}

fn signed_operand(cpu: &Armv8Cpu, reg: u8, sf: bool) -> i64 {
    if sf {
        read_reg(cpu, reg, true) as i64
    } else {
        (read_reg(cpu, reg, false) as u32 as i32) as i64
    }
}

fn unsigned_operand(cpu: &Armv8Cpu, reg: u8, sf: bool) -> u64 {
    if sf {
        read_reg(cpu, reg, true)
    } else {
        read_reg(cpu, reg, false) as u32 as u64
    }
}
