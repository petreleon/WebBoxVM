use super::*;

fn store_instr(size: u8, sf: bool, cond: u8, imm: u64) -> Instr {
    Instr {
        cond,
        imm,
        size,
        ..instr(Opcode::Str, 2, 1, 0xFF, 0, sf)
    }
}

#[test]
fn compiles_scalar_store_with_helper_import() {
    let block = block(vec![store_instr(8, true, 0, 16)]);

    let module = Wasm64Compiler::compile(&block).expect("compile str");

    assert_eq!(module.guest_instr_count, 1);
    assert!(module.uses_guest_helpers);
    assert!(
        module
            .bytes
            .windows(b"jitStoreGuest".len())
            .any(|w| w == b"jitStoreGuest")
    );
    assert!(module.bytes.contains(&opcodes::OP_CALL));
}

#[test]
fn scalar_store_stops_block_after_store_boundary() {
    let block = block(vec![
        instr(Opcode::Movz, 0, 0, 0, 5, true),
        store_instr(4, false, 1, 4),
        instr(Opcode::AddImm, 1, 0, 0, 7, true),
    ]);

    let module = Wasm64Compiler::compile(&block).expect("compile store prefix");

    assert_eq!(module.guest_instr_count, 2);
    assert!(module.uses_guest_helpers);
    assert_eq!(module.exit_pc, 0x1008);
}
