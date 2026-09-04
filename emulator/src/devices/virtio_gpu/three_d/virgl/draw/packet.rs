mod batch;
mod material_batch;

use super::super::SamplerConfig;
use super::super::{DepthCompare, DepthState};
use super::{DrawMaterial, DrawWork};

pub(in crate::devices::virtio_gpu::three_d) use batch::{packet as batch_packet, depth_packet as depth_batch_packet};
pub(in crate::devices::virtio_gpu::three_d) use material_batch::packet as material_batch_packet;

pub(in crate::devices::virtio_gpu::three_d) fn packet(
    sequence: u32,
    width: u32,
    height: u32,
    clear: [f32; 4],
    work: &DrawWork,
) -> Vec<u8> {
    match &work.material {
        DrawMaterial::Solid(color) => solid(sequence, width, height, clear, work, *color),
        DrawMaterial::VertexColor => vertex_color(sequence, width, height, clear, work),
        DrawMaterial::Textured(texture) => textured(sequence, width, height, clear, work, texture),
        DrawMaterial::TexturedPair(textures) => textured_pair(sequence, width, height, clear, work, textures),
        DrawMaterial::TextureColor(texture) => texture_color(sequence, width, height, clear, work, texture),
    }
}

fn vertex_color(
    sequence: u32,
    width: u32,
    height: u32,
    clear: [f32; 4],
    work: &DrawWork,
) -> Vec<u8> {
    let version = if work.depth_state.is_some() { 12 } else { 7 };
    let mut packet = header(version, sequence, width, height, work.vertex_count);
    floats(&mut packet, clear.into_iter().chain([0.0; 4]));
    packet.extend_from_slice(&work.vertices);
    state(&mut packet, work);
    if let Some(state) = work.depth_state {
        floats(&mut packet, [1.0].into_iter());
        packet.extend_from_slice(&state.wire().to_le_bytes());
    }
    packet
}

fn solid(
    sequence: u32,
    width: u32,
    height: u32,
    clear: [f32; 4],
    work: &DrawWork,
    color: [f32; 4],
) -> Vec<u8> {
    let version = match work.depth_state {
        None => 2,
        Some(DepthState { compare: DepthCompare::Less, write: true }) => 9,
        Some(DepthState { write: true, .. }) => 10,
        Some(_) => 11,
    };
    let mut packet = header(version, sequence, width, height, work.vertex_count);
    floats(&mut packet, clear.into_iter().chain(color));
    packet.extend_from_slice(&work.vertices);
    state(&mut packet, work);
    if let Some(state) = work.depth_state {
        floats(&mut packet, [1.0].into_iter());
        if version == 10 { packet.extend_from_slice(&state.compare.wire().to_le_bytes()); }
        if version == 11 { packet.extend_from_slice(&state.wire().to_le_bytes()); }
    }
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
    let depth = work.depth_state;
    let extended = depth.is_some() || texture.sampler != SamplerConfig::CLAMP_NEAREST;
    let mut packet = header(
        if depth.is_some() { 13 } else if extended { 5 } else { 3 }, sequence, width, height, work.vertex_count,
    );
    floats(&mut packet, clear.into_iter().chain([0.0; 4]));
    packet.extend_from_slice(&work.vertices);
    state(&mut packet, work);
    if extended {
        packet.extend_from_slice(&texture.sampler.wire().to_le_bytes());
    }
    for value in [texture.width, texture.height] {
        packet.extend_from_slice(&value.to_le_bytes());
    }
    packet.extend_from_slice(&texture.bgra);
    if let Some(state) = depth {
        floats(&mut packet, [1.0].into_iter());
        packet.extend_from_slice(&state.wire().to_le_bytes());
    }
    packet
}

fn textured_pair(
    sequence: u32,
    width: u32,
    height: u32,
    clear: [f32; 4],
    work: &DrawWork,
    textures: &[super::TextureSnapshot; 2],
) -> Vec<u8> {
    let extended = textures
        .iter()
        .any(|texture| texture.sampler != SamplerConfig::CLAMP_NEAREST);
    let mut packet = header(
        if extended { 6 } else { 4 }, sequence, width, height, work.vertex_count,
    );
    floats(&mut packet, clear.into_iter().chain([0.0; 4]));
    packet.extend_from_slice(&work.vertices);
    state(&mut packet, work);
    if extended {
        for texture in textures {
            packet.extend_from_slice(&texture.sampler.wire().to_le_bytes());
        }
    }
    for texture in textures {
        for value in [texture.width, texture.height] {
            packet.extend_from_slice(&value.to_le_bytes());
        }
    }
    for texture in textures {
        packet.extend_from_slice(&texture.bgra);
    }
    packet
}

fn texture_color(
    sequence: u32,
    width: u32,
    height: u32,
    clear: [f32; 4],
    work: &DrawWork,
    texture: &super::TextureSnapshot,
) -> Vec<u8> {
    let mut packet = header(if work.depth_state.is_some() { 14 } else { 8 }, sequence, width, height, work.vertex_count);
    floats(&mut packet, clear.into_iter().chain([0.0; 4]));
    packet.extend_from_slice(&work.vertices);
    state(&mut packet, work);
    packet.extend_from_slice(&texture.sampler.wire().to_le_bytes());
    for value in [texture.width, texture.height] {
        packet.extend_from_slice(&value.to_le_bytes());
    }
    packet.extend_from_slice(&texture.bgra);
    if let Some(state) = work.depth_state { floats(&mut packet, [1.0].into_iter()); packet.extend_from_slice(&state.wire().to_le_bytes()); }
    packet
}

fn header(version: u32, sequence: u32, width: u32, height: u32, vertex_count: u32) -> Vec<u8> {
    let mut packet = b"VGD1".to_vec();
    for value in [version, sequence, width, height, vertex_count] {
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
