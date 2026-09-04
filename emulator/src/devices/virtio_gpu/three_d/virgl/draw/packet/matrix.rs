use super::super::{DrawWork, GpuMatrix};

pub(super) fn packet(
    sequence: u32,
    width: u32,
    height: u32,
    clear: [f32; 4],
    work: &DrawWork,
    matrix: &GpuMatrix,
    color: [f32; 4],
) -> Vec<u8> {
    let mut packet = super::header(15, sequence, width, height, work.vertex_count);
    super::floats(&mut packet, clear.into_iter().chain(color));
    super::floats(&mut packet, matrix.rows.into_iter());
    packet.extend_from_slice(&matrix.raw_vertices);
    super::state(&mut packet, work);
    packet
}

pub(super) fn vertex_color(
    sequence: u32,
    width: u32,
    height: u32,
    clear: [f32; 4],
    work: &DrawWork,
    matrix: &GpuMatrix,
) -> Vec<u8> {
    let mut packet = super::header(16, sequence, width, height, work.vertex_count);
    super::floats(&mut packet, clear.into_iter().chain([0.0; 4]));
    super::floats(&mut packet, matrix.rows.into_iter());
    packet.extend_from_slice(&matrix.raw_vertices);
    super::state(&mut packet, work);
    packet
}

pub(super) fn texture(
    sequence: u32,
    width: u32,
    height: u32,
    clear: [f32; 4],
    work: &DrawWork,
    matrix: &GpuMatrix,
    texture: &super::super::TextureSnapshot,
) -> Vec<u8> {
    let mut packet = super::header(17, sequence, width, height, work.vertex_count);
    super::floats(&mut packet, clear.into_iter().chain([0.0; 4]));
    super::floats(&mut packet, matrix.rows.into_iter());
    packet.extend_from_slice(&matrix.raw_vertices);
    super::state(&mut packet, work);
    for value in [texture.sampler.wire(), texture.width, texture.height] { packet.extend_from_slice(&value.to_le_bytes()); }
    packet.extend_from_slice(&texture.bgra);
    packet
}
