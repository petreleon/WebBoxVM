use super::super::super::draw::{DrawCall, MAX_VIRGL_DRAW_VERTICES, TRIANGLE_VERTICES};

const CMD_DRAW_VBO: u8 = 8;
const PIPE_PRIM_TRIANGLES: u32 = 4;

pub(super) fn decode(command: u8, object: u8, words: &[u32]) -> Option<DrawCall> {
    match (command, object, words) {
        (
            CMD_DRAW_VBO,
            0,
            [
                start,
                count,
                PIPE_PRIM_TRIANGLES,
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
        ) if *indexed <= 1 && valid_count(*count) => Some(DrawCall {
            start: *start,
            count: *count,
            indexed: *indexed != 0,
        }),
        _ => None,
    }
}

fn valid_count(count: u32) -> bool {
    (TRIANGLE_VERTICES..=MAX_VIRGL_DRAW_VERTICES).contains(&count)
        && count.is_multiple_of(TRIANGLE_VERTICES)
}
