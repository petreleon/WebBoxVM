use super::{read_u32, words_are};
use super::draw::{CLEAR, INLINE, INLINE_VERTICES, UNIFORM, UNIFORM_VERTICES, VIEWPORT};

#[derive(Debug, PartialEq, Eq)]
pub(super) enum ResidentSolid { Draw(u32), Uniform(u32) }

pub(super) fn packet(packet: &[u8]) -> Result<ResidentSolid, String> {
    if packet.len() != 204 || packet.get(..4) != Some(b"VGB1")
        || [4, 12, 16, 20, 24].into_iter().zip([6, 1024, 768, 1, 1])
            .any(|(at, want)| read_u32(packet, at) != Some(want))
        || !words_are(packet, 28, &CLEAR) || read_u32(packet, 44) != Some(0)
        || read_u32(packet, 48) != Some(6) || !words_are(packet, 68, &VIEWPORT)
        || !words_are(packet, 92, &[448, 336, 128, 96])
    {
        return Err("guest emitted an invalid initial resident VGB1 packet".into());
    }
    let sequence = read_u32(packet, 8).filter(|sequence| *sequence != 0)
        .ok_or_else(|| "VGB1 resident packet has no nonzero sequence".to_string())?;
    match (words_are(packet, 52, &INLINE), words_are(packet, 108, &INLINE_VERTICES), words_are(packet, 52, &UNIFORM), words_are(packet, 108, &UNIFORM_VERTICES)) {
        (true, true, false, false) => Ok(ResidentSolid::Draw(sequence)),
        (false, false, true, true) => Ok(ResidentSolid::Uniform(sequence)),
        _ => Err("guest emitted an unrecognized initial resident VGB1 draw".into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn put(packet: &mut [u8], offset: usize, words: &[u32]) {
        for (index, word) in words.iter().enumerate() { packet[offset + index * 4..][..4].copy_from_slice(&word.to_le_bytes()); }
    }

    fn packet(color: &[u32], vertices: &[u32]) -> Vec<u8> {
        let mut packet = vec![0; 204]; packet[..4].copy_from_slice(b"VGB1");
        put(&mut packet, 4, &[6, 7, 1024, 768, 1, 1]); put(&mut packet, 28, &CLEAR);
        put(&mut packet, 48, &[6]); put(&mut packet, 52, color); put(&mut packet, 68, &VIEWPORT);
        put(&mut packet, 92, &[448, 336, 128, 96]); put(&mut packet, 108, vertices); packet
    }

    #[test]
    fn distinguishes_exact_initial_resident_draws() {
        assert_eq!(super::packet(&packet(&INLINE, &INLINE_VERTICES)), Ok(ResidentSolid::Draw(7)));
        assert_eq!(super::packet(&packet(&UNIFORM, &UNIFORM_VERTICES)), Ok(ResidentSolid::Uniform(7)));
        let mut invalid = packet(&INLINE, &INLINE_VERTICES); put(&mut invalid, 24, &[0]);
        assert!(super::packet(&invalid).is_err());
    }
}
