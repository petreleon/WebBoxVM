pub(super) fn read_config_u64(value: u64, offset: u64, size: u8) -> Option<u64> {
    let bytes = value.to_le_bytes();
    let offset = offset as usize;
    let len = match size {
        1 | 2 | 4 | 8 => Some(size as usize),
        _ => None,
    }?;
    let end = offset.checked_add(len)?;
    if end > bytes.len() {
        return None;
    }

    let mut out = [0u8; 8];
    out[..len].copy_from_slice(&bytes[offset..end]);
    Some(u64::from_le_bytes(out))
}

pub(super) fn mask_read(value: u64, size: u8) -> Option<u64> {
    match size {
        1 => Some(value & 0xff),
        2 => Some(value & 0xffff),
        4 => Some(value & 0xffff_ffff),
        8 => Some(value),
        _ => None,
    }
}
