use super::*;

#[test]
fn compiles_dc_zva_as_two_staged_zero_stores() {
    let block = block(vec![instr(Opcode::DcZva, 0, 0, 0, 0, true)]);

    let module = Wasm64Compiler::compile(&block).expect("compile dc zva");
    let calls = module
        .bytes
        .iter()
        .filter(|&&byte| byte == opcodes::OP_CALL)
        .count();

    assert_eq!(module.guest_instr_count, 1);
    assert!(module.uses_guest_helpers);
    assert_eq!(module.exit_pc, 0x1004);
    assert!(calls >= 2);
}

#[test]
fn dc_zva_stops_block_after_zero_boundary() {
    let block = block(vec![
        instr(Opcode::Movz, 0, 0, 0, 5, true),
        instr(Opcode::DcZva, 0, 0, 0, 0, true),
        instr(Opcode::AddImm, 1, 0, 0, 7, true),
    ]);

    let module = Wasm64Compiler::compile(&block).expect("compile dc zva prefix");

    assert_eq!(module.guest_instr_count, 2);
    assert!(module.uses_guest_helpers);
    assert_eq!(module.exit_pc, 0x1008);
}
