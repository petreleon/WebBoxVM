use crate::arm64::Opcode;

#[test]
fn names_installer_trace_system_and_atomic_opcodes() {
    assert_eq!(Opcode::Mrs.name(), "Mrs");
    assert_eq!(Opcode::Ldxr.name(), "Ldxr");
    assert_eq!(Opcode::Stxp.name(), "Stxp");
    assert_eq!(Opcode::AtomicPair.name(), "AtomicPair");
    assert_eq!(Opcode::Eret.name(), "Eret");
    assert_eq!(Opcode::Prfm.name(), "Prfm");
    assert_eq!(Opcode::SimdLdp.name(), "SimdLdp");
    assert_eq!(Opcode::SimdStp.name(), "SimdStp");
    assert_eq!(Opcode::SimdStr.name(), "SimdStr");
    assert_eq!(Opcode::SimdCmeqReg.name(), "SimdCmeqReg");
    assert_eq!(Opcode::SimdCmhsReg.name(), "SimdCmhsReg");
    assert_eq!(Opcode::SimdAddhn.name(), "SimdAddhn");
    assert_eq!(Opcode::SimdShrn.name(), "SimdShrn");
    assert_eq!(Opcode::Paciasp.name(), "Paciasp");
    assert_eq!(Opcode::BtiC.name(), "BtiC");
}
