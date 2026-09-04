use super::super::super::{BlendMode, DrawMaterial, DrawWork, ResidentTexture};

const HEADER_BYTES: usize = 48;
const DRAW_BYTES: usize = 52;
const TEXTURE_BYTES: usize = 16;

pub(super) fn packet(
    sequence: u32,
    width: u32,
    height: u32,
    clear: [f32; 4],
    works: &[DrawWork],
    depth: bool,
    resident: bool,
    predecessor: Option<u32>,
) -> Option<Vec<u8>> {
    let [work] = works else {
        return None;
    };
    let (kind, texture, stride) = source(&work.material)?;
    let bytes = usize::try_from(work.vertex_count)
        .ok()?
        .checked_mul(stride)?;
    if depth
        || !resident
        || predecessor.is_some()
        || work.blend != BlendMode::SourceOver
        || work.depth_resource.is_some()
        || work.depth_state.is_some()
        || !width_checked(width, height)
        || !source_valid(sequence, texture)
        || work.vertices.len() != bytes
    {
        return None;
    }
    let mut packet = Vec::with_capacity(
        HEADER_BYTES
            .checked_add(DRAW_BYTES)?
            .checked_add(TEXTURE_BYTES)?
            .checked_add(bytes)?,
    );
    packet.extend_from_slice(b"VGM1");
    for value in [12, sequence, width, height, 1, 2] {
        packet.extend_from_slice(&value.to_le_bytes());
    }
    floats(&mut packet, clear);
    packet.extend_from_slice(&0.0f32.to_le_bytes());
    packet.extend_from_slice(&kind.to_le_bytes());
    packet.extend_from_slice(&0u32.to_le_bytes());
    packet.extend_from_slice(&work.vertex_count.to_le_bytes());
    floats(&mut packet, work.viewport);
    for value in work.scissor.map(rect_words).unwrap_or([0; 4]) {
        packet.extend_from_slice(&value.to_le_bytes());
    }
    for value in [
        texture.sampler.wire(),
        texture.width,
        texture.height,
        texture.producer_sequence,
    ] {
        packet.extend_from_slice(&value.to_le_bytes());
    }
    packet.extend_from_slice(&work.vertices);
    Some(packet)
}

fn source(material: &DrawMaterial) -> Option<(u32, &ResidentTexture, usize)> {
    match material {
        DrawMaterial::ResidentTextured(texture) => Some((3, texture, 24)),
        DrawMaterial::ResidentTextureColor(texture) => Some((5, texture, 40)),
        _ => None,
    }
}

fn source_valid(sequence: u32, texture: &ResidentTexture) -> bool {
    sequence != texture.producer_sequence
        && texture.resource_id != 0
        && width_checked(texture.width, texture.height)
}

fn width_checked(width: u32, height: u32) -> bool {
    width != 0 && height != 0
}

fn rect_words(rect: crate::devices::virtio_gpu::protocol::Rect) -> [u32; 4] {
    [rect.x, rect.y, rect.width, rect.height]
}

fn floats(packet: &mut Vec<u8>, values: impl IntoIterator<Item = f32>) {
    for value in values {
        packet.extend_from_slice(&value.to_le_bytes());
    }
}
