use super::Opcode;

impl Opcode {
    pub(in crate::arm64) fn is_fp_scalar(self) -> bool {
        super::fp_scalar_category::is_opcode(self)
    }

    pub(in crate::arm64) fn is_simd_data(self) -> bool {
        super::simd_data_category::is_opcode(self)
    }

    pub(in crate::arm64) fn is_simd_memory_or_crypto(self) -> bool {
        super::simd_memory_category::is_opcode(self)
    }

    pub(in crate::arm64) fn is_sve(self) -> bool {
        super::sve_category::is_opcode(self)
    }
}
