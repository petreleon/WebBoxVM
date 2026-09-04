use super::{frame_pixels, read_u32, words_are};

const CLEAR: [u32; 4] = [0x3e80_0000, 0x3f00_0000, 0x3f40_0000, 0x3f80_0000];
const VIEWPORT: [u32; 6] = [0x4380_0000, 0x4340_0000, 0x3f00_0000, 0x4400_0000, 0x43c0_0000, 0x3f00_0000];
const SCISSOR: [u32; 4] = [448, 336, 128, 96];
const SOLID: [u32; 4] = [0x3f80_0000, 0, 0, 0x3f00_0000];
const SOLID_VERTICES: [u32; 12] = [
    0, 0x3f40_0000, 0x3f00_0000, 0x3f80_0000, 0xbf40_0000, 0xbf40_0000,
    0x3f00_0000, 0x3f80_0000, 0x3f40_0000, 0xbf40_0000, 0x3f00_0000, 0x3f80_0000,
];
const TEXTURE_VERTICES: [u32; 30] = [
    0, 0x3f40_0000, 0xbf00_0000, 0x3f80_0000, 0x3f80_0000, 0, 0, 0x3f80_0000, 0, 0x3f80_0000,
    0xbf40_0000, 0xbf40_0000, 0xbf00_0000, 0x3f80_0000, 0, 0x3f80_0000, 0, 0x3f80_0000, 0, 0x3f80_0000,
    0x3f40_0000, 0xbf40_0000, 0xbf00_0000, 0x3f80_0000, 0, 0, 0x3f80_0000, 0x3f80_0000, 0, 0x3f80_0000,
];
const TEXTURE: [u8; 16] = [128, 128, 128, 255, 128, 128, 128, 255, 128, 128, 128, 255, 128, 128, 128, 255];

pub(super) fn material_batch_sequence(packet: &[u8]) -> Result<u32, String> {
    if packet.len() != 364 || packet.get(..4) != Some(b"VGM1")
        || [4, 12, 16, 20, 24].into_iter().zip([1, 1024, 768, 2, 1]).any(|(at, want)| read_u32(packet, at) != Some(want))
        || !words_are(packet, 28, &CLEAR) || read_u32(packet, 44) != Some(0x3f80_0000)
        || !solid(packet) || !texture_color(packet)
    {
        return Err("guest emitted an invalid standard VirGL material-batch packet".into());
    }
    read_u32(packet, 8).filter(|sequence| *sequence != 0)
        .ok_or_else(|| "VGM1 material-batch packet has no nonzero sequence".into())
}

fn solid(packet: &[u8]) -> bool {
    words_are(packet, 48, &[1, 7, 3]) && words_are(packet, 60, &VIEWPORT)
        && words_are(packet, 84, &SCISSOR) && words_are(packet, 100, &SOLID)
        && words_are(packet, 116, &SOLID_VERTICES)
}

fn texture_color(packet: &[u8]) -> bool {
    words_are(packet, 164, &[5, 7, 3]) && words_are(packet, 176, &VIEWPORT)
        && words_are(packet, 200, &SCISSOR) && words_are(packet, 216, &[0x1092, 2, 2])
        && packet.get(228..244) == Some(&TEXTURE) && words_are(packet, 244, &TEXTURE_VERTICES)
}

pub(crate) fn is_material_batch_readback(packet: &[u8]) -> bool {
    frame_pixels(packet).is_some_and(|pixels| {
        let center = (384 * 1024 + 512) * 4;
        pixels[..4] == [191, 128, 64, 255] && pixels[center..center + 4] == [32, 32, 64, 255]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_requires_ordered_depth_materials() {
        let mut packet = vec![0; 364]; packet[..4].copy_from_slice(b"VGM1");
        put(&mut packet, 4, &[1, 99, 1024, 768, 2, 1]); put(&mut packet, 28, &CLEAR); put(&mut packet, 44, &[0x3f80_0000]);
        put(&mut packet, 48, &[1, 7, 3]); put(&mut packet, 60, &VIEWPORT); put(&mut packet, 84, &SCISSOR); put(&mut packet, 100, &SOLID); put(&mut packet, 116, &SOLID_VERTICES);
        put(&mut packet, 164, &[5, 7, 3]); put(&mut packet, 176, &VIEWPORT); put(&mut packet, 200, &SCISSOR); put(&mut packet, 216, &[0x1092, 2, 2]); packet[228..244].copy_from_slice(&TEXTURE); put(&mut packet, 244, &TEXTURE_VERTICES);
        assert_eq!(material_batch_sequence(&packet), Ok(99)); put(&mut packet, 168, &[5]);
        assert!(material_batch_sequence(&packet).is_err());
    }

    fn put(packet: &mut [u8], offset: usize, values: &[u32]) {
        for (index, value) in values.iter().enumerate() {
            packet[offset + index * 4..offset + index * 4 + 4].copy_from_slice(&value.to_le_bytes());
        }
    }
}
