use crate::arch::arm64::Opcode;

pub(super) fn uses_guest_memory_helper(op: Opcode) -> bool {
    matches!(
        op,
        Opcode::DcZva
            | Opcode::Ldp
            | Opcode::Ldpsw
            | Opcode::Ldr
            | Opcode::LdrSign
            | Opcode::SimdLd1
            | Opcode::SimdLd1Multi
            | Opcode::SimdLdp
            | Opcode::SimdLdr
            | Opcode::SimdStr
            | Opcode::SimdStp
            | Opcode::Stp
            | Opcode::Str
    )
}
