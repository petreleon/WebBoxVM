use super::{frame_pixels, read_u32, words_are};

pub(super) fn vgd1_sequence(packet: &[u8]) -> Result<u32, String> {
    let vertices = [
        0xbd80_0000,
        0xbe80_0000,
        0,
        0x3f80_0000,
        0xbe80_0000,
        0xbe80_0000,
        0,
        0x3f80_0000,
        0xbe80_0000,
        0x3e80_0000,
        0,
        0x3f80_0000,
        0x3e80_0000,
        0xbe80_0000,
        0,
        0x3f80_0000,
        0x3d80_0000,
        0xbe80_0000,
        0,
        0x3f80_0000,
        0x3d80_0000,
        0x3e80_0000,
        0,
        0x3f80_0000,
    ];
    let viewport = [
        0x4380_0000,
        0x4340_0000,
        0x3f00_0000,
        0x4400_0000,
        0x43c0_0000,
        0x3f00_0000,
    ];
    if packet.len() != 192
        || packet.get(..4) != Some(b"VGD1")
        || [4, 12, 16, 20]
            .into_iter()
            .zip([2, 1024, 768, 6])
            .any(|(at, want)| read_u32(packet, at) != Some(want))
        || !words_are(
            packet,
            24,
            &[0x3e80_0000, 0x3f00_0000, 0x3f40_0000, 0x3f80_0000],
        )
        || !words_are(packet, 40, &[0x3f4c_cccd, 0x3ecc_cccd, 0x3e4c_cccd, 0x3f00_0000])
        || !words_are(packet, 56, &vertices)
        || !words_are(packet, 152, &viewport)
        || !words_are(packet, 176, &[448, 336, 128, 96])
    {
        return Err("guest emitted an invalid standard VirGL draw packet".into());
    }
    read_u32(packet, 8)
        .filter(|sequence| *sequence != 0)
        .ok_or_else(|| "VGD1 packet has no nonzero sequence".into())
}

pub(crate) fn is_triangle_readback(packet: &[u8]) -> bool {
    frame_pixels(packet).is_some_and(|pixels| {
        let left = (384 * 1024 + 465) * 4;
        let right = (384 * 1024 + 530) * 4;
        let gap = (384 * 1024 + 512) * 4;
        pixels[..4] == [191, 128, 64, 255]
            && pixels[left..left + 4] == [121, 115, 134, 255]
            && pixels[right..right + 4] == [121, 115, 134, 255]
            && pixels[gap..gap + 4] == [191, 128, 64, 255]
    })
}
