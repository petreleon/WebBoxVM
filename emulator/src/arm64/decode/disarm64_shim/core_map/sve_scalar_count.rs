use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    match m {
        M::r#inch | M::r#incw | M::r#incd if is_vector_inc(raw) => {
            Some(Opcode::SveIncPatternVector)
        }
        M::r#dech | M::r#decw | M::r#decd if is_vector_dec(raw) => {
            Some(Opcode::SveDecPatternVector)
        }
        M::r#incb | M::r#inch | M::r#incw | M::r#incd if is_scalar_inc(raw) => {
            Some(Opcode::SveIncScalar)
        }
        M::r#decb | M::r#dech | M::r#decw | M::r#decd if is_scalar_dec(raw) => {
            Some(Opcode::SveDecScalar)
        }
        M::r#incp if (raw & 0xFF3F_FE00) == 0x252C_8000 => Some(Opcode::SveIncpVector),
        M::r#incp if (raw & 0xFF3F_FE00) == 0x252C_8800 => Some(Opcode::SveIncpScalar),
        M::r#decp if (raw & 0xFF3F_FE00) == 0x252D_8000 => Some(Opcode::SveDecpVector),
        M::r#decp if (raw & 0xFF3F_FE00) == 0x252D_8800 => Some(Opcode::SveDecpScalar),
        M::r#cntp if (raw & 0xFF3F_C200) == 0x2520_8000 => Some(Opcode::SveCntp),
        _ => None,
    }
}

fn is_scalar_inc(raw: u32) -> bool {
    matches!(
        raw & 0xFFF0_FC00,
        0x0430_E000 | 0x0470_E000 | 0x04B0_E000 | 0x04F0_E000
    )
}

fn is_scalar_dec(raw: u32) -> bool {
    matches!(
        raw & 0xFFF0_FC00,
        0x0430_E400 | 0x0470_E400 | 0x04B0_E400 | 0x04F0_E400
    )
}

fn is_vector_inc(raw: u32) -> bool {
    matches!(raw & 0xFFF0_FC00, 0x0470_C000 | 0x04B0_C000 | 0x04F0_C000)
}

fn is_vector_dec(raw: u32) -> bool {
    matches!(raw & 0xFFF0_FC00, 0x0470_C400 | 0x04B0_C400 | 0x04F0_C400)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_sve_scalar_inc_dec_mnemonics_by_encoding() {
        let cases = [
            (0x0431_E3F1, Opcode::SveIncScalar),
            (0x0471_E3F1, Opcode::SveIncScalar),
            (0x04B1_E3F1, Opcode::SveIncScalar),
            (0x04F1_E3F1, Opcode::SveIncScalar),
            (0x0431_E7F1, Opcode::SveDecScalar),
            (0x0471_E7F1, Opcode::SveDecScalar),
            (0x04B1_E7F1, Opcode::SveDecScalar),
            (0x04F1_E7F1, Opcode::SveDecScalar),
            (0x252C_88E1, Opcode::SveIncpScalar),
            (0x256C_88E1, Opcode::SveIncpScalar),
            (0x25AC_88E1, Opcode::SveIncpScalar),
            (0x25EC_88E1, Opcode::SveIncpScalar),
            (0x252D_88E1, Opcode::SveDecpScalar),
            (0x256D_88E1, Opcode::SveDecpScalar),
            (0x25AD_88E1, Opcode::SveDecpScalar),
            (0x25ED_88E1, Opcode::SveDecpScalar),
            (0x256C_80E1, Opcode::SveIncpVector),
            (0x25AC_80E1, Opcode::SveIncpVector),
            (0x25EC_80E1, Opcode::SveIncpVector),
            (0x256D_80E1, Opcode::SveDecpVector),
            (0x25AD_80E1, Opcode::SveDecpVector),
            (0x25ED_80E1, Opcode::SveDecpVector),
            (0x0471_C3F1, Opcode::SveIncPatternVector),
            (0x04B1_C3F1, Opcode::SveIncPatternVector),
            (0x04F1_C3F1, Opcode::SveIncPatternVector),
            (0x0471_C7F1, Opcode::SveDecPatternVector),
            (0x04B1_C7F1, Opcode::SveDecPatternVector),
            (0x04F1_C7F1, Opcode::SveDecPatternVector),
            (0x2520_8000, Opcode::SveCntp),
            (0x2560_8000, Opcode::SveCntp),
            (0x25A0_8000, Opcode::SveCntp),
            (0x25E0_8000, Opcode::SveCntp),
        ];

        for (raw, expected) in cases {
            let decoded =
                disarm64::decoder::decode(raw).expect("disarm64 should decode SVE count word");
            assert_eq!(map(raw, decoded.mnemonic), Some(expected));
        }
    }
}
