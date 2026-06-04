use super::*;

pub(in crate::arm64::execute) fn exec_bitfield(cpu: &mut Armv8Cpu, instr: Instr) {
    let size = if instr.sf { 64 } else { 32 };
    let r = instr.rm as u32;
    let s = instr.imm as u32;
    let src = read_reg(cpu, instr.rn, instr.sf);

    let val = match instr.op {
        Opcode::Ubfm => bitfield_extract(src, r, s, size, false),
        Opcode::Sbfm => bitfield_extract(src, r, s, size, true),
        Opcode::Bfm => {
            let dst = read_reg(cpu, instr.rd, instr.sf);
            bitfield_insert(dst, src, r, s, size)
        }
        _ => unreachable!(),
    };
    write_reg(cpu, instr.rd, val, instr.sf);
}

fn bitfield_extract(src: u64, r: u32, s: u32, size: u32, signed: bool) -> u64 {
    let result = if s >= r {
        let len = s - r + 1;
        let mask = bitmask(len);
        let extracted = (src >> r) & mask;
        if signed {
            sign_extend(extracted, s - r, size)
        } else {
            extracted
        }
    } else {
        let len = s + 1;
        let mask = bitmask(len);
        let shift = size - r;
        let extracted = (src & mask) << shift;
        if signed {
            sign_extend(extracted, shift + s, size)
        } else {
            extracted
        }
    };
    word_truncate(result, size)
}

fn bitfield_insert(dst: u64, src: u64, r: u32, s: u32, size: u32) -> u64 {
    let result = if s >= r {
        let len = s - r + 1;
        let mask = bitmask(len);
        let extracted = (src >> r) & mask;
        (dst & !mask) | extracted
    } else {
        let len = s + 1;
        let mask = bitmask(len);
        let shift = size - r;
        let dst_mask = !(mask << shift);
        (dst & dst_mask) | ((src & mask) << shift)
    };
    word_truncate(result, size)
}

fn bitmask(len: u32) -> u64 {
    if len >= 64 { !0 } else { (1u64 << len) - 1 }
}

fn sign_extend(val: u64, sign_bit: u32, size: u32) -> u64 {
    if sign_bit < 63 && (val & (1u64 << sign_bit)) != 0 {
        let extend_mask = !((1u64 << (sign_bit + 1)) - 1);
        val | (extend_mask & full_width_mask(size))
    } else {
        val
    }
}

fn word_truncate(val: u64, size: u32) -> u64 {
    if size == 64 { val } else { val & WORD_MASK }
}

fn full_width_mask(size: u32) -> u64 {
    if size == 64 { !0 } else { WORD_MASK }
}

// ── Conditional compare ──
