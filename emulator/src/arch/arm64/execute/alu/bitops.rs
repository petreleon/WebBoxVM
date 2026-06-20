use super::*;

pub(in crate::arch::arm64::execute) fn exec_rev(cpu: &mut Armv8Cpu, instr: Instr) {
    if instr.sf {
        write_reg(
            cpu,
            instr.rd,
            read_reg(cpu, instr.rn, true).swap_bytes(),
            true,
        );
    } else {
        write_reg(
            cpu,
            instr.rd,
            (read_reg(cpu, instr.rn, false) as u32).swap_bytes() as u64,
            false,
        );
    }
}

pub(in crate::arch::arm64::execute) fn exec_rev16(cpu: &mut Armv8Cpu, instr: Instr) {
    const MASK_EVEN: u64 = 0xFF00_FF00_FF00_FF00;
    const MASK_ODD: u64 = 0x00FF_00FF_00FF_00FF;
    if instr.sf {
        let val = read_reg(cpu, instr.rn, true);
        write_reg(
            cpu,
            instr.rd,
            ((val & MASK_EVEN) >> 8) | ((val & MASK_ODD) << 8),
            true,
        );
    } else {
        let val = read_reg(cpu, instr.rn, false) as u32;
        write_reg(
            cpu,
            instr.rd,
            (((val & 0xFF00_FF00) >> 8) | ((val & 0x00FF_00FF) << 8)) as u64,
            false,
        );
    }
}

pub(in crate::arch::arm64::execute) fn exec_rbit(cpu: &mut Armv8Cpu, instr: Instr) {
    if instr.sf {
        write_reg(
            cpu,
            instr.rd,
            read_reg(cpu, instr.rn, true).reverse_bits(),
            true,
        );
    } else {
        write_reg(
            cpu,
            instr.rd,
            (read_reg(cpu, instr.rn, false) as u32).reverse_bits() as u64,
            false,
        );
    }
}

pub(in crate::arch::arm64::execute) fn exec_clz(cpu: &mut Armv8Cpu, instr: Instr) {
    if instr.sf {
        write_reg(
            cpu,
            instr.rd,
            read_reg(cpu, instr.rn, true).leading_zeros() as u64,
            true,
        );
    } else {
        write_reg(
            cpu,
            instr.rd,
            (read_reg(cpu, instr.rn, false) as u32).leading_zeros() as u64,
            false,
        );
    }
}

pub(in crate::arch::arm64::execute) fn exec_cls(cpu: &mut Armv8Cpu, instr: Instr) {
    let count = if instr.sf {
        let value = read_reg(cpu, instr.rn, true);
        leading_sign_bits_64(value)
    } else {
        let value = read_reg(cpu, instr.rn, false) as u32;
        leading_sign_bits_32(value)
    };
    write_reg(cpu, instr.rd, count as u64, instr.sf);
}

pub(in crate::arch::arm64::execute) fn exec_crc32(cpu: &mut Armv8Cpu, instr: Instr) {
    exec_crc(cpu, instr, 0xedb8_8320);
}

pub(in crate::arch::arm64::execute) fn exec_crc32c(cpu: &mut Armv8Cpu, instr: Instr) {
    exec_crc(cpu, instr, 0x82f6_3b78);
}

fn exec_crc(cpu: &mut Armv8Cpu, instr: Instr, poly: u32) {
    let mut crc = read_reg(cpu, instr.rn, false) as u32;
    let value = read_reg(cpu, instr.rm, instr.size == 8);

    for byte_index in 0..instr.size {
        crc ^= ((value >> (byte_index * 8)) & 0xff) as u32;
        for _ in 0..8 {
            crc = if (crc & 1) != 0 {
                (crc >> 1) ^ poly
            } else {
                crc >> 1
            };
        }
    }

    write_reg(cpu, instr.rd, crc as u64, false);
}

fn leading_sign_bits_64(value: u64) -> u32 {
    let count = if (value >> 63) == 0 {
        value.leading_zeros()
    } else {
        value.leading_ones()
    };
    count - 1
}

fn leading_sign_bits_32(value: u32) -> u32 {
    let count = if (value >> 31) == 0 {
        value.leading_zeros()
    } else {
        value.leading_ones()
    };
    count - 1
}
