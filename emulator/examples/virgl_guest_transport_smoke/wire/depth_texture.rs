use super::{frame_pixels, read_u32, words_are};

const CLEAR: [u32; 4] = [0x3e80_0000, 0x3f00_0000, 0x3f40_0000, 0x3f80_0000];
const VERTICES: [u32; 18] = [
    0, 0x3f40_0000, 0xbf00_0000, 0x3f800000, 0, 0x3f800000,
    0xbf400000, 0xbf400000, 0xbf000000, 0x3f800000, 0, 0x3f800000,
    0x3f400000, 0xbf400000, 0xbf000000, 0x3f800000, 0, 0x3f800000,
];
const VIEWPORT: [u32; 6] = [0x4380_0000, 0x4340_0000, 0x3f000000, 0x44000000, 0x43c00000, 0x3f000000];
const TEXTURE: [u8; 16] = [10, 20, 30, 255, 40, 50, 60, 255, 70, 80, 90, 255, 100, 110, 120, 255];

pub(super) fn depth_texture_sequence(packet: &[u8]) -> Result<u32, String> {
    if packet.len() != 204 || packet.get(..4) != Some(b"VGD1")
        || [4, 12, 16, 20].into_iter().zip([13, 1024, 768, 3]).any(|(at, want)| read_u32(packet, at) != Some(want))
        || !words_are(packet, 24, &CLEAR) || !words_are(packet, 40, &[0, 0, 0, 0])
        || !words_are(packet, 56, &VERTICES) || !words_are(packet, 128, &VIEWPORT)
        || !words_are(packet, 152, &[448, 336, 128, 96]) || !words_are(packet, 168, &[0x1092, 2, 2])
        || packet.get(180..196) != Some(&TEXTURE) || !words_are(packet, 196, &[0x3f800000, 7])
    {
        return Err("guest emitted an invalid standard VirGL depth-texture packet".into());
    }
    read_u32(packet, 8).filter(|sequence| *sequence != 0)
        .ok_or_else(|| "VGD1 depth-texture packet has no nonzero sequence".into())
}

pub(crate) fn is_depth_texture_readback(packet: &[u8]) -> bool {
    frame_pixels(packet).is_some_and(|pixels| {
        let center = (384 * 1024 + 512) * 4;
        let clipped = (384 * 1024 + 400) * 4;
        pixels[..4] == [191, 128, 64, 255] && pixels[center..center + 4] == [10, 20, 30, 255]
            && pixels[clipped..clipped + 4] == [191, 128, 64, 255]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_thirteen_requires_exact_sampler_and_depth_dsa() {
        let mut packet = vec![0; 204];
        packet[..4].copy_from_slice(b"VGD1");
        put(&mut packet, 4, &[13, 99, 1024, 768, 3]); put(&mut packet, 24, &CLEAR);
        put(&mut packet, 56, &VERTICES); put(&mut packet, 128, &VIEWPORT);
        put(&mut packet, 152, &[448, 336, 128, 96]); put(&mut packet, 168, &[0x1092, 2, 2]);
        packet[180..196].copy_from_slice(&TEXTURE); put(&mut packet, 196, &[0x3f800000, 7]);
        assert_eq!(depth_texture_sequence(&packet), Ok(99));
        put(&mut packet, 200, &[5]);
        assert!(depth_texture_sequence(&packet).is_err());
    }

    fn put(packet: &mut [u8], offset: usize, values: &[u32]) {
        for (index, value) in values.iter().enumerate() {
            packet[offset + index * 4..offset + index * 4 + 4].copy_from_slice(&value.to_le_bytes());
        }
    }
}
