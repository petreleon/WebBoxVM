use super::packet::{PACKET_HEADER_BYTES, VERTEX_FLOATS};
use super::{
    CAPSET_ID, CAPSET_SIZE, CAPSET_VERSION, MAX_3D_DIMENSION, MAX_3D_INDICES, MAX_3D_VERTICES,
};

pub(in crate::devices::virtio_gpu) const VIRGL_CAPSET_ID: u32 = 1;
pub(in crate::devices::virtio_gpu) const VIRGL2_CAPSET_ID: u32 = 2;
const VIRGL_CAPSET_VERSION: u32 = 1;
const VIRGL2_CAPSET_VERSION: u32 = 2;
const VIRGL_CAPSET_SIZE: u32 = 308;
// `sizeof(struct virgl_caps_v2)` in current upstream's growable layout.
const VIRGL2_CAPSET_SIZE: u32 = 1376;
pub(in crate::devices::virtio_gpu) const CAPSET_COUNT: u32 = 3;
const VIRGL_SAMPLER_FORMATS_0: u32 = 1 << 1;
const VIRGL_SAMPLER_FORMATS_2: u32 = 1 << 3;
const VIRGL_COLOR_RENDER_FORMATS: u32 = 0b1_1110;
const VIRGL_DEPTH_STENCIL_FORMATS: u32 = 1 << 18;
const VIRGL_VERTEX_FORMATS: u32 = (1 << 29) | (1 << 31);
const VIRGL_PRIMITIVES: u32 = (1 << 4) | (1 << 5) | (1 << 6);
const VIRGL_UBO: u32 = 1 << 18;

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
const VIRGL2: Capset = Capset {
    id: VIRGL2_CAPSET_ID,
    version: VIRGL2_CAPSET_VERSION,
    size: VIRGL2_CAPSET_SIZE,
};
const WBG3: Capset = Capset {
    id: CAPSET_ID,
    version: CAPSET_VERSION,
    size: CAPSET_SIZE,
};

pub(super) fn by_index(index: u32) -> Option<&'static Capset> {
    match index {
        0 => Some(&VIRGL),
        1 => Some(&VIRGL2),
        2 => Some(&WBG3),
        _ => None,
    }
}

pub(super) fn supports(id: u32) -> bool {
    matches!(id, VIRGL_CAPSET_ID | VIRGL2_CAPSET_ID | CAPSET_ID)
}

pub(super) fn is_virgl_capset(id: u32) -> bool {
    matches!(id, VIRGL_CAPSET_ID | VIRGL2_CAPSET_ID)
}

pub(super) fn data(id: u32, version: u32) -> Option<Vec<u8>> {
    match (id, version) {
        (VIRGL_CAPSET_ID, VIRGL_CAPSET_VERSION) => Some(virgl_caps()),
        (VIRGL2_CAPSET_ID, VIRGL2_CAPSET_VERSION) => Some(virgl2_caps()),
        (CAPSET_ID, CAPSET_VERSION) => Some(wbg3_caps()),
        _ => None,
    }
}

fn virgl_caps() -> Vec<u8> {
    let mut caps = vec![0; VIRGL_CAPSET_SIZE as usize];
    write_virgl_v1_caps(&mut caps, VIRGL_CAPSET_VERSION);
    caps
}

fn virgl2_caps() -> Vec<u8> {
    let mut caps = vec![0; VIRGL2_CAPSET_SIZE as usize];
    write_virgl_v1_caps(&mut caps, VIRGL2_CAPSET_VERSION);
    // The trailing `virgl_caps_v2` fields remain zero until a generic feature
    // has an implementation and a guest-visible validation path.
    caps
}

fn write_virgl_v1_caps(caps: &mut [u8], version: u32) {
    write_u32(caps, 0, version);
    write_u32(caps, 4, VIRGL_SAMPLER_FORMATS_0);
    write_u32(caps, 12, VIRGL_SAMPLER_FORMATS_2);
    write_u32(caps, 68, VIRGL_COLOR_RENDER_FORMATS);
    write_u32(caps, 132, VIRGL_DEPTH_STENCIL_FORMATS);
    write_u32(caps, 196, VIRGL_VERTEX_FORMATS);
    write_u32(caps, 260, VIRGL_UBO);
    write_u32(caps, 268, 1);
    write_u32(caps, 280, 1);
    write_u32(caps, 284, 1);
    write_u32(caps, 288, VIRGL_PRIMITIVES);
    write_u32(caps, 296, 1);
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
