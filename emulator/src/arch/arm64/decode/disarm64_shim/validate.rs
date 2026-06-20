use super::*;
use std::sync::OnceLock;

pub(super) fn validate(raw: u32, legacy: Instr) -> Option<Instr> {
    let log = log_mismatches();
    let strict = strict_mismatches();
    if !log && !strict {
        return Some(legacy);
    }

    if let Some(expected) = mapped_mismatch(raw, legacy.op) {
        eprintln!(
            "DISARM64 MISMATCH: raw=0x{raw:08x} legacy={:?} disarm64={expected:?}",
            legacy.op
        );
        if strict {
            panic!(
                "DISARM64 STRICT MISMATCH raw=0x{raw:08x} legacy={:?} disarm64={expected:?}",
                legacy.op
            );
        }
    }

    Some(legacy)
}

fn mapped_mismatch(raw: u32, legacy: Opcode) -> Option<Opcode> {
    let d64 = decoder::decode(raw)?;
    let expected = mnemonic_to_opcode(raw, d64.mnemonic)?;
    (legacy != expected).then_some(expected)
}

fn log_mismatches() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("WEBBOXVM_DISARM64_MISMATCHES").is_some())
}

fn strict_mismatches() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("WEBBOXVM_DISARM64_STRICT").is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapped_mismatch_detects_known_opcode_disagreement() {
        assert_eq!(
            mapped_mismatch(0x9343_3020, Opcode::Ubfm),
            Some(Opcode::Sbfm)
        );
    }

    #[test]
    fn mapped_mismatch_ignores_matching_opcode() {
        assert_eq!(mapped_mismatch(0x9343_3020, Opcode::Sbfm), None);
    }
}
