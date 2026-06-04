use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    Some(match m {
        M::r#hint if raw == 0xD503_205F => Opcode::Wfe,
        M::r#hint if raw == 0xD503_207F => Opcode::Wfi,
        M::r#hint => Opcode::Nop,
        M::r#clrex if raw == 0xD503_305F => Opcode::NopBarrier,
        M::r#dmb | M::r#dsb | M::r#isb if legacy_barrier(raw) => Opcode::NopBarrier,
        M::r#mrs if ((raw >> 20) & 0xFFF) == 0xD53 => Opcode::Mrs,
        M::r#msr if ((raw >> 20) & 0xFFF) == 0xD51 => Opcode::Msr,
        M::r#msr if legacy_daif_alias(raw) => Opcode::Nop,
        M::r#sys if (raw & 0xFFFF_FFE0) == 0xD50B_7420 => Opcode::DcZva,
        M::r#sys if legacy_tlbi(raw) => Opcode::Tlbi,
        M::r#svc => Opcode::Svc,
        M::r#brk => Opcode::Brk,
        M::r#udf => Opcode::Udf,
        M::r#eret => Opcode::Eret,
        _ => return None,
    })
}

fn legacy_barrier(raw: u32) -> bool {
    matches!(
        raw,
        0xD503_309F
            | 0xD503_30BF
            | 0xD503_30DF
            | 0xD503_39BF
            | 0xD503_3BBF
            | 0xD503_3F9F
            | 0xD503_3FDF
    )
}

fn legacy_daif_alias(raw: u32) -> bool {
    (raw & 0xFFFF_F01F) == 0xD503_401F && matches!((raw >> 5) & 0x7, 0b110 | 0b111)
}

fn legacy_tlbi(raw: u32) -> bool {
    let op0 = (raw >> 19) & 0x3;
    let l = (raw >> 21) & 1;
    let crn = (raw >> 12) & 0xF;
    l == 0 && op0 == 1 && crn == 8
}
