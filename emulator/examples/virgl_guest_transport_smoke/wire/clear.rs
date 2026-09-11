use super::{read_u32, words_are};

const COLOR: [u32; 4] = [0x3e80_0000, 0x3f00_0000, 0x3f40_0000, 0x3f80_0000];

pub(super) fn vgc1_sequence(packet: &[u8]) -> Result<u32, String> {
    let resident_candidate =
        read_u32(packet, 4) == Some(2) && packet.len() == 40 && read_u32(packet, 36) == Some(0);
    let standard = read_u32(packet, 4) == Some(1) && packet.len() == 36;
    if !matches!(packet.get(..4), Some(b"VGC1"))
        || !(standard || resident_candidate)
        || read_u32(packet, 12) != Some(1024)
        || read_u32(packet, 16) != Some(768)
        || !words_are(packet, 20, &COLOR)
    {
        return Err("guest emitted an invalid VirGL clear envelope".into());
    }
    read_u32(packet, 8)
        .filter(|sequence| *sequence != 0)
        .ok_or_else(|| "VGC1 packet has no nonzero sequence".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn packet(version: u32, predecessor: Option<u32>) -> Vec<u8> {
        let mut packet = b"VGC1".to_vec();
        for word in [version, 7, 1024, 768].into_iter().chain(COLOR) {
            packet.extend_from_slice(&word.to_le_bytes());
        }
        if let Some(predecessor) = predecessor {
            packet.extend_from_slice(&predecessor.to_le_bytes());
        }
        packet
    }

    #[test]
    fn accepts_standard_and_initial_resident_clear_envelopes() {
        assert_eq!(vgc1_sequence(&packet(1, None)), Ok(7));
        assert_eq!(vgc1_sequence(&packet(2, Some(0))), Ok(7));
    }

    #[test]
    fn rejects_resident_candidate_with_a_predecessor() {
        assert!(vgc1_sequence(&packet(2, Some(6))).is_err());
    }
}
