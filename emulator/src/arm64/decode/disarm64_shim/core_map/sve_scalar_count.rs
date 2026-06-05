use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    match m {
        M::r#incb | M::r#inch | M::r#incw | M::r#incd if is_scalar_inc(raw) => {
            Some(Opcode::SveIncScalar)
        }
        M::r#decb | M::r#dech | M::r#decw | M::r#decd if is_scalar_dec(raw) => {
            Some(Opcode::SveDecScalar)
        }
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
        ];

        for (raw, expected) in cases {
            let decoded =
                disarm64::decoder::decode(raw).expect("disarm64 should decode SVE count word");
            assert_eq!(map(raw, decoded.mnemonic), Some(expected));
        }
    }
}
