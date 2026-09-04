use super::{frame_pixels, read_u32, words_are};

const VIEWPORT: [u32; 6] = [0x4380_0000, 0x4340_0000, 0x3f00_0000, 0x4400_0000, 0x43c0_0000, 0x3f00_0000];
const VERTICES: [u32; 12] = [0, 0x3f40_0000, 0, 0x3f80_0000, 0xbf40_0000, 0xbf40_0000, 0, 0x3f80_0000, 0x3f40_0000, 0xbf40_0000, 0, 0x3f80_0000];
const DEPTH_NEAR: [u32; 12] = [0, 0x3f40_0000, 0xbf00_0000, 0x3f80_0000, 0xbf40_0000, 0xbf40_0000, 0xbf00_0000, 0x3f80_0000, 0x3f40_0000, 0xbf40_0000, 0xbf00_0000, 0x3f80_0000];
const DEPTH_FAR: [u32; 12] = [0, 0x3f40_0000, 0x3f00_0000, 0x3f80_0000, 0xbf40_0000, 0xbf40_0000, 0x3f00_0000, 0x3f80_0000, 0x3f40_0000, 0xbf40_0000, 0x3f00_0000, 0x3f80_0000];

pub(super) fn batch_sequence(packet: &[u8]) -> Result<u32, String> {
    if packet.len() != 264 || packet.get(..4) != Some(b"VGB1")
        || [4, 12, 16, 20, 24].into_iter().zip([1, 1024, 768, 2, 0]).any(|(at, want)| read_u32(packet, at) != Some(want))
        || !words_are(packet, 28, &[0, 0, 0, 0x3f80_0000]) || read_u32(packet, 44) != Some(0)
        || !draw(packet, 48, [0x3f80_0000, 0, 0, 0x3f00_0000], &VERTICES)
        || !draw(packet, 156, [0, 0x3f80_0000, 0, 0x3f00_0000], &VERTICES)
    {
        return Err("guest emitted an invalid standard VirGL solid-batch packet".into());
    }
    read_u32(packet, 8).filter(|sequence| *sequence != 0)
        .ok_or_else(|| "VGB1 solid-batch packet has no nonzero sequence".into())
}

pub(super) fn depth_batch_sequence(packet: &[u8]) -> Result<u32, String> {
    if packet.len() != 264 || packet.get(..4) != Some(b"VGB1")
        || [4, 12, 16, 20, 24].into_iter().zip([2, 1024, 768, 2, 0]).any(|(at, want)| read_u32(packet, at) != Some(want))
        || !words_are(packet, 28, &[0, 0, 0, 0x3f80_0000]) || read_u32(packet, 44) != Some(0x3f80_0000)
        || !draw(packet, 48, [0x3f80_0000, 0, 0, 0x3f00_0000], &DEPTH_NEAR)
        || !draw(packet, 156, [0, 0x3f80_0000, 0, 0x3f00_0000], &DEPTH_FAR)
    {
        return Err("guest emitted an invalid standard VirGL depth-batch packet".into());
    }
    read_u32(packet, 8).filter(|sequence| *sequence != 0)
        .ok_or_else(|| "VGB1 depth-batch packet has no nonzero sequence".into())
}

fn draw(packet: &[u8], offset: usize, color: [u32; 4], vertices: &[u32]) -> bool {
    read_u32(packet, offset) == Some(3) && words_are(packet, offset + 4, &color)
        && words_are(packet, offset + 20, &VIEWPORT)
        && words_are(packet, offset + 44, &[448, 336, 128, 96])
        && words_are(packet, offset + 60, vertices)
}

pub(crate) fn is_solid_batch_readback(packet: &[u8]) -> bool {
    frame_pixels(packet).is_some_and(|pixels| {
        let center = (384 * 1024 + 512) * 4;
        pixels[..4] == [0, 0, 0, 255] && pixels[center..center + 4] == [0, 128, 64, 255]
    })
}

pub(crate) fn is_depth_batch_readback(packet: &[u8]) -> bool {
    frame_pixels(packet).is_some_and(|pixels| {
        let center = (384 * 1024 + 512) * 4;
        pixels[..4] == [0, 0, 0, 255] && pixels[center..center + 4] == [0, 0, 128, 255]
    })
}
