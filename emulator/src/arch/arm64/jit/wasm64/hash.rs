pub fn hash_raw_words(start_pa: u64, raw_words: impl IntoIterator<Item = u32>) -> u64 {
    raw_words
        .into_iter()
        .fold(hash_seed(start_pa), hash_raw_word)
}

pub(super) fn hash_seed(start_pa: u64) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    for byte in start_pa.to_le_bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

pub(super) fn hash_raw_word(mut hash: u64, raw: u32) -> u64 {
    for byte in raw.to_le_bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;
