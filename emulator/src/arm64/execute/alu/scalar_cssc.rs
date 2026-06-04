use super::*;

pub(in crate::arm64::execute) fn exec_abs(cpu: &mut Armv8Cpu, instr: Instr) {
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

pub(in crate::arm64::execute) fn exec_ctz(cpu: &mut Armv8Cpu, instr: Instr) {
    let count = if instr.sf {
        read_reg(cpu, instr.rn, true).trailing_zeros()
    } else {
        (read_reg(cpu, instr.rn, false) as u32).trailing_zeros()
    };
    write_reg(cpu, instr.rd, count as u64, instr.sf);
}

pub(in crate::arm64::execute) fn exec_cnt(cpu: &mut Armv8Cpu, instr: Instr) {
    let count = if instr.sf {
        read_reg(cpu, instr.rn, true).count_ones()
    } else {
        (read_reg(cpu, instr.rn, false) as u32).count_ones()
    };
    write_reg(cpu, instr.rd, count as u64, instr.sf);
}
