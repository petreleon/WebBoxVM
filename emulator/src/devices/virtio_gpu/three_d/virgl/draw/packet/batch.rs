use super::super::{DepthCompare, DrawMaterial, DrawWork, MAX_VIRGL_BATCH_DRAWS};

const HEADER_BYTES: usize = 48;
const DRAW_STATE_BYTES: usize = 60;

pub(in crate::devices::virtio_gpu::three_d) fn packet(
    sequence: u32,
    width: u32,
    height: u32,
    clear: [f32; 4],
    works: &[DrawWork],
) -> Option<Vec<u8>> {
    encode(sequence, width, height, clear, works, None)
}

pub(in crate::devices::virtio_gpu::three_d) fn depth_packet(
    sequence: u32,
    width: u32,
    height: u32,
    clear: [f32; 4],
    works: &[DrawWork],
) -> Option<Vec<u8>> {
    encode(sequence, width, height, clear, works, Some(works.first()?.depth_compare?))
}

fn encode(
    sequence: u32,
    width: u32,
    height: u32,
    clear: [f32; 4],
    works: &[DrawWork],
    depth: Option<DepthCompare>,
) -> Option<Vec<u8>> {
    if works.len() < 2 || works.len() > MAX_VIRGL_BATCH_DRAWS {
        return None;
    }
    let version = match depth {
        None => 1,
        Some(DepthCompare::Less) => 2,
        Some(_) => 3,
    };
    let body = works.iter().try_fold(0usize, |total, work| {
        let bytes = usize::try_from(work.vertex_count).ok()?.checked_mul(16)?;
        (matches!(work.material, DrawMaterial::Solid(_))
            && work.depth_resource.is_some() == depth.is_some()
            && work.depth_compare == depth
            && work.vertices.len() == bytes)
            .then(|| total.checked_add(DRAW_STATE_BYTES + bytes))?
    })?;
    let mut packet = Vec::with_capacity(HEADER_BYTES.checked_add(body)?);
    packet.extend_from_slice(b"VGB1");
    let flags = match depth { Some(compare) if version == 3 => compare.wire(), _ => 0 };
    for value in [version, sequence, width, height, works.len() as u32, flags] {
        packet.extend_from_slice(&value.to_le_bytes());
    }
    floats(&mut packet, clear);
    packet.extend_from_slice(&(if depth.is_some() { 1.0f32 } else { 0.0 }).to_le_bytes());
    for work in works {
        let DrawMaterial::Solid(color) = work.material else { return None; };
        packet.extend_from_slice(&work.vertex_count.to_le_bytes());
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
