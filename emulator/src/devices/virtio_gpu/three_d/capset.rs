use super::packet::{PACKET_HEADER_BYTES, VERTEX_FLOATS};
use super::{
    CAPSET_ID, CAPSET_SIZE, CAPSET_VERSION, MAX_3D_DIMENSION, MAX_3D_INDICES, MAX_3D_VERTICES,
};

pub(in crate::devices::virtio_gpu) const VIRGL_CAPSET_ID: u32 = 1;
const VIRGL_CAPSET_VERSION: u32 = 1;
const VIRGL_CAPSET_SIZE: u32 = 308;
pub(in crate::devices::virtio_gpu) const CAPSET_COUNT: u32 = 2;
const VIRGL_SAMPLER_FORMATS_0: u32 = 1 << 1;
const VIRGL_SAMPLER_FORMATS_2: u32 = 1 << 3;
const VIRGL_COLOR_RENDER_FORMATS: u32 = 0b1_1110;
const VIRGL_VERTEX_FORMATS: u32 = (1 << 29) | (1 << 31);
const VIRGL_PRIMITIVES: u32 = (1 << 4) | (1 << 5) | (1 << 6);

pub(super) struct Capset {
    pub id: u32,
    pub version: u32,
    pub size: u32,
}

const VIRGL: Capset = Capset {
    id: VIRGL_CAPSET_ID,
    version: VIRGL_CAPSET_VERSION,
    size: VIRGL_CAPSET_SIZE,
};
const WBG3: Capset = Capset {
    id: CAPSET_ID,
    version: CAPSET_VERSION,
    size: CAPSET_SIZE,
};

pub(super) fn by_index(index: u32) -> Option<&'static Capset> {
    match index {
        0 => Some(&VIRGL),
        1 => Some(&WBG3),
        _ => None,
    }
}

pub(super) fn supports(id: u32) -> bool {
    matches!(id, VIRGL_CAPSET_ID | CAPSET_ID)
}

pub(super) fn data(id: u32, version: u32) -> Option<Vec<u8>> {
    match (id, version) {
        (VIRGL_CAPSET_ID, VIRGL_CAPSET_VERSION) => Some(virgl_caps()),
        (CAPSET_ID, CAPSET_VERSION) => Some(wbg3_caps()),
        _ => None,
    }
}

fn virgl_caps() -> Vec<u8> {
    let mut caps = vec![0; VIRGL_CAPSET_SIZE as usize];
    write_u32(&mut caps, 0, VIRGL_CAPSET_VERSION);
    write_u32(&mut caps, 4, VIRGL_SAMPLER_FORMATS_0);
    write_u32(&mut caps, 12, VIRGL_SAMPLER_FORMATS_2);
    write_u32(&mut caps, 68, VIRGL_COLOR_RENDER_FORMATS);
    write_u32(&mut caps, 196, VIRGL_VERTEX_FORMATS);
    write_u32(&mut caps, 268, 1);
    write_u32(&mut caps, 280, 1);
    write_u32(&mut caps, 284, 1);
    write_u32(&mut caps, 288, VIRGL_PRIMITIVES);
    caps
}

fn wbg3_caps() -> Vec<u8> {
    let mut caps = b"WBG3".to_vec();
    for value in [
        CAPSET_VERSION,
        MAX_3D_DIMENSION,
        MAX_3D_VERTICES,
        MAX_3D_INDICES,
        PACKET_HEADER_BYTES as u32,
        VERTEX_FLOATS as u32,
        2,
    ] {
        caps.extend_from_slice(&value.to_le_bytes());
    }
    caps
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}
