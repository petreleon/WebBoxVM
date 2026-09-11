use super::{read_u32, words_are};
use super::texture::{TEXTURE as TEXTURED_PIXELS, TextureMode, VERTICES as TEXTURED_VERTICES};
use super::texture_color::VERTICES as TEXTURE_COLOR_VERTICES;
use super::texture_pair::{LEFT, RIGHT, VERTICES as PAIR_VERTICES};
use super::vertex_color::VERTICES as VERTEX_COLOR_VERTICES;

const CLEAR: [u32; 4] = [0x3e80_0000, 0x3f00_0000, 0x3f40_0000, 0x3f80_0000];
const VIEWPORT: [u32; 6] = [0x4380_0000, 0x4340_0000, 0x3f00_0000, 0x4400_0000, 0x43c0_0000, 0x3f00_0000];
const SCISSOR: [u32; 4] = [448, 336, 128, 96];

#[derive(Debug, PartialEq, Eq)]
pub(super) enum ResidentMaterial {
    Texture(u32, TextureMode),
    Pair(u32),
    VertexColor(u32),
    TextureColor(u32),
}

pub(super) fn sequence(packet: &[u8]) -> Result<ResidentMaterial, String> {
    let sequence = read_u32(packet, 8)
        .filter(|sequence| *sequence != 0)
        .ok_or_else(|| "VGM1 resident material has no nonzero sequence".to_string())?;
    match (packet.len(), read_u32(packet, 48)) {
        (200, Some(3)) => texture(packet).map(|mode| ResidentMaterial::Texture(sequence, mode)),
        (228, Some(4)) if pair(packet) => Ok(ResidentMaterial::Pair(sequence)),
        (196, Some(2)) if vertex_color(packet) => Ok(ResidentMaterial::VertexColor(sequence)),
        (248, Some(5)) if texture_color(packet) => Ok(ResidentMaterial::TextureColor(sequence)),
        _ => Err("guest emitted an invalid initial resident material packet".into()),
    }
}

fn base(packet: &[u8], bytes: usize, kind: u32) -> bool {
    packet.len() == bytes && packet.get(..4) == Some(b"VGM1")
        && [4, 12, 16, 20, 24].into_iter().zip([2, 1024, 768, 1, 2])
            .all(|(at, want)| read_u32(packet, at) == Some(want))
        && words_are(packet, 28, &CLEAR) && read_u32(packet, 44) == Some(0)
        && words_are(packet, 48, &[kind, 0, 3]) && words_are(packet, 60, &VIEWPORT)
        && words_are(packet, 84, &SCISSOR)
}

fn texture(packet: &[u8]) -> Result<TextureMode, String> {
    let (mode, u) = match read_u32(packet, 100) {
        Some(0x1080) => (TextureMode::Repeat, 0x3f80_0000),
        Some(0x3292) => (TextureMode::Linear, 0x3f00_0000),
        _ => return Err("guest emitted an unsupported resident texture sampler".into()),
    };
    (base(packet, 200, 3) && words_are(packet, 104, &[2, 2])
        && packet.get(112..128) == Some(TEXTURED_PIXELS)
        && texture_vertices(packet, 128, u))
        .then_some(mode)
        .ok_or_else(|| "guest emitted an invalid resident textured packet".into())
}

fn pair(packet: &[u8]) -> bool {
    base(packet, 228, 4) && words_are(packet, 100, &[0x3292, 2, 2])
        && packet.get(112..128) == Some(LEFT) && words_are(packet, 128, &[0x1080, 2, 2])
        && packet.get(140..156) == Some(RIGHT) && words_are(packet, 156, PAIR_VERTICES)
}

fn vertex_color(packet: &[u8]) -> bool {
    base(packet, 196, 2) && words_are(packet, 100, VERTEX_COLOR_VERTICES)
}

fn texture_color(packet: &[u8]) -> bool {
    base(packet, 248, 5) && words_are(packet, 100, &[0x1092, 2, 2])
        && packet[112..128].chunks_exact(4).all(|pixel| pixel == [128, 128, 128, 255])
        && words_are(packet, 128, TEXTURE_COLOR_VERTICES)
}

fn texture_vertices(packet: &[u8], offset: usize, u: u32) -> bool {
    TEXTURED_VERTICES.iter().enumerate().all(|(index, expected)| {
        read_u32(packet, offset + index * 4) == Some(if matches!(index, 4 | 10 | 16) { u } else { *expected })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn put(packet: &mut [u8], offset: usize, values: &[u32]) {
        for (index, value) in values.iter().enumerate() {
            packet[offset + index * 4..][..4].copy_from_slice(&value.to_le_bytes());
        }
    }

    fn base_packet(bytes: usize, kind: u32) -> Vec<u8> {
        let mut packet = vec![0; bytes]; packet[..4].copy_from_slice(b"VGM1");
        put(&mut packet, 4, &[2, 7, 1024, 768, 1, 2]); put(&mut packet, 28, &CLEAR);
        put(&mut packet, 48, &[kind, 0, 3]); put(&mut packet, 60, &VIEWPORT); put(&mut packet, 84, &SCISSOR);
        packet
    }

    fn texture_packet(mode: TextureMode) -> Vec<u8> {
        let (sampler, u) = if mode == TextureMode::Repeat { (0x1080, 0x3f80_0000) } else { (0x3292, 0x3f00_0000) };
        let mut packet = base_packet(200, 3); put(&mut packet, 100, &[sampler, 2, 2]);
        packet[112..128].copy_from_slice(TEXTURED_PIXELS); texture_vertices_put(&mut packet, 128, u); packet
    }

    fn texture_vertices_put(packet: &mut [u8], offset: usize, u: u32) {
        for (index, value) in TEXTURED_VERTICES.iter().enumerate() {
            put(packet, offset + index * 4, &[if matches!(index, 4 | 10 | 16) { u } else { *value }]);
        }
    }

    #[test]
    fn recognizes_each_initial_resident_material_envelope() {
        assert_eq!(sequence(&texture_packet(TextureMode::Repeat)), Ok(ResidentMaterial::Texture(7, TextureMode::Repeat)));
        let mut pair_packet = base_packet(228, 4); put(&mut pair_packet, 100, &[0x3292, 2, 2]); pair_packet[112..128].copy_from_slice(LEFT); put(&mut pair_packet, 128, &[0x1080, 2, 2]); pair_packet[140..156].copy_from_slice(RIGHT); put(&mut pair_packet, 156, PAIR_VERTICES);
        assert_eq!(sequence(&pair_packet), Ok(ResidentMaterial::Pair(7)));
        let mut vertex_packet = base_packet(196, 2); put(&mut vertex_packet, 100, VERTEX_COLOR_VERTICES);
        assert_eq!(sequence(&vertex_packet), Ok(ResidentMaterial::VertexColor(7)));
        let mut color_packet = base_packet(248, 5); put(&mut color_packet, 100, &[0x1092, 2, 2]); for pixel in color_packet[112..128].chunks_exact_mut(4) { pixel.copy_from_slice(&[128, 128, 128, 255]); } put(&mut color_packet, 128, TEXTURE_COLOR_VERTICES);
        assert_eq!(sequence(&color_packet), Ok(ResidentMaterial::TextureColor(7)));
    }

    #[test]
    fn rejects_replacement_or_zero_sequence_material_envelopes() {
        let mut replacement = texture_packet(TextureMode::Linear); put(&mut replacement, 4, &[3]);
        assert!(sequence(&replacement).is_err()); let mut zero = texture_packet(TextureMode::Linear); put(&mut zero, 8, &[0]);
        assert!(sequence(&zero).is_err());
    }
}
