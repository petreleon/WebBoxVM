use super::{frame_pixels, read_u32, words_are};

const VERTICES: &[u32] = &[
    0, 0x3f40_0000, 0, 0x3f80_0000, 0x3f80_0000, 0x3f20_0000,
    0xbf40_0000, 0xbf40_0000, 0, 0x3f80_0000, 0x3f80_0000, 0x3f20_0000,
    0x3f40_0000, 0xbf40_0000, 0, 0x3f80_0000, 0x3f80_0000, 0x3f20_0000,
];
const VIEWPORT: &[u32] = &[
    0x4380_0000, 0x4340_0000, 0x3f00_0000, 0x4400_0000, 0x43c0_0000, 0x3f00_0000,
];
const LEFT: &[u8] = &[
    10, 20, 30, 255, 40, 50, 60, 255, 70, 80, 90, 255, 100, 110, 120, 255,
];
const RIGHT: &[u8] = &[
    255, 255, 255, 255, 0, 0, 0, 255, 0, 0, 0, 255, 0, 0, 0, 255,
];

pub(super) fn vtp1_sequence(packet: &[u8]) -> Result<u32, String> {
    if packet.len() != 224
        || packet.get(..4) != Some(b"VGD1")
        || [4, 12, 16, 20].into_iter().zip([6, 1024, 768, 3])
            .any(|(at, want)| read_u32(packet, at) != Some(want))
        || !words_are(packet, 24, &[0x3e80_0000, 0x3f00_0000, 0x3f40_0000, 0x3f80_0000])
        || !words_are(packet, 40, &[0, 0, 0, 0])
        || !words_are(packet, 56, VERTICES)
        || !words_are(packet, 128, VIEWPORT)
        || !words_are(packet, 152, &[448, 336, 128, 96])
        || !words_are(packet, 168, &[0x3292, 0x1080, 2, 2, 2, 2])
        || packet.get(192..208) != Some(LEFT)
        || packet.get(208..) != Some(RIGHT)
    {
        return Err("guest emitted an invalid independent-sampler VirGL packet".into());
    }
    read_u32(packet, 8)
        .filter(|sequence| *sequence != 0)
        .ok_or_else(|| "VGD1 texture-pair packet has no nonzero sequence".into())
}

pub(crate) fn is_texture_pair_readback(packet: &[u8]) -> bool {
    frame_pixels(packet).is_some_and(|pixels| {
        let center = (384 * 1024 + 512) * 4;
        let clipped = (384 * 1024 + 400) * 4;
        pixels[..4] == [191, 128, 64, 255]
            && pixels[center..center + 4] == [55, 65, 75, 255]
            && pixels[clipped..clipped + 4] == [191, 128, 64, 255]
    })
}
