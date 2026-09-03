use super::{frame_pixels, read_u32, words_are};

const VERTICES: &[u32] = &[
    0, 0x3f40_0000, 0, 0x3f80_0000, 0x3f80_0000, 0, 0, 0x3f80_0000, 0, 0x3f80_0000,
    0xbf40_0000, 0xbf40_0000, 0, 0x3f80_0000, 0, 0x3f80_0000, 0, 0x3f80_0000, 0, 0x3f80_0000,
    0x3f40_0000, 0xbf40_0000, 0, 0x3f80_0000, 0, 0, 0x3f80_0000, 0x3f80_0000, 0, 0x3f80_0000,
];
const VIEWPORT: &[u32] = &[
    0x4380_0000, 0x4340_0000, 0x3f00_0000, 0x4400_0000, 0x43c0_0000, 0x3f00_0000,
];

pub(super) fn vtc1_sequence(packet: &[u8]) -> Result<u32, String> {
    if packet.len() != 244
        || packet.get(..4) != Some(b"VGD1")
        || [4, 12, 16, 20].into_iter().zip([8, 1024, 768, 3])
            .any(|(at, want)| read_u32(packet, at) != Some(want))
        || !words_are(packet, 24, &[0x3e80_0000, 0x3f00_0000, 0x3f40_0000, 0x3f80_0000])
        || !words_are(packet, 40, &[0, 0, 0, 0])
        || !words_are(packet, 56, VERTICES)
        || !words_are(packet, 176, VIEWPORT)
        || !words_are(packet, 200, &[448, 336, 128, 96])
        || !words_are(packet, 216, &[0x1092, 2, 2])
        || !packet[228..].chunks_exact(4).all(|pixel| pixel == [128, 128, 128, 255])
    {
        return Err("guest emitted an invalid texture-color VirGL packet".into());
    }
    read_u32(packet, 8)
        .filter(|sequence| *sequence != 0)
        .ok_or_else(|| "VGD1 texture-color packet has no nonzero sequence".into())
}

pub(crate) fn is_texture_color_readback(packet: &[u8]) -> bool {
    frame_pixels(packet).is_some_and(|pixels| {
        let center = (384 * 1024 + 512) * 4;
        let clipped = (384 * 1024 + 400) * 4;
        pixels[..4] == [191, 128, 64, 255]
            && pixels[center..center + 4] == [32, 32, 64, 255]
            && pixels[clipped..clipped + 4] == [191, 128, 64, 255]
    })
}
