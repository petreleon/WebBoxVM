use super::super::{BlendMode, DrawMaterial, DrawWork, MAX_VIRGL_BATCH_DRAWS};
use crate::devices::virtio_gpu::three_d::virgl::{DepthCompare, DepthState};

const HEADER_BYTES: usize = 48;
const DRAW_STATE_BYTES: usize = 60;

pub(in crate::devices::virtio_gpu::three_d) fn packet(
    sequence: u32,
    width: u32,
    height: u32,
    clear: [f32; 4],
    works: &[DrawWork],
    resident: bool,
    resident_predecessor: Option<u32>,
) -> Option<Vec<u8>> {
    encode(sequence, width, height, clear, works, None, false, resident, resident_predecessor)
}

pub(in crate::devices::virtio_gpu::three_d) fn depth_packet(
    sequence: u32,
    width: u32,
    height: u32,
    clear: [f32; 4],
    works: &[DrawWork],
) -> Option<Vec<u8>> {
    let state = works.first()?.depth_state?;
    encode(sequence, width, height, clear, works, Some(state), works.iter().any(|work| work.depth_state != Some(state)), false, None)
}

fn encode(
    sequence: u32,
    width: u32,
    height: u32,
    clear: [f32; 4],
    works: &[DrawWork],
    depth: Option<DepthState>,
    mixed: bool,
    resident: bool,
    resident_predecessor: Option<u32>,
) -> Option<Vec<u8>> {
    let blend = works.first()?.blend;
    if works.iter().any(|work| work.blend != blend)
        || (!resident && !blend.is_replace() && works.len() < 2)
        || works.len() > MAX_VIRGL_BATCH_DRAWS
    {
        return None;
    }
    let read_only = works.iter().any(|work| work.depth_state.is_some_and(|state| !state.write));
    let version = match (depth, mixed, read_only, resident, resident_predecessor, blend) {
        (None, false, false, true, None, BlendMode::SourceOver) => 6,
        (None, false, false, true, Some(_), BlendMode::SourceOver) => 7,
        (None, false, false, false, _, BlendMode::SourceOver) => 1,
        (None, false, false, false, _, BlendMode::Replace) => 8,
        (None, false, false, false, _, BlendMode::ReplaceRgb) => 10,
        (Some(DepthState { compare: DepthCompare::Less, write: true }), false, false, false, _, BlendMode::SourceOver) => 2,
        (Some(DepthState { write: true, .. }), false, false, false, _, BlendMode::SourceOver) => 3,
        (Some(_), true, false, false, _, BlendMode::SourceOver) => 4,
        (Some(_), _, true, false, _, BlendMode::SourceOver) => 5,
        (Some(_), _, _, false, _, BlendMode::Replace) => 9,
        (Some(_), _, _, false, _, BlendMode::ReplaceRgb) => 11,
        _ => return None,
    };
    let per_draw_depth = matches!(version, 4 | 5 | 9 | 11);
    let depth_resource = works.first()?.depth_resource;
    let body = works.iter().try_fold(0usize, |total, work| {
        let bytes = usize::try_from(work.vertex_count).ok()?.checked_mul(16)?;
        (matches!(work.material, DrawMaterial::Solid(_))
            && work.depth_resource == depth_resource
            && work.depth_resource.is_some() == depth.is_some()
            && (per_draw_depth && work.depth_state.is_some() || !per_draw_depth && work.depth_state == depth)
            && work.vertices.len() == bytes)
            .then(|| total.checked_add(DRAW_STATE_BYTES + if per_draw_depth { 4 } else { 0 } + bytes))?
    })?;
    let header_bytes = HEADER_BYTES.checked_add(if version == 7 { 4 } else { 0 })?;
    let mut packet = Vec::with_capacity(header_bytes.checked_add(body)?);
    packet.extend_from_slice(b"VGB1");
    let flags = match depth { Some(state) if version == 3 => state.compare.wire(), _ if matches!(version, 6 | 7) => 1, _ => 0 };
    for value in [version, sequence, width, height, works.len() as u32, flags] {
        packet.extend_from_slice(&value.to_le_bytes());
    }
    floats(&mut packet, clear);
    packet.extend_from_slice(&(if depth.is_some() { 1.0f32 } else { 0.0 }).to_le_bytes());
    if version == 7 { packet.extend_from_slice(&resident_predecessor?.to_le_bytes()); }
    for work in works {
        let DrawMaterial::Solid(color) = work.material else { return None; };
        packet.extend_from_slice(&work.vertex_count.to_le_bytes());
        if version == 4 { packet.extend_from_slice(&work.depth_state?.compare.wire().to_le_bytes()); }
        if matches!(version, 5 | 9 | 11) { packet.extend_from_slice(&work.depth_state?.wire().to_le_bytes()); }
        floats(&mut packet, color.into_iter().chain(work.viewport));
        for value in work
            .scissor
            .map(|rect| [rect.x, rect.y, rect.width, rect.height])
            .unwrap_or([0; 4])
        {
            packet.extend_from_slice(&value.to_le_bytes());
        }
        packet.extend_from_slice(&work.vertices);
    }
    Some(packet)
}

fn floats(packet: &mut Vec<u8>, values: impl IntoIterator<Item = f32>) {
    for value in values {
        packet.extend_from_slice(&value.to_le_bytes());
    }
}
