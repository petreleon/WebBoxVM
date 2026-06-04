use super::*;

pub(super) fn map(m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    Some(match m {
        M::r#aese => Opcode::SimdAese,
        M::r#aesd => Opcode::SimdAesd,
        M::r#aesmc => Opcode::SimdAesmc,
        M::r#aesimc => Opcode::SimdAesimc,
        M::r#pmull | M::r#pmull2 => Opcode::SimdPmull,
        M::r#sha1h => Opcode::SimdSha1h,
        M::r#sha256su0 => Opcode::SimdSha256Su0,
        M::r#sha512su0 => Opcode::SimdSha512Su0,
        M::r#sha512h => Opcode::SimdSha512H,
        M::r#sha512h2 => Opcode::SimdSha512H2,
        M::r#sha512su1 => Opcode::SimdSha512Su1,
        M::r#sha1c => Opcode::SimdSha1C,
        M::r#sha1m => Opcode::SimdSha1M,
        M::r#sha1p => Opcode::SimdSha1P,
        M::r#sha1su0 => Opcode::SimdSha1Su0,
        M::r#sha1su1 => Opcode::SimdSha1Su1,
        M::r#sha256h => Opcode::SimdSha256H,
        M::r#sha256h2 => Opcode::SimdSha256H2,
        M::r#sha256su1 => Opcode::SimdSha256Su1,
        M::r#sm4e => Opcode::SimdSm4e,
        M::r#sm3partw1 => Opcode::SimdSm3Partw1,
        M::r#sm3partw2 => Opcode::SimdSm3Partw2,
        M::r#sm3ss1 => Opcode::SimdSm3Ss1,
        M::r#sm3tt1a => Opcode::SimdSm3Tt1A,
        M::r#sm3tt1b => Opcode::SimdSm3Tt1B,
        M::r#sm3tt2a => Opcode::SimdSm3Tt2A,
        M::r#sm3tt2b => Opcode::SimdSm3Tt2B,
        M::r#eor3 => Opcode::SimdEor3,
        M::r#bcax => Opcode::SimdBcax,
        M::r#rax1 => Opcode::SimdRax1,
        M::r#xar => Opcode::SimdXar,
        _ => return None,
    })
}
