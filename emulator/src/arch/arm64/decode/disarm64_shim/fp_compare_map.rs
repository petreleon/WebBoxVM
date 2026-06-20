use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    Some(match m {
        M::r#facge if sve_fp_size_valid(raw) && (raw & 0xFF20_E010) == 0x6500_C010 => {
            Opcode::SveFpFacge
        }
        M::r#facge if simd_fp_compare(raw, 0x2E20_EC00, 0x7E20_EC00) => Opcode::SimdFpFacgeVec,
        M::r#facgt if sve_fp_size_valid(raw) && (raw & 0xFF20_E010) == 0x6500_E010 => {
            Opcode::SveFpFacgt
        }
        M::r#facgt if simd_fp_compare(raw, 0x2EA0_EC00, 0x7EA0_EC00) => Opcode::SimdFpFacgtVec,
        M::r#fcmeq if sve_fp_size_valid(raw) && sve_fp_cmp(raw, 0x6500_6000, 0x6512_2000) => {
            Opcode::SveFpFcmeq
        }
        M::r#fcmeq if simd_fp_compare(raw, 0x0E20_E400, 0x5E20_E400) => Opcode::SimdFpFcmeqVec,
        M::r#fcmeq if simd_fp_zero_compare(raw, 0x0EA0_D800, 0x5EA0_D800) => {
            Opcode::SimdFpFcmeqZero
        }
        M::r#fcmge if sve_fp_size_valid(raw) && sve_fp_cmp(raw, 0x6500_4000, 0x6510_2000) => {
            Opcode::SveFpFcmge
        }
        M::r#fcmge if simd_fp_zero_compare(raw, 0x2EA0_C800, 0x7EA0_C800) => {
            Opcode::SimdFpFcmgeZero
        }
        M::r#fcmge if simd_fp_compare(raw, 0x2E20_E400, 0x7E20_E400) => Opcode::SimdFpFcmgeVec,
        M::r#fcmgt if sve_fp_size_valid(raw) && sve_fp_cmp(raw, 0x6500_4010, 0x6510_2010) => {
            Opcode::SveFpFcmgt
        }
        M::r#fcmgt if simd_fp_zero_compare(raw, 0x0EA0_C800, 0x5EA0_C800) => {
            Opcode::SimdFpFcmgtZero
        }
        M::r#fcmgt if simd_fp_compare(raw, 0x2EA0_E400, 0x7EA0_E400) => Opcode::SimdFpFcmgtVec,
        M::r#fcmne if sve_fp_size_valid(raw) && sve_fp_cmp(raw, 0x6500_6010, 0x6513_2000) => {
            Opcode::SveFpFcmne
        }
        M::r#fcmle if sve_fp_size_valid(raw) && (raw & 0xFF3F_E010) == 0x6511_2010 => {
            Opcode::SveFpFcmle
        }
        M::r#fcmle if simd_fp_zero_compare(raw, 0x2EA0_D800, 0x7EA0_D800) => {
            Opcode::SimdFpFcmleZero
        }
        M::r#fcmlt if sve_fp_size_valid(raw) && (raw & 0xFF3F_E010) == 0x6511_2000 => {
            Opcode::SveFpFcmlt
        }
        M::r#fcmlt if simd_fp_zero_compare(raw, 0x0EA0_E800, 0x5EA0_E800) => {
            Opcode::SimdFpFcmltZero
        }
        _ => return None,
    })
}
