pub(in crate::arch::arm64::execute) fn predicate_to_bytes(pred: [u64; 4]) -> [u8; 32] {
    let mut bytes = [0; 32];
    for (word_index, word) in pred.iter().enumerate() {
        bytes[word_index * 8..word_index * 8 + 8].copy_from_slice(&word.to_le_bytes());
    }
    bytes
}

pub(in crate::arch::arm64::execute) fn predicate_from_bytes(bytes: &[u8; 256]) -> [u64; 4] {
    let mut pred = [0; 4];
    for (word_index, word) in pred.iter_mut().enumerate() {
        let offset = word_index * 8;
        let mut word_bytes = [0; 8];
        word_bytes.copy_from_slice(&bytes[offset..offset + 8]);
        *word = u64::from_le_bytes(word_bytes);
    }
    pred
}
