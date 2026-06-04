use super::*;

pub(in crate::arm64::execute) enum ShiftDir {
    Left,
    Right,
    ArithRight,
    RotateRight,
}

pub(in crate::arm64::execute) fn exec_variable_shift(
    cpu: &mut Armv8Cpu,
    instr: Instr,
    dir: ShiftDir,
) {
    let n_val = read_reg(cpu, instr.rn, instr.sf);
    let m_val = read_reg(cpu, instr.rm, instr.sf);
    let val = if instr.sf {
        let shift = (m_val & 63) as u32;
        match dir {
            ShiftDir::Left => n_val << shift,
            ShiftDir::Right => n_val >> shift,
            ShiftDir::ArithRight => ((n_val as i64) >> shift) as u64,
            ShiftDir::RotateRight => n_val.rotate_right(shift),
        }
    } else {
        let shift = (m_val & 31) as u32;
        match dir {
            ShiftDir::Left => ((n_val as u32) << shift) as u64,
            ShiftDir::Right => ((n_val as u32) >> shift) as u64,
            ShiftDir::ArithRight => ((n_val as i32) >> shift) as u32 as u64,
            ShiftDir::RotateRight => (n_val as u32).rotate_right(shift) as u64,
        }
    };
    write_reg(cpu, instr.rd, val, instr.sf);
}

pub(in crate::arm64::execute) fn exec_extract(cpu: &mut Armv8Cpu, instr: Instr) {
    let size = if instr.sf { 64 } else { 32 };
    let lsb = (instr.imm as u32) & (size - 1);
    let low = read_reg(cpu, instr.rm, instr.sf);
    let high = read_reg(cpu, instr.rn, instr.sf);
    let val = if lsb == 0 {
        low
    } else if instr.sf {
        (low >> lsb) | (high << (64 - lsb))
    } else {
        (((low as u32) >> lsb) | ((high as u32) << (32 - lsb))) as u64
    };
    write_reg(cpu, instr.rd, val, instr.sf);
}

// ── Divide ──
