use super::{frame_pixels, read_u32, words_are};

pub(super) fn vgd1_sequence(packet: &[u8]) -> Result<u32, String> {
    let vertices = [
        0,
        0x3f40_0000,
        0,
        0x3f80_0000,
        0xbf40_0000,
        0xbf40_0000,
        0,
        0x3f80_0000,
        0x3f40_0000,
        0xbf40_0000,
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
    if packet.len() != 144
        || packet.get(..4) != Some(b"VGD1")
        || [4, 12, 16, 20]
            .into_iter()
            .zip([2, 1024, 768, 3])
            .any(|(at, want)| read_u32(packet, at) != Some(want))
        || !words_are(
            packet,
            24,
            &[0x3e80_0000, 0x3f00_0000, 0x3f40_0000, 0x3f80_0000],
        )
        || !words_are(packet, 40, &[0, 0x3f80_0000, 0, 0x3e80_0000])
        || !words_are(packet, 56, &vertices)
        || !words_are(packet, 104, &viewport)
        || !words_are(packet, 128, &[448, 336, 128, 96])
    {
        return Err("guest emitted an invalid standard VirGL draw packet".into());
    }
    read_u32(packet, 8)
        .filter(|sequence| *sequence != 0)
        .ok_or_else(|| "VGD1 packet has no nonzero sequence".into())
}

pub(crate) fn is_triangle_readback(packet: &[u8]) -> bool {
    frame_pixels(packet).is_some_and(|pixels| {
        let center = (384 * 1024 + 512) * 4;
        let clipped = (384 * 1024 + 400) * 4;
        pixels[..4] == [191, 128, 64, 255]
            && pixels[center..center + 4] == [143, 160, 48, 255]
            && pixels[clipped..clipped + 4] == [191, 128, 64, 255]
    })
}
