use super::*;

pub(super) fn opcode(raw: u32) -> Option<Opcode> {
    if (raw & 0xFFFF_F0FF) == 0xD503_30BF {
        Some(Opcode::Dmb)
    } else if (raw & 0xFFFF_F0FF) == 0xD503_309F || dsb_nxs(raw) {
        Some(Opcode::Dsb)
    } else if (raw & 0xFFFF_F0FF) == 0xD503_30DF {
        Some(Opcode::Isb)
    } else {
        None
    }
}

fn dsb_nxs(raw: u32) -> bool {
    (raw & 0xFFFF_F3FF) == 0xD503_323F
}
