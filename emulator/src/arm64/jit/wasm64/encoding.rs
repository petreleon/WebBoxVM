pub(super) fn encode_name(dst: &mut Vec<u8>, name: &str) {
    encode_u32(dst, name.len() as u32);
    dst.extend_from_slice(name.as_bytes());
}

pub(super) fn encode_u32(dst: &mut Vec<u8>, mut value: u32) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        dst.push(byte);
        if value == 0 {
            break;
        }
    }
}

pub(super) fn encode_u64(dst: &mut Vec<u8>, mut value: u64) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        dst.push(byte);
        if value == 0 {
            break;
        }
    }
}

pub(super) fn encode_i64(dst: &mut Vec<u8>, mut value: i64) {
    loop {
        let byte = (value as u8) & 0x7f;
        value >>= 7;
        let done = (value == 0 && (byte & 0x40) == 0) || (value == -1 && (byte & 0x40) != 0);
        dst.push(if done { byte } else { byte | 0x80 });
        if done {
            break;
        }
    }
}
