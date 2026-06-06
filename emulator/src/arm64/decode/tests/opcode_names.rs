use crate::arm64::Opcode;

#[test]
fn names_installer_trace_system_and_atomic_opcodes() {
    assert_eq!(Opcode::Mrs.name(), "Mrs");
    assert_eq!(Opcode::Ldxr.name(), "Ldxr");
    assert_eq!(Opcode::Stxp.name(), "Stxp");
    assert_eq!(Opcode::AtomicPair.name(), "AtomicPair");
    assert_eq!(Opcode::SimdCmeqReg.name(), "SimdCmeqReg");
    assert_eq!(Opcode::SimdCmhsReg.name(), "SimdCmhsReg");
    assert_eq!(Opcode::SimdShrn.name(), "SimdShrn");
}
