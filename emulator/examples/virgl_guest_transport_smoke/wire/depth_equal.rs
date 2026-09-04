use super::{frame_pixels, read_u32, words_are};

const CLEAR: [u32; 4] = [0, 0, 0, 0x3f80_0000];
const COLOR: [u32; 4] = [0, 0, 0x3f80_0000, 0x3f00_0000];
const VERTICES: [u32; 12] = [
    0, 0x3f40_0000, 0x3f80_0000, 0x3f80_0000, 0xbf40_0000, 0xbf40_0000,
    0x3f800000, 0x3f800000, 0x3f400000, 0xbf400000, 0x3f800000, 0x3f800000,
];
const VIEWPORT: [u32; 6] = [0x4380_0000, 0x4340_0000, 0x3f00_0000, 0x4400_0000, 0x43c0_0000, 0x3f00_0000];

pub(super) fn depth_equal_sequence(packet: &[u8]) -> Result<u32, String> {
    if packet.len() != 152 || packet.get(..4) != Some(b"VGD1")
        || [4, 12, 16, 20].into_iter().zip([10, 1024, 768, 3]).any(|(at, want)| read_u32(packet, at) != Some(want))
        || !words_are(packet, 24, &CLEAR) || !words_are(packet, 40, &COLOR)
        || !words_are(packet, 56, &VERTICES) || !words_are(packet, 104, &VIEWPORT)
        || !words_are(packet, 128, &[448, 336, 128, 96])
        || read_u32(packet, 144) != Some(0x3f80_0000) || read_u32(packet, 148) != Some(2)
    {
        return Err("guest emitted an invalid standard VirGL depth-equal packet".into());
    }
    read_u32(packet, 8).filter(|sequence| *sequence != 0)
        .ok_or_else(|| "VGD1 depth-equal packet has no nonzero sequence".into())
}

pub(crate) fn is_depth_equal_readback(packet: &[u8]) -> bool {
    frame_pixels(packet).is_some_and(|pixels| {
        let middle = (384 * 1024 + 512) * 4;
        pixels[..4] == [0, 0, 0, 255] && pixels[middle..middle + 4] == [128, 0, 0, 255]
    })
}
