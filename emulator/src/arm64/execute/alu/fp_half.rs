pub(in crate::arm64::execute) fn f16_to_f32(bits: u16) -> f32 {
    let sign = ((bits as u32) & 0x8000) << 16;
    let exp = ((bits >> 10) & 0x1F) as i32;
    let frac = (bits & 0x03FF) as u32;

    let out = if exp == 0 {
        if frac == 0 {
            sign
        } else {
            let mut mant = frac;
            let mut unbiased = -14;
            while (mant & 0x0400) == 0 {
                mant <<= 1;
                unbiased -= 1;
            }
            mant &= 0x03FF;
            sign | (((unbiased + 127) as u32) << 23) | (mant << 13)
        }
    } else if exp == 0x1F {
        sign | 0x7F80_0000 | (frac << 13)
    } else {
        sign | (((exp - 15 + 127) as u32) << 23) | (frac << 13)
    };

    f32::from_bits(out)
}

pub(in crate::arm64::execute) fn f32_to_f16_bits(value: f32) -> u16 {
    let bits = value.to_bits();
    let sign = ((bits >> 16) & 0x8000) as u16;
    let exp = ((bits >> 23) & 0xFF) as i32;
    let frac = bits & 0x7F_FFFF;

    if exp == 0xFF {
        return f16_inf_nan(sign, frac >> 13);
    }
    if exp == 0 {
        return sign;
    }

    let half_exp = exp - 127 + 15;
    if half_exp >= 0x1F {
        return sign | 0x7C00;
    }
    if half_exp <= 0 {
        if half_exp < -10 {
            return sign;
        }
        let mant = frac | 0x80_0000;
        let rounded = round_shift_right_even_u32(mant, (14 - half_exp) as u32);
        return sign | rounded as u16;
    }

    let mut half_frac = round_shift_right_even_u32(frac, 13);
    let mut stored_exp = half_exp as u16;
    if half_frac == 0x0400 {
        stored_exp += 1;
        half_frac = 0;
        if stored_exp >= 0x1F {
            return sign | 0x7C00;
        }
    }

    sign | (stored_exp << 10) | half_frac as u16
}

pub(in crate::arm64::execute) fn f64_to_f16_bits(value: f64) -> u16 {
    let bits = value.to_bits();
    let sign = ((bits >> 48) & 0x8000) as u16;
    let exp = ((bits >> 52) & 0x7FF) as i32;
    let frac = bits & 0xF_FFFF_FFFF_FFFF;

    if exp == 0x7FF {
        return f16_inf_nan(sign, (frac >> 42) as u32);
    }
    if exp == 0 {
        return sign;
    }

    let half_exp = exp - 1023 + 15;
    if half_exp >= 0x1F {
        return sign | 0x7C00;
    }
    if half_exp <= 0 {
        if half_exp < -10 {
            return sign;
        }
        let mant = frac | (1u64 << 52);
        let rounded = round_shift_right_even_u64(mant, (43 - half_exp) as u32);
        return sign | rounded as u16;
    }

    let mut half_frac = round_shift_right_even_u64(frac, 42);
    let mut stored_exp = half_exp as u16;
    if half_frac == 0x0400 {
        stored_exp += 1;
        half_frac = 0;
        if stored_exp >= 0x1F {
            return sign | 0x7C00;
        }
    }

    sign | (stored_exp << 10) | half_frac as u16
}

fn f16_inf_nan(sign: u16, payload: u32) -> u16 {
    if payload == 0 {
        sign | 0x7C00
    } else {
        sign | 0x7C00 | ((payload as u16) | 0x0200)
    }
}

fn round_shift_right_even_u32(value: u32, shift: u32) -> u32 {
    if shift == 0 {
        return value;
    }
    let truncated = value >> shift;
    let halfway = 1u32 << (shift - 1);
    let remainder = value & ((1u32 << shift) - 1);
    if remainder > halfway || (remainder == halfway && (truncated & 1) != 0) {
        truncated + 1
    } else {
        truncated
    }
}

fn round_shift_right_even_u64(value: u64, shift: u32) -> u64 {
    if shift == 0 {
        return value;
    }
    let truncated = value >> shift;
    let halfway = 1u64 << (shift - 1);
    let remainder = value & ((1u64 << shift) - 1);
    if remainder > halfway || (remainder == halfway && (truncated & 1) != 0) {
        truncated + 1
    } else {
        truncated
    }
}
