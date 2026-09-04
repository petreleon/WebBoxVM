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
