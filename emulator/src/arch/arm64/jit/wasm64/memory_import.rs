use super::encoding::{encode_u32, encode_u64};
use super::opcodes::LIMITS_MEMORY64;

#[cfg(target_feature = "atomics")]
const LIMITS_HAS_MAXIMUM: u32 = 0x01;
#[cfg(target_feature = "atomics")]
const LIMITS_SHARED: u32 = 0x02;
#[cfg(target_feature = "atomics")]
const SHARED_MEMORY_MAX_PAGES: u64 = 65_536;

pub(super) fn append_memory_type(section: &mut Vec<u8>) {
    #[cfg(target_feature = "atomics")]
    {
        encode_u32(
            section,
            LIMITS_MEMORY64 | LIMITS_HAS_MAXIMUM | LIMITS_SHARED,
        );
        encode_u64(section, 0);
        encode_u64(section, SHARED_MEMORY_MAX_PAGES);
    }
    #[cfg(not(target_feature = "atomics"))]
    {
        encode_u32(section, LIMITS_MEMORY64);
        encode_u64(section, 0);
    }
}
