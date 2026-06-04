use super::*;

pub(in crate::arm64::execute) fn extend_reg_val(
    cpu: &Armv8Cpu,
    rm: u8,
    option: u8,
    shift: u8,
    sf: bool,
) -> u64 {
    let mut val = read_reg(
        cpu,
        rm,
        if option == 3 || option == 7 {
            sf
        } else {
            option >= 2
        },
    );
    val = match option {
        0 => (val as u8) as u64,           // UXTB
        1 => (val as u16) as u64,          // UXTH
        2 => (val as u32) as u64,          // UXTW
        3 => val,                          // UXTX
        4 => ((val as i8) as i64) as u64,  // SXTB
        5 => ((val as i16) as i64) as u64, // SXTH
        6 => ((val as i32) as i64) as u64, // SXTW
        7 => val,                          // SXTX
        _ => val,
    };
    if sf {
        val << shift
    } else {
        ((val as u32) << shift) as u64
    }
}

pub(in crate::arm64::execute) fn shifted_reg_val(
    cpu: &Armv8Cpu,
    rm: u8,
    shift_type: u8,
    amount: u8,
    sf: bool,
) -> u64 {
    let val = read_reg(cpu, rm, sf);
    let amount = amount as u32;
    if amount == 0 {
        return val;
    }
    match shift_type {
        0 => {
            if sf {
                val << amount
            } else {
                ((val as u32) << amount) as u64
            }
        }
        1 => {
            if sf {
                val >> amount
            } else {
                ((val as u32) >> amount) as u64
            }
        }
        2 => {
            if sf {
                ((val as i64) >> amount) as u64
            } else {
                (((val as u32) as i32) >> amount) as u64
            }
        }
        3 => {
            if sf {
                val.rotate_right(amount)
            } else {
                (val as u32).rotate_right(amount) as u64
            }
        }
        _ => val,
    }
}

// ── Logical register ──
