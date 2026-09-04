use super::{frame_pixels, read_u32, words_are};

const CLEAR: [u32; 4] = [0x3dcc_cccd, 0x3e4c_cccd, 0x3e99_999a, 0x3f80_0000];
const COLOR: [u32; 4] = [0, 0x3f80_0000, 0, 0x3e80_0000];
const VERTICES: [u32; 24] = [
    0, 0x3f40_0000, 0xbf00_0000, 0x3f80_0000, 0xbf40_0000, 0xbf40_0000, 0xbf00_0000, 0x3f80_0000,
    0x3f40_0000, 0xbf40_0000, 0xbf00_0000, 0x3f80_0000, 0, 0x3f40_0000, 0x3f00_0000, 0x3f80_0000,
    0xbf40_0000, 0xbf40_0000, 0x3f00_0000, 0x3f80_0000, 0x3f40_0000, 0xbf40_0000, 0x3f00_0000, 0x3f80_0000,
];
const VIEWPORT: [u32; 6] = [0x4380_0000, 0x4340_0000, 0x3f00_0000, 0x4400_0000, 0x43c0_0000, 0x3f00_0000];

pub(super) fn depth_sequence(packet: &[u8]) -> Result<u32, String> {
    if packet.len() != 196 || packet.get(..4) != Some(b"VGD1")
        || [4, 12, 16, 20].into_iter().zip([9, 1024, 768, 6]).any(|(at, want)| read_u32(packet, at) != Some(want))
        || !words_are(packet, 24, &CLEAR) || !words_are(packet, 40, &COLOR)
        || !words_are(packet, 56, &VERTICES) || !words_are(packet, 152, &VIEWPORT)
        || !words_are(packet, 176, &[448, 336, 128, 96]) || read_u32(packet, 192) != Some(0x3f80_0000)
    {
        return Err("guest emitted an invalid standard VirGL depth packet".into());
    }
    read_u32(packet, 8).filter(|sequence| *sequence != 0)
        .ok_or_else(|| "VGD1 depth packet has no nonzero sequence".into())
}

pub(crate) fn is_depth_readback(packet: &[u8]) -> bool {
    frame_pixels(packet).is_some_and(|pixels| {
        let middle = (384 * 1024 + 512) * 4;
        pixels[..4] == [77, 51, 26, 255] && pixels[middle..middle + 4] == [58, 102, 20, 255]
    })
}
