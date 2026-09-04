use super::material::VertexTransform;

const POSITION_BYTES: usize = 16;
const TEXTURED_BYTES: usize = 24;
const VERTEX_COLOR_BYTES: usize = 32;
const TEXTURE_COLOR_BYTES: usize = 40;

pub(super) fn apply(vertices: &mut Vec<u8>, transform: VertexTransform) -> bool {
    match transform {
        VertexTransform::Offset(offset) => translate(vertices, offset),
        VertexTransform::MultiplyColor(color) => multiply_color(vertices, color),
        VertexTransform::TextureColor(color) => add_constant_color(vertices, color),
    }
}

fn translate(vertices: &mut [u8], [x, y]: [f32; 2]) -> bool {
    if !vertices.len().is_multiple_of(POSITION_BYTES) {
        return false;
    }
    for vertex in vertices.chunks_exact_mut(POSITION_BYTES) {
        for (offset, delta) in [(0, x), (4, y)] {
            let Some(bytes) = vertex.get_mut(offset..offset + 4) else {
                return false;
            };
            let value = f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) + delta;
            if !value.is_finite() {
                return false;
            }
            bytes.copy_from_slice(&value.to_le_bytes());
        }
    }
    true
}

fn multiply_color(vertices: &mut [u8], color: [f32; 4]) -> bool {
    if !vertices.len().is_multiple_of(VERTEX_COLOR_BYTES) {
        return false;
    }
    for vertex in vertices.chunks_exact_mut(VERTEX_COLOR_BYTES) {
        for (factor, bytes) in color.into_iter().zip(vertex[16..].chunks_exact_mut(4)) {
            let value = f32::from_le_bytes(bytes.try_into().unwrap()) * factor;
            if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                return false;
            }
            bytes.copy_from_slice(&value.to_le_bytes());
        }
    }
    true
}

fn add_constant_color(vertices: &mut Vec<u8>, color: [f32; 4]) -> bool {
    if !vertices.len().is_multiple_of(TEXTURED_BYTES) {
        return false;
    }
    let Some(capacity) = vertices.len().checked_div(TEXTURED_BYTES).and_then(|count| count.checked_mul(TEXTURE_COLOR_BYTES)) else {
        return false;
    };
    let mut expanded = Vec::with_capacity(capacity);
    for vertex in vertices.chunks_exact(TEXTURED_BYTES) {
        expanded.extend_from_slice(&vertex[..POSITION_BYTES]);
        for value in color {
            expanded.extend_from_slice(&value.to_le_bytes());
        }
        expanded.extend_from_slice(&vertex[POSITION_BYTES..]);
    }
    *vertices = expanded;
    true
}
