use super::{read_u32, words_are};

const CLEAR: [u32; 4] = [0x3e80_0000, 0x3f00_0000, 0x3f40_0000, 0x3f80_0000];
const VERTICES: [u32; 24] = [
    0, 0x3f40_0000, 0xbf00_0000, 0x3f800000, 0x3f800000, 0, 0, 0x3f800000,
    0xbf400000, 0xbf400000, 0xbf000000, 0x3f800000, 0, 0x3f800000, 0, 0x3f800000,
    0x3f400000, 0xbf400000, 0xbf000000, 0x3f800000, 0, 0, 0x3f800000, 0x3f800000,
];
const VIEWPORT: [u32; 6] = [0x43800000, 0x43400000, 0x3f000000, 0x44000000, 0x43c00000, 0x3f000000];

pub(super) fn depth_vertex_color_sequence(packet: &[u8]) -> Result<u32, String> {
    if packet.len() != 200 || packet.get(..4) != Some(b"VGD1")
        || [4, 12, 16, 20].into_iter().zip([12, 1024, 768, 3]).any(|(at, want)| read_u32(packet, at) != Some(want))
        || !words_are(packet, 24, &CLEAR) || !words_are(packet, 40, &[0, 0, 0, 0])
        || !words_are(packet, 56, &VERTICES) || !words_are(packet, 152, &VIEWPORT)
        || !words_are(packet, 176, &[448, 336, 128, 96])
        || read_u32(packet, 192) != Some(0x3f800000) || read_u32(packet, 196) != Some(5)
    {
        return Err("guest emitted an invalid standard VirGL depth vertex-color packet".into());
    }
    read_u32(packet, 8).filter(|sequence| *sequence != 0)
        .ok_or_else(|| "VGD1 depth vertex-color packet has no nonzero sequence".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_twelve_requires_its_exact_read_only_dsa_layout() {
        let mut packet = vec![0; 200];
        packet[..4].copy_from_slice(b"VGD1");
        put(&mut packet, 4, &[12, 99, 1024, 768, 3]); put(&mut packet, 24, &CLEAR);
        put(&mut packet, 56, &VERTICES); put(&mut packet, 152, &VIEWPORT);
        put(&mut packet, 176, &[448, 336, 128, 96]); put(&mut packet, 192, &[0x3f800000, 5]);
        assert_eq!(depth_vertex_color_sequence(&packet), Ok(99));
        put(&mut packet, 196, &[7]);
        assert!(depth_vertex_color_sequence(&packet).is_err());
    }

    fn put(packet: &mut [u8], offset: usize, values: &[u32]) {
        for (index, value) in values.iter().enumerate() {
            packet[offset + index * 4..offset + index * 4 + 4].copy_from_slice(&value.to_le_bytes());
        }
    }
}
