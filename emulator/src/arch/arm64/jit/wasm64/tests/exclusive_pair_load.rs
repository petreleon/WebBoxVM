use super::*;

#[test]
fn compiles_observed_ldxp_with_exclusive_pair_helper() {
    let instr = crate::arch::arm64::decode(0xc87f_0480).expect("decode observed ldxp");
    assert_eq!(instr.op, Opcode::Ldxp);
    assert_eq!((instr.rd, instr.rm, instr.rn, instr.sf), (0, 1, 4, true));

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile ldxp");

    assert_eq!(module.guest_instr_count, 1);
    assert!(module.uses_guest_helpers);
    assert!(
        module
            .bytes
            .windows(b"jitLoadExclusivePair".len())
            .any(|w| w == b"jitLoadExclusivePair")
    );
}

#[test]
fn exclusive_pair_load_stops_block_after_boundary() {
    let ldxp = crate::arch::arm64::decode(0xc87f_0480).expect("decode observed ldxp");
    let block = block(vec![
        instr(Opcode::Movz, 0, 0, 0, 5, true),
        ldxp,
        instr(Opcode::AddImm, 1, 0, 0, 7, true),
    ]);

    let module = Wasm64Compiler::compile(&block).expect("compile ldxp prefix");

    assert_eq!(module.guest_instr_count, 2);
    assert_eq!(module.exit_pc, 0x1008);
}
