use crate::devices::virtio_gpu::protocol::{CTRL_HEADER_LEN, Rect, read_u32};

pub(super) fn read_create_2d(input: &[u8]) -> Option<(u32, u32, u32, u32)> {
    (input.len() >= 40).then_some((
        read_u32(input, 24)?,
        read_u32(input, 28)?,
        read_u32(input, 32)?,
        read_u32(input, 36)?,
    ))
}

pub(super) fn read_rect_resource(input: &[u8], minimum: usize) -> Option<(Rect, u32)> {
    (input.len() >= minimum)
        .then_some((Rect::decode(input, CTRL_HEADER_LEN)?, read_u32(input, 40)?))
}
