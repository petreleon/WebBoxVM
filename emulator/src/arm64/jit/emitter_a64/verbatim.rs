use crate::arm64::opcodes::Opcode;

pub(super) fn can_emit_verbatim(op: Opcode) -> bool {
    matches!(
        op,
        Opcode::Add
            | Opcode::Sub
            | Opcode::Adds
            | Opcode::Subs
            | Opcode::AddImm
            | Opcode::SubImm
            | Opcode::AddsImm
            | Opcode::SubsImm
            | Opcode::AndReg
            | Opcode::OrrReg
            | Opcode::EorReg
            | Opcode::AndsReg
            | Opcode::AndImm
            | Opcode::OrrImm
            | Opcode::EorImm
            | Opcode::AndsImm
            | Opcode::MovReg
            | Opcode::Movz
            | Opcode::Movk
            | Opcode::Movn
            | Opcode::Cmp
            | Opcode::CmpImm
            | Opcode::Sxtw
            | Opcode::Sbfm
            | Opcode::Bfm
            | Opcode::Ubfm
            | Opcode::Csel
            | Opcode::Csinc
            | Opcode::Csinv
            | Opcode::Csneg
            | Opcode::Ccmp
            | Opcode::Udiv
            | Opcode::Sdiv
            | Opcode::Madd
            | Opcode::Msub
            | Opcode::Umulh
            | Opcode::Smulh
            | Opcode::Lslv
            | Opcode::Lsrv
            | Opcode::Asrv
            | Opcode::Rorv
            | Opcode::Rev
            | Opcode::Rev32
            | Opcode::Rev16
            | Opcode::Rbit
            | Opcode::Clz
            | Opcode::Nop
            | Opcode::NopBarrier
            | Opcode::AddExt
            | Opcode::SubExt
            | Opcode::AddsExt
            | Opcode::SubsExt
    )
}
