use super::*;

pub(super) fn is_nop_like(op: Opcode) -> bool {
    matches!(
        op,
        Opcode::Nop
            | Opcode::NopBarrier
            | Opcode::Chkfeat
            | Opcode::GcsPushM
            | Opcode::GcsPushX
            | Opcode::GcsPopM
            | Opcode::GcsPopX
            | Opcode::GcsPopCx
            | Opcode::GcsSs1
            | Opcode::GcsSs2
            | Opcode::Smstop
            | Opcode::Pacia1716
            | Opcode::Pacib1716
            | Opcode::Autia1716
            | Opcode::Autib1716
            | Opcode::Paciaz
            | Opcode::Paciasp
            | Opcode::Pacibz
            | Opcode::Pacibsp
            | Opcode::Autiaz
            | Opcode::Autiasp
            | Opcode::Autibz
            | Opcode::Autibsp
            | Opcode::Xpaclri
            | Opcode::Bti
            | Opcode::BtiC
            | Opcode::BtiJ
            | Opcode::BtiJc
            | Opcode::Sev
            | Opcode::Sevl
            | Opcode::Esb
            | Opcode::PsbCsync
            | Opcode::TsbCsync
            | Opcode::GcsbDsync
            | Opcode::Csdb
            | Opcode::Clrbhb
            | Opcode::Yield
            | Opcode::Dgh
            | Opcode::Sb
            | Opcode::DaifSet
            | Opcode::DaifClr
            | Opcode::Prfm
            | Opcode::Clrex
            | Opcode::Dmb
            | Opcode::Dsb
            | Opcode::Isb
    )
}
