use super::super::{BlendMode, DrawMaterial, DrawWork, TextureSnapshot, MAX_VIRGL_BATCH_DRAWS};

const HEADER_BYTES: usize = 48;
const DRAW_BYTES: usize = 52;

pub(in crate::devices::virtio_gpu::three_d) fn packet(
    sequence: u32,
    width: u32,
    height: u32,
    clear: [f32; 4],
    works: &[DrawWork],
    depth: bool,
    resident: bool,
    predecessor: Option<u32>,
) -> Option<Vec<u8>> {
    let blend = works.first()?.blend;
    let singleton = !depth && (resident || blend == BlendMode::Replace);
    if works.iter().any(|work| work.blend != blend)
        || (works.len() < 2 && !singleton)
        || works.len() > MAX_VIRGL_BATCH_DRAWS
    {
        return None;
    }
    let version = match (depth, resident, predecessor, blend) {
        (_, false, _, BlendMode::SourceOver) | (true, true, _, BlendMode::SourceOver) => 1,
        (false, true, None, BlendMode::SourceOver) => 2,
        (false, true, Some(_), BlendMode::SourceOver) => 3,
        (false, false, _, BlendMode::Replace) => 4,
        _ => return None,
    };
    let body = works.iter().try_fold(0usize, |total, work| {
        if !valid(work, depth) { return None; }
        let bytes = material_bytes(&work.material)?.checked_add(work.vertices.len())?;
        total.checked_add(DRAW_BYTES + bytes)
    })?;
    let header_bytes = HEADER_BYTES.checked_add(usize::from(version == 3) * 4)?;
    let mut packet = Vec::with_capacity(header_bytes.checked_add(body)?);
    packet.extend_from_slice(b"VGM1");
    let flags = if depth { 1 } else if resident { 2 } else { 0 };
    for value in [version, sequence, width, height, works.len() as u32, flags] {
        packet.extend_from_slice(&value.to_le_bytes());
    }
    floats(&mut packet, clear);
    packet.extend_from_slice(&(if depth { 1.0f32 } else { 0.0 }).to_le_bytes());
    if version == 3 { packet.extend_from_slice(&predecessor?.to_le_bytes()); }
    for work in works {
        encode_work(&mut packet, work)?;
    }
    Some(packet)
}

fn valid(work: &DrawWork, depth: bool) -> bool {
    let bytes = usize::try_from(work.vertex_count)
        .ok()
        .and_then(|count| count.checked_mul(stride(&work.material)))
        .is_some_and(|bytes| work.vertices.len() == bytes);
    bytes
        && work.depth_resource.is_some() == depth
        && work.depth_state.is_some() == depth
        && (!depth || !matches!(work.material, DrawMaterial::TexturedPair(_)))
}

fn encode_work(packet: &mut Vec<u8>, work: &DrawWork) -> Option<()> {
    let kind = kind(&work.material);
    packet.extend_from_slice(&kind.to_le_bytes());
    packet.extend_from_slice(&work.depth_state.map_or(0, |state| state.wire()).to_le_bytes());
    packet.extend_from_slice(&work.vertex_count.to_le_bytes());
    floats(packet, work.viewport);
    for value in work.scissor.map(rect_words).unwrap_or([0; 4]) {
        packet.extend_from_slice(&value.to_le_bytes());
    }
    match &work.material {
        DrawMaterial::Solid(color) => floats(packet, *color),
        DrawMaterial::VertexColor => {}
        DrawMaterial::Textured(source) | DrawMaterial::TextureColor(source) => texture(packet, source)?,
        DrawMaterial::TexturedPair(textures) => {
            texture(packet, &textures[0])?;
            texture(packet, &textures[1])?;
        }
    }
    packet.extend_from_slice(&work.vertices);
    Some(())
}

fn material_bytes(material: &DrawMaterial) -> Option<usize> {
    let vertices = match material {
        DrawMaterial::Solid(_) => 16,
        DrawMaterial::VertexColor => 0,
        DrawMaterial::Textured(texture) | DrawMaterial::TextureColor(texture) => texture_bytes(texture)?,
        DrawMaterial::TexturedPair(textures) => texture_bytes(&textures[0])?.checked_add(texture_bytes(&textures[1])?)?,
    };
    Some(vertices)
}

fn texture(packet: &mut Vec<u8>, texture: &TextureSnapshot) -> Option<()> {
    texture_bytes(texture)?;
    for value in [texture.sampler.wire(), texture.width, texture.height] {
        packet.extend_from_slice(&value.to_le_bytes());
    }
    packet.extend_from_slice(&texture.bgra);
    Some(())
}

fn texture_bytes(texture: &TextureSnapshot) -> Option<usize> {
    let bytes = usize::try_from(texture.width)
        .ok()?
        .checked_mul(texture.height as usize)?
        .checked_mul(4)?;
    (texture.bgra.len() == bytes).then_some(12usize.checked_add(bytes)?)
}

fn kind(material: &DrawMaterial) -> u32 {
    match material {
        DrawMaterial::Solid(_) => 1,
        DrawMaterial::VertexColor => 2,
        DrawMaterial::Textured(_) => 3,
        DrawMaterial::TexturedPair(_) => 4,
        DrawMaterial::TextureColor(_) => 5,
    }
}

fn stride(material: &DrawMaterial) -> usize {
    match material {
        DrawMaterial::Solid(_) => 16,
        DrawMaterial::Textured(_) | DrawMaterial::TexturedPair(_) => 24,
        DrawMaterial::VertexColor => 32,
        DrawMaterial::TextureColor(_) => 40,
    }
}

fn rect_words(rect: crate::devices::virtio_gpu::protocol::Rect) -> [u32; 4] {
    [rect.x, rect.y, rect.width, rect.height]
}

fn floats(packet: &mut Vec<u8>, values: impl IntoIterator<Item = f32>) {
    for value in values {
        packet.extend_from_slice(&value.to_le_bytes());
    }
}
