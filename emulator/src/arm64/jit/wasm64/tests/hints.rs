use super::*;

#[test]
fn compiles_noop_hint_aliases() {
    let block = block(vec![
        instr(Opcode::Autiasp, 0, 0, 0, 0, true),
        instr(Opcode::BtiC, 0, 0, 0, 0, true),
        instr(Opcode::Yield, 0, 0, 0, 0, true),
    ]);

    let module = Wasm64Compiler::compile(&block).expect("compile no-op aliases");

    assert_eq!(module.guest_instr_count, 3);
    assert_eq!(module.exit_pc, 0x100c);
}
