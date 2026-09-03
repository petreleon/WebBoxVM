use super::{DrawMaterial, DrawWork, TRIANGLE_VERTICES};

pub(in crate::devices::virtio_gpu::three_d) fn packet(
    sequence: u32,
    width: u32,
    height: u32,
    clear: [f32; 4],
    work: &DrawWork,
) -> Vec<u8> {
    match &work.material {
        DrawMaterial::Solid(color) => solid(sequence, width, height, clear, work, *color),
        DrawMaterial::Textured(texture) => textured(sequence, width, height, clear, work, texture),
    }
}

fn solid(
    sequence: u32,
    width: u32,
    height: u32,
    clear: [f32; 4],
    work: &DrawWork,
    color: [f32; 4],
) -> Vec<u8> {
    let mut packet = header(2, sequence, width, height);
    floats(&mut packet, clear.into_iter().chain(color));
    packet.extend_from_slice(&work.vertices);
    state(&mut packet, work);
    packet
}

fn textured(
    sequence: u32,
    width: u32,
    height: u32,
    clear: [f32; 4],
    work: &DrawWork,
    texture: &super::TextureSnapshot,
) -> Vec<u8> {
    let mut packet = header(3, sequence, width, height);
    floats(&mut packet, clear.into_iter().chain([0.0; 4]));
    packet.extend_from_slice(&work.vertices);
    state(&mut packet, work);
    for value in [texture.width, texture.height] {
        packet.extend_from_slice(&value.to_le_bytes());
    }
    packet.extend_from_slice(&texture.bgra);
    packet
}

fn header(version: u32, sequence: u32, width: u32, height: u32) -> Vec<u8> {
    let mut packet = b"VGD1".to_vec();
    for value in [version, sequence, width, height, TRIANGLE_VERTICES] {
        packet.extend_from_slice(&value.to_le_bytes());
    }
    packet
}

fn floats(values: &mut Vec<u8>, source: impl Iterator<Item = f32>) {
    for value in source {
        values.extend_from_slice(&value.to_le_bytes());
    }
}

fn state(packet: &mut Vec<u8>, work: &DrawWork) {
    floats(packet, work.viewport.into_iter());
    for value in work
        .scissor
        .map(|rect| [rect.x, rect.y, rect.width, rect.height])
        .unwrap_or([0; 4])
    {
        packet.extend_from_slice(&value.to_le_bytes());
    }
}
