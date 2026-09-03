use super::{frame_pixels, read_u32, words_are};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TextureMode {
    Repeat,
    Linear,
}

const VERTICES: &[u32] = &[
    0,
    0x3f40_0000,
    0,
    0x3f80_0000,
    0,
    0x3f80_0000,
    0xbf40_0000,
    0xbf40_0000,
    0,
    0x3f80_0000,
    0,
    0x3f80_0000,
    0x3f40_0000,
    0xbf40_0000,
    0,
    0x3f80_0000,
    0,
    0x3f80_0000,
];
const VIEWPORT: &[u32] = &[
    0x4380_0000,
    0x4340_0000,
    0x3f00_0000,
    0x4400_0000,
    0x43c0_0000,
    0x3f00_0000,
];
const TEXTURE: &[u8] = &[
    10, 20, 30, 255, 40, 50, 60, 255, 70, 80, 90, 255, 100, 110, 120, 255,
];

pub(super) fn vgt1_sequence(packet: &[u8]) -> Result<(u32, TextureMode), String> {
    let (mode, u) = match read_u32(packet, 168) {
        Some(0x1080) => (TextureMode::Repeat, 0x3f80_0000),
        Some(0x3292) => (TextureMode::Linear, 0x3f00_0000),
        _ => return Err("guest emitted an unsupported textured sampler state".into()),
    };
    if packet.len() != 196
        || packet.get(..4) != Some(b"VGD1")
        || [4, 12, 16, 20]
            .into_iter()
            .zip([5, 1024, 768, 3])
            .any(|(at, want)| read_u32(packet, at) != Some(want))
        || !words_are(
            packet,
            24,
            &[0x3e80_0000, 0x3f00_0000, 0x3f40_0000, 0x3f80_0000],
        )
        || !words_are(packet, 40, &[0, 0, 0, 0])
        || !vertices_match(packet, u)
        || !words_are(packet, 128, VIEWPORT)
        || !words_are(packet, 152, &[448, 336, 128, 96])
        || !words_are(packet, 172, &[2, 2])
        || packet.get(180..) != Some(TEXTURE)
    {
        return Err("guest emitted an invalid standard VirGL textured draw packet".into());
    }
    read_u32(packet, 8)
        .filter(|sequence| *sequence != 0)
        .map(|sequence| (sequence, mode))
        .ok_or_else(|| "VGD1 textured packet has no nonzero sequence".into())
}

fn vertices_match(packet: &[u8], u: u32) -> bool {
    VERTICES.iter().enumerate().all(|(index, expected)| {
        let expected = if matches!(index, 4 | 10 | 16) {
            u
        } else {
            *expected
        };
        read_u32(packet, 56 + index * 4) == Some(expected)
    })
}

pub(crate) fn is_textured_triangle_readback(packet: &[u8], mode: TextureMode) -> bool {
    let center_pixel = match mode {
        TextureMode::Repeat => [10, 20, 30, 255],
        TextureMode::Linear => [25, 35, 45, 255],
    };
    frame_pixels(packet).is_some_and(|pixels| {
        let center = (384 * 1024 + 512) * 4;
        let clipped = (384 * 1024 + 400) * 4;
        pixels[..4] == [191, 128, 64, 255]
            && pixels[center..center + 4] == center_pixel
            && pixels[clipped..clipped + 4] == [191, 128, 64, 255]
    })
}
