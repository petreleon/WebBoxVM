use super::*;

pub(super) fn map(m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    Some(match m {
        M::r#ldxr | M::r#ldxrb | M::r#ldxrh | M::r#ldaxr | M::r#ldaxrb | M::r#ldaxrh => {
            Opcode::Ldxr
        }
        M::r#ldar | M::r#ldarb | M::r#ldarh | M::r#ldapr => Opcode::Ldar,
        M::r#stxr | M::r#stxrb | M::r#stxrh | M::r#stlxr | M::r#stlxrb | M::r#stlxrh => {
            Opcode::Stxr
        }
        M::r#stlr | M::r#stlrb | M::r#stlrh => Opcode::Stlr,
        M::r#ldxp | M::r#ldaxp => Opcode::Ldxp,
        M::r#stxp | M::r#stlxp => Opcode::Stxp,
        _ => return None,
    })
}
