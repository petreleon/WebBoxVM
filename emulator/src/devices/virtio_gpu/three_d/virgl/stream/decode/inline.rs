use super::super::super::inline::InlineWrite;

const CMD_RESOURCE_INLINE_WRITE: u8 = 9;

pub(super) fn decode(command: u8, object: u8, words: &[u32]) -> Option<InlineWrite> {
    let [resource, 0, 0, 0, 0, offset, 0, 0, width, 1, 1, data @ ..] = words else {
        return None;
    };
    if command != CMD_RESOURCE_INLINE_WRITE || object != 0 || *resource == 0 || *width == 0 {
        return None;
    }
    let bytes = usize::try_from(*width).ok()?;
    if data.len() != bytes.checked_add(3)?.checked_div(4)? {
        return None;
    }
    let mut data: Vec<u8> = data.iter().flat_map(|word| word.to_le_bytes()).collect();
    if data[bytes..].iter().any(|byte| *byte != 0) {
        return None;
    }
    data.truncate(bytes);
    Some(InlineWrite {
        resource: *resource,
        offset: *offset,
        bytes: data,
    })
}
