use super::super::super::draw::{DrawCall, Primitive};

const CMD_DRAW_VBO: u8 = 8;

pub(super) fn decode(command: u8, object: u8, words: &[u32]) -> Option<DrawCall> {
    match (command, object, words) {
        (
            CMD_DRAW_VBO,
            0,
            [
                start,
                count,
                primitive,
                indexed,
                1,
                0,
                0,
                0,
                _,
                _,
                _,
                0,
            ],
        ) => {
            let primitive = Primitive::from_wire(*primitive)?;
            (*indexed <= 1 && primitive.valid_count(*count)).then_some(DrawCall {
                start: *start,
                count: *count,
                primitive,
                indexed: *indexed != 0,
            })
        }
        _ => None,
    }
}
