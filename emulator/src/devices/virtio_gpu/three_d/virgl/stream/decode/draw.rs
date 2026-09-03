use super::super::super::draw::DrawCall;

const CMD_DRAW_VBO: u8 = 8;
const PIPE_PRIM_TRIANGLES: u32 = 4;

pub(super) fn decode(command: u8, object: u8, words: &[u32]) -> Option<DrawCall> {
    match (command, object, words) {
        (CMD_DRAW_VBO, 0, [start, 3, PIPE_PRIM_TRIANGLES, 0, 1, 0, 0, 0, _, _, _, 0]) => {
            Some(DrawCall { start: *start })
        }
        _ => None,
    }
}
