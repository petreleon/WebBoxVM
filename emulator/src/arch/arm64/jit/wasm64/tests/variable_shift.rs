use super::*;

#[test]
fn compiles_variable_shift_family() {
    let block = block(vec![
        instr(Opcode::Lslv, 1, 2, 3, 0, true),
        instr(Opcode::Lsrv, 4, 5, 6, 0, false),
        instr(Opcode::Asrv, 7, 8, 9, 0, false),
        instr(Opcode::Rorv, 10, 11, 12, 0, true),
    ]);

    let module = Wasm64Compiler::compile(&block).expect("compile variable shifts");

    assert_eq!(module.guest_instr_count, 4);
    assert!(module.bytes.contains(&opcodes::OP_I64_SHL));
    assert!(module.bytes.contains(&opcodes::OP_I64_SHR_U));
    assert!(module.bytes.contains(&opcodes::OP_I64_SHR_S));
    assert!(module.bytes.contains(&opcodes::OP_I64_ROTR));
}

#[test]
fn compiles_32_bit_rotate_right_with_width_mask() {
    let block = block(vec![instr(Opcode::Rorv, 1, 2, 3, 0, false)]);

    let module = Wasm64Compiler::compile(&block).expect("compile rorv32");

    assert_eq!(module.guest_instr_count, 1);
    assert!(module.bytes.contains(&opcodes::OP_LOCAL_SET));
    assert!(module.bytes.contains(&opcodes::OP_I64_OR));
}
