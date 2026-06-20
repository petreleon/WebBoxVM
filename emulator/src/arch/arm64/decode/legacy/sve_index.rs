use super::*;

const MASK: u32 = 0xFF20_FC00;
const INDEX_II: u32 = 0x0420_4000;
const INDEX_IR: u32 = 0x0420_4800;
const INDEX_RI: u32 = 0x0420_4400;
const INDEX_RR: u32 = 0x0420_4C00;

pub(super) fn decode(raw: u32) -> DecodeStep {
    let form = raw & MASK;
    if !matches!(form, INDEX_II | INDEX_IR | INDEX_RI | INDEX_RR) {
        return DecodeStep::Miss;
    }

    let start_imm = if matches!(form, INDEX_II | INDEX_IR) {
        simm5(raw >> 5)
    } else {
        0
    };
    let step_imm = if matches!(form, INDEX_II | INDEX_RI) {
        simm5(raw >> 16)
    } else {
        0
    };

    DecodeStep::Hit(Instr {
        op: Opcode::SveIndex,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: ((raw >> 16) & 0x1F) as u8,
        imm: pack_imms(start_imm, step_imm),
        sf: ((raw >> 22) & 0x3) == 3,
        cond: index_mode(form),
        size: 1u8 << (((raw >> 22) & 0x3) as u8),
    })
}

fn index_mode(form: u32) -> u8 {
    let start_is_imm = matches!(form, INDEX_II | INDEX_IR) as u8;
    let step_is_imm = matches!(form, INDEX_II | INDEX_RI) as u8;
    start_is_imm | (step_is_imm << 1)
}

fn simm5(value: u32) -> i64 {
    (((value & 0x1F) as i8) << 3 >> 3) as i64
}

fn pack_imms(start: i64, step: i64) -> u64 {
    (start as i32 as u32 as u64) | ((step as i32 as u32 as u64) << 32)
}
