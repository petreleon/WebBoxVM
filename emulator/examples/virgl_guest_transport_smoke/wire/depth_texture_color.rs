use super::{frame_pixels, read_u32, words_are};

const CLEAR: [u32; 4] = [0x3e80_0000, 0x3f00_0000, 0x3f40_0000, 0x3f80_0000];
const VERTICES: [u32; 30] = [
    0, 0x3f40_0000, 0xbf00_0000, 0x3f80_0000, 0x3f80_0000, 0, 0, 0x3f80_0000, 0, 0x3f80_0000,
    0xbf40_0000, 0xbf40_0000, 0xbf00_0000, 0x3f80_0000, 0, 0x3f80_0000, 0, 0x3f80_0000, 0, 0x3f80_0000,
    0x3f40_0000, 0xbf40_0000, 0xbf00_0000, 0x3f80_0000, 0, 0, 0x3f80_0000, 0x3f80_0000, 0, 0x3f80_0000,
];
const VIEWPORT: [u32; 6] = [0x4380_0000, 0x4340_0000, 0x3f00_0000, 0x4400_0000, 0x43c0_0000, 0x3f00_0000];
const TEXTURE: [u8; 16] = [128, 128, 128, 255, 128, 128, 128, 255, 128, 128, 128, 255, 128, 128, 128, 255];

pub(super) fn depth_texture_color_sequence(packet: &[u8]) -> Result<u32, String> {
    if packet.len() != 252 || packet.get(..4) != Some(b"VGD1")
        || [4, 12, 16, 20].into_iter().zip([14, 1024, 768, 3]).any(|(at, want)| read_u32(packet, at) != Some(want))
        || !words_are(packet, 24, &CLEAR) || !words_are(packet, 40, &[0, 0, 0, 0])
        || !words_are(packet, 56, &VERTICES) || !words_are(packet, 176, &VIEWPORT)
        || !words_are(packet, 200, &[448, 336, 128, 96]) || !words_are(packet, 216, &[0x1092, 2, 2])
        || packet.get(228..244) != Some(&TEXTURE) || !words_are(packet, 244, &[0x3f800000, 7])
    {
        return Err("guest emitted an invalid standard VirGL depth-texture-color packet".into());
    }
    read_u32(packet, 8).filter(|sequence| *sequence != 0)
        .ok_or_else(|| "VGD1 depth-texture-color packet has no nonzero sequence".into())
}

pub(crate) fn is_depth_texture_color_readback(packet: &[u8]) -> bool {
    frame_pixels(packet).is_some_and(|pixels| {
        let center = (384 * 1024 + 512) * 4; let clipped = (384 * 1024 + 400) * 4;
        pixels[..4] == [191, 128, 64, 255] && pixels[center..center + 4] == [32, 32, 64, 255]
            && pixels[clipped..clipped + 4] == [191, 128, 64, 255]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_fourteen_requires_exact_modulation_and_depth_dsa() {
        let mut packet = vec![0; 252]; packet[..4].copy_from_slice(b"VGD1");
        put(&mut packet, 4, &[14, 99, 1024, 768, 3]); put(&mut packet, 24, &CLEAR); put(&mut packet, 56, &VERTICES);
        put(&mut packet, 176, &VIEWPORT); put(&mut packet, 200, &[448, 336, 128, 96]); put(&mut packet, 216, &[0x1092, 2, 2]);
        packet[228..244].copy_from_slice(&TEXTURE); put(&mut packet, 244, &[0x3f800000, 7]);
        assert_eq!(depth_texture_color_sequence(&packet), Ok(99)); put(&mut packet, 248, &[5]);
        assert!(depth_texture_color_sequence(&packet).is_err());
    }

    fn put(packet: &mut [u8], offset: usize, values: &[u32]) {
        for (index, value) in values.iter().enumerate() {
            packet[offset + index * 4..offset + index * 4 + 4].copy_from_slice(&value.to_le_bytes());
        }
    }
}
