use super::*;

pub(in crate::arm64::execute) fn exec_madd(cpu: &mut Armv8Cpu, instr: Instr) {
    let sf_src = instr.size == 0 && instr.sf;
    let n = read_reg(cpu, instr.rn, sf_src);
    let m = read_reg(cpu, instr.rm, sf_src);
    let a = read_reg(cpu, instr.cond, instr.sf);
    let val = match instr.size {
        0 => {
            if instr.sf {
                a.wrapping_add(n.wrapping_mul(m))
            } else {
                ((a as u32).wrapping_add((n as u32).wrapping_mul(m as u32))) as u64
            }
        }
        1 => a.wrapping_add((n as u32 as u64).wrapping_mul(m as u32 as u64)),
        2 => {
            a.wrapping_add(((n as u32 as i32) as i64).wrapping_mul((m as u32 as i32) as i64) as u64)
        }
        _ => return,
    };
    write_reg(cpu, instr.rd, val, instr.sf);
}

pub(in crate::arm64::execute) fn exec_msub(cpu: &mut Armv8Cpu, instr: Instr) {
    let sf_src = instr.size == 0 && instr.sf;
    let n = read_reg(cpu, instr.rn, sf_src);
    let m = read_reg(cpu, instr.rm, sf_src);
    let a = read_reg(cpu, instr.cond, instr.sf);
    let val = match instr.size {
        0 => {
            if instr.sf {
                a.wrapping_sub(n.wrapping_mul(m))
            } else {
                ((a as u32).wrapping_sub((n as u32).wrapping_mul(m as u32))) as u64
            }
        }
        1 => a.wrapping_sub((n as u32 as u64).wrapping_mul(m as u32 as u64)),
        2 => {
            a.wrapping_sub(((n as u32 as i32) as i64).wrapping_mul((m as u32 as i32) as i64) as u64)
        }
        _ => return,
    };
    write_reg(cpu, instr.rd, val, instr.sf);
}

// ── Variable shift ──

pub(in crate::arm64::execute) fn exec_div(cpu: &mut Armv8Cpu, instr: Instr, signed: bool) {
    let n = read_reg(cpu, instr.rn, instr.sf);
    let m = read_reg(cpu, instr.rm, instr.sf);
    let val = if m == 0 {
        0
    } else if instr.sf {
        if signed {
            (n as i64).checked_div(m as i64).unwrap_or(n as i64) as u64
        } else {
            n / m
        }
    } else if signed {
        (n as i32).checked_div(m as i32).unwrap_or(n as i32) as u32 as u64
    } else {
        ((n as u32) / (m as u32)) as u64
    };
    write_reg(cpu, instr.rd, val, instr.sf);
}

// ── Reverse bits/bytes ──
