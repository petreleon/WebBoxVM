use super::*;

#[test]
fn compiles_scalar_bitfield_prefix() {
    let block = block(vec![
        instr(Opcode::Ubfm, 8, 9, 8, 15, false),
        instr(Opcode::Sbfm, 6, 7, 0, 15, false),
        instr(Opcode::Bfm, 4, 5, 48, 55, true),
    ]);

    let module = Wasm64Compiler::compile(&block).expect("compile bitfield prefix");

    assert_eq!(module.guest_instr_count, 3);
    assert!(module.bytes.contains(&opcodes::OP_I64_AND));
    assert!(module.bytes.contains(&opcodes::OP_I64_SHL));
    assert!(module.bytes.contains(&opcodes::OP_I64_SHR_S));
    assert!(module.bytes.contains(&opcodes::OP_I64_SHR_U));
}

#[test]
fn unsupported_32_bit_bitfield_range_ends_compiled_prefix() {
    let block = block(vec![
        instr(Opcode::Movz, 0, 0, 0, 5, true),
        instr(Opcode::Ubfm, 8, 9, 32, 15, false),
    ]);

    let module = Wasm64Compiler::compile(&block).expect("compile prefix");

    assert_eq!(module.guest_instr_count, 1);
    assert_eq!(module.exit_pc, 0x1004);
}
