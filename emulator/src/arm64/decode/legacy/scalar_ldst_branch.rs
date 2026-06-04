use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    let bits28_24 = (raw >> 24) & 0x1F;
    let bits28_23 = (raw >> 23) & 0x3F;
    let bits28_21 = (raw >> 21) & 0xFF;
    let bits31_26 = (raw >> 26) & 0x3F;
    let bits31_24 = (raw >> 24) & 0xFF;
    let bits31_24_masked_7e = ((raw >> 24) & 0x7E) as u32;

    if bits28_23 == 0b100010 {
        return DecodeStep::from_option(data_proc::decode_addsub_imm(raw));
    }
    if bits28_23 == 0b100101 {
        let opc = (raw >> 29) & 3;
        if opc == 0 {
            return DecodeStep::from_option(data_proc::decode_movn(raw));
        }
        if opc == 2 {
            return DecodeStep::from_option(data_proc::decode_movz(raw));
        }
        if opc == 3 {
            return DecodeStep::from_option(data_proc::decode_movk(raw));
        }
    }
    if bits28_23 == 0b100100 {
        return DecodeStep::from_option(data_proc::decode_logical_imm(raw));
    }
    if bits28_23 == 0b100111 {
        return DecodeStep::from_option(data_proc::decode_extract(raw));
    }
    if bits28_23 == 0b100110 {
        return DecodeStep::from_option(data_proc::decode_bitfield(raw));
    }
    if bits28_21 == 0b11010100 || bits28_21 == 0b11010010 {
        return DecodeStep::from_option(data_proc::decode_condsel(raw));
    }
    if bits28_21 == 0b11010000 {
        return DecodeStep::from_option(data_proc::decode_addsub_carry(raw));
    }
    if bits28_21 == 0b11010110 {
        let bit30 = (raw >> 30) & 1;
        if bit30 == 1 {
            return DecodeStep::from_option(data_proc::decode_dp_1src(raw));
        } else {
            return DecodeStep::from_option(data_proc::decode_dp_2src(raw));
        }
    }

    let dp_reg_pat = bits28_24;
    if dp_reg_pat == 0b11010 || dp_reg_pat == 0b01011 {
        return DecodeStep::from_option(data_proc::decode_dp_register(raw));
    }

    if bits28_24 == 0b01010 {
        return DecodeStep::from_option(data_proc::decode_logical_reg(raw));
    }

    if let Some(instr) = ldst::decode_lse_atomic(raw) {
        return DecodeStep::Hit(instr);
    }

    let ldst_family = (raw >> 24) & 0xF8;
    if ldst_family == 0x38 || ldst_family == 0x78 || ldst_family == 0xB8 || ldst_family == 0xF8 {
        if let Some(instr) = ldst::decode_ldrauth(raw) {
            return DecodeStep::Hit(instr);
        }
        if ((raw >> 22) & 0x3FF) == 0b1111100110 {
            return DecodeStep::from_option(system::decode_nop());
        }
        return DecodeStep::from_option(ldst::decode_ldst(raw));
    }

    if (raw & 0x3B00_0000) == 0x1800_0000 {
        return DecodeStep::from_option(ldst::decode_ldr_lit(raw));
    }

    let ldp_pat = (raw >> 24) & 0x1F;
    if ldp_pat & 0b11100 == 0b01000 && ldp_pat != 0b01011 {
        let is_excl = ((raw >> 29) & 1) == 0;
        if is_excl {
            return DecodeStep::from_option(ldst::decode_ldst_excl(raw));
        } else if is_ldst_pair(raw) {
            return DecodeStep::from_option(ldst::decode_ldst_pair(raw));
        }
    }
    if is_ldst_pair(raw) {
        return DecodeStep::from_option(ldst::decode_ldst_pair(raw));
    }

    if bits31_26 == 0b000101 {
        return DecodeStep::from_option(branch::decode_b(raw));
    }
    if bits31_26 == 0b100101 {
        return DecodeStep::from_option(branch::decode_bl(raw));
    }
    if bits31_24 == 0b01010100 {
        return DecodeStep::from_option(branch::decode_bcond(raw));
    }
    if bits31_24_masked_7e == 0b00110100 {
        return DecodeStep::from_option(branch::decode_cbz(raw));
    }
    if bits31_24_masked_7e == 0b00110110 {
        return DecodeStep::from_option(branch::decode_tbz(raw));
    }
    if bits31_24 == 0xD6 {
        return DecodeStep::from_option(branch::decode_branch_reg(raw));
    }
    if bits28_24 == 0b11011 {
        return DecodeStep::from_option(data_proc::decode_mul(raw));
    }
    DecodeStep::Miss
}
