use super::{MAX_3D_DIMENSION, MAX_3D_INDICES, MAX_3D_VERTICES};
use crate::devices::virtio_gpu::protocol::read_u32;

pub(super) const PACKET_HEADER_BYTES: usize = 48;
const MVP_FLOATS: usize = 16;
pub(super) const VERTEX_FLOATS: usize = 7;
const FIXED_PACKET_BYTES: usize = PACKET_HEADER_BYTES + MVP_FLOATS * 4;
pub(in crate::devices::virtio_gpu) const MAX_WBG3_PACKET_BYTES: usize =
    FIXED_PACKET_BYTES + MAX_3D_VERTICES as usize * VERTEX_FLOATS * 4 + MAX_3D_INDICES as usize * 2;

pub(super) fn decode_submit(input: &[u8]) -> Option<&[u8]> {
    let size = usize::try_from(read_u32(input, 24)?).ok()?;
    if read_u32(input, 28)? != 0 || input.len() != 32usize.checked_add(size)? {
        return None;
    }
    let packet = input.get(32..)?;
    validate_packet(packet).then_some(packet)
}

fn validate_packet(packet: &[u8]) -> bool {
    if packet.len() < FIXED_PACKET_BYTES || packet.get(..4) != Some(b"WBG3") {
        return false;
    }
    let fields = (
        read_u32(packet, 4),
        read_u32(packet, 8),
        read_u32(packet, 16),
        read_u32(packet, 20),
        read_u32(packet, 24),
        read_u32(packet, 28),
    );
    let (Some(1), Some(1), Some(width), Some(height), Some(vertices), Some(indices)) = fields
    else {
        return false;
    };
    if width == 0
        || height == 0
        || width > MAX_3D_DIMENSION
        || height > MAX_3D_DIMENSION
        || vertices > MAX_3D_VERTICES
        || indices > MAX_3D_INDICES
        || indices % 3 != 0
    {
        return false;
    }
    let Some(vertex_bytes) = (vertices as usize).checked_mul(VERTEX_FLOATS * 4) else {
        return false;
    };
    let expected = FIXED_PACKET_BYTES
        .checked_add(vertex_bytes)
        .and_then(|len| len.checked_add(indices as usize * 2));
    if expected != Some(packet.len())
        || !finite_floats(&packet[32..FIXED_PACKET_BYTES + vertex_bytes])
    {
        return false;
    }
    packet[FIXED_PACKET_BYTES + vertex_bytes..]
        .chunks_exact(2)
        .all(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]) < vertices as u16)
}

fn finite_floats(bytes: &[u8]) -> bool {
    bytes
        .chunks_exact(4)
        .all(|value| f32::from_le_bytes(value.try_into().expect("four-byte float")).is_finite())
}
