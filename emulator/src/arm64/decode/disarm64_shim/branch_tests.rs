use super::*;

#[test]
fn maps_conditional_branch_mnemonic_separately_from_unconditional() {
    let conditional = decoder::decode(0x5400_002A).expect("disarm64 should decode b.ge");
    let unconditional = decoder::decode(0x1400_0000).expect("disarm64 should decode b");

    assert_eq!(format!("{:?}", conditional.mnemonic), "b_");
    assert_eq!(
        mnemonic_to_opcode(0x5400_002A, conditional.mnemonic),
        Some(Opcode::BCond)
    );

    assert_eq!(format!("{:?}", unconditional.mnemonic), "b");
    assert_eq!(
        mnemonic_to_opcode(0x1400_0000, unconditional.mnemonic),
        Some(Opcode::B)
    );
}
