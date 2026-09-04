use super::{frame_pixels, read_u32, words_are};

const CLEAR: [u32; 4] = [0x3e80_0000, 0x3f00_0000, 0x3f40_0000, 0x3f80_0000];
const INLINE: [u32; 4] = [0x3f4c_cccd, 0x3ecc_cccd, 0x3e4c_cccd, 0x3f00_0000];
const UNIFORM: [u32; 4] = [0x3e4c_cccd, 0x3f19_999a, 0x3ecccccd, 0x3f00_0000];
const VERTICES: [u32; 24] = [
    0xbd80_0000, 0xbe80_0000, 0, 0x3f80_0000, 0xbe80_0000, 0xbe80_0000, 0, 0x3f80_0000,
    0xbe80_0000, 0x3e80_0000, 0, 0x3f80_0000, 0x3e80_0000, 0xbe80_0000, 0, 0x3f80_0000,
    0x3d80_0000, 0xbe80_0000, 0, 0x3f80_0000, 0x3d80_0000, 0x3e80_0000, 0, 0x3f80_0000,
];
const VIEWPORT: [u32; 6] = [
    0x4380_0000, 0x4340_0000, 0x3f00_0000, 0x4400_0000, 0x43c0_0000, 0x3f00_0000,
];

pub(super) fn vgd1_sequence(packet: &[u8]) -> Result<u32, String> {
    sequence(packet, INLINE, "draw")
}

pub(super) fn uniform_sequence(packet: &[u8]) -> Result<u32, String> {
    sequence(packet, UNIFORM, "uniform-buffer draw")
}

fn sequence(packet: &[u8], color: [u32; 4], label: &str) -> Result<u32, String> {
    if packet.len() != 192
        || packet.get(..4) != Some(b"VGD1")
        || [4, 12, 16, 20].into_iter().zip([2, 1024, 768, 6])
            .any(|(at, want)| read_u32(packet, at) != Some(want))
        || !words_are(packet, 24, &CLEAR)
        || !words_are(packet, 40, &color)
        || !words_are(packet, 56, &VERTICES)
        || !words_are(packet, 152, &VIEWPORT)
        || !words_are(packet, 176, &[448, 336, 128, 96])
    {
        return Err(format!("guest emitted an invalid standard VirGL {label} packet"));
    }
    read_u32(packet, 8)
        .filter(|sequence| *sequence != 0)
        .ok_or_else(|| format!("VGD1 {label} packet has no nonzero sequence"))
}

pub(crate) fn is_triangle_readback(packet: &[u8]) -> bool {
    solid_readback(packet, [121, 115, 134, 255])
}

pub(crate) fn is_uniform_readback(packet: &[u8]) -> bool {
    solid_readback(packet, [147, 141, 58, 255])
}

fn solid_readback(packet: &[u8], color: [u8; 4]) -> bool {
    frame_pixels(packet).is_some_and(|pixels| {
        let left = (384 * 1024 + 465) * 4;
        let right = (384 * 1024 + 530) * 4;
        let gap = (384 * 1024 + 512) * 4;
        pixels[..4] == [191, 128, 64, 255]
            && pixels[left..left + 4] == color
            && pixels[right..right + 4] == color
            && pixels[gap..gap + 4] == [191, 128, 64, 255]
    })
}
