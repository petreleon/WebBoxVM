use super::*;

pub(super) fn map(raw: u32, m: disarm64::decoder::Mnemonic) -> Option<Opcode> {
    use disarm64::decoder::Mnemonic as M;
    Some(match m {
        M::r#cas | M::r#casa | M::r#casal | M::r#casl if legacy_cas(raw) => Opcode::Cas,
        M::r#casp | M::r#caspa | M::r#caspal | M::r#caspl if legacy_casp(raw) => Opcode::Casp,
        M::r#swpp | M::r#swppa | M::r#swppal | M::r#swppl if legacy_lse128_pair(raw) => {
            Opcode::AtomicPair
        }
        M::r#ldsetp | M::r#ldsetpa | M::r#ldsetpal | M::r#ldsetpl if legacy_lse128_pair(raw) => {
            Opcode::AtomicPair
        }
        M::r#ldclrp | M::r#ldclrpa | M::r#ldclrpal | M::r#ldclrpl if legacy_lse128_pair(raw) => {
            Opcode::AtomicPair
        }
        m if legacy_single_atomic(raw) && super::atomic_mnemonics::single_atomic(m) => {
            Opcode::Atomic
        }
        _ => return None,
    })
}

fn legacy_cas(raw: u32) -> bool {
    (raw & 0x3FA0_7C00) == 0x08A0_7C00 && ((raw >> 30) & 3) >= 2
}

fn legacy_casp(raw: u32) -> bool {
    (raw & 0x3F20_7C00) == 0x0820_7C00 && ((raw >> 30) & 3) <= 1
}

fn legacy_lse128_pair(raw: u32) -> bool {
    let rt = raw & 0x1F;
    let rs = (raw >> 16) & 0x1F;
    let atomic_op = (raw >> 12) & 0xF;
    (raw & 0xFF20_0C00) == 0x1920_0000
        && matches!(atomic_op, 0x1 | 0x3 | 0x8)
        && rt != 31
        && rs != 31
        && rt != rs
}

fn legacy_single_atomic(raw: u32) -> bool {
    (raw & 0x3F20_0C00) == 0x3820_0000
}
