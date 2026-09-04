use super::{frame_pixels, read_u32, words_are};

const CLEAR: [u32; 4] = [0, 0, 0, 0x3f80_0000];
const RED: [u32; 4] = [0x3f80_0000, 0, 0, 0x3f00_0000];
const BLUE: [u32; 4] = [0, 0, 0x3f80_0000, 0x3f00_0000];
const VERTICES: [u32; 12] = [
    0, 0x3f40_0000, 0x3f80_0000, 0x3f80_0000, 0xbf40_0000, 0xbf40_0000,
    0x3f800000, 0x3f800000, 0x3f400000, 0xbf400000, 0x3f800000, 0x3f800000,
];
const VIEWPORT: [u32; 6] = [0x4380_0000, 0x4340_0000, 0x3f00_0000, 0x4400_0000, 0x43c0_0000, 0x3f00_0000];

pub(super) fn depth_equal_batch_sequence(packet: &[u8]) -> Result<u32, String> {
    if packet.len() != 264 || packet.get(..4) != Some(b"VGB1")
        || [4, 12, 16, 20, 24].into_iter().zip([3, 1024, 768, 2, 2]).any(|(at, want)| read_u32(packet, at) != Some(want))
        || !words_are(packet, 28, &CLEAR) || read_u32(packet, 44) != Some(0x3f80_0000)
        || !draw(packet, 48, RED) || !draw(packet, 156, BLUE)
    {
        return Err("guest emitted an invalid standard VirGL depth-equal-batch packet".into());
    }
    read_u32(packet, 8).filter(|sequence| *sequence != 0)
        .ok_or_else(|| "VGB1 depth-equal-batch packet has no nonzero sequence".into())
}

fn draw(packet: &[u8], offset: usize, color: [u32; 4]) -> bool {
    read_u32(packet, offset) == Some(3) && words_are(packet, offset + 4, &color)
        && words_are(packet, offset + 20, &VIEWPORT)
        && words_are(packet, offset + 44, &[448, 336, 128, 96])
        && words_are(packet, offset + 60, &VERTICES)
}

pub(crate) fn is_depth_equal_batch_readback(packet: &[u8]) -> bool {
    frame_pixels(packet).is_some_and(|pixels| {
        let center = (384 * 1024 + 512) * 4;
        pixels[..4] == [0, 0, 0, 255] && pixels[center..center + 4] == [128, 0, 64, 255]
    })
}
