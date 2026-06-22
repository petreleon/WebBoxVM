use super::*;

fn load_instr(op: Opcode, size: u8, sf: bool, cond: u8, imm: u64) -> Instr {
    Instr {
        cond,
        imm,
        size,
        ..instr(op, 2, 1, 0xFF, 0, sf)
    }
}

#[test]
fn compiles_unsigned_scalar_load_with_helper_import() {
    let block = block(vec![load_instr(Opcode::Ldr, 4, false, 0, 16)]);

    let module = Wasm64Compiler::compile(&block).expect("compile ldr");

    assert_eq!(module.guest_instr_count, 1);
    assert!(module.uses_guest_helpers);
    assert!(
        module
            .bytes
            .windows(b"jitLoadGuest".len())
            .any(|w| w == b"jitLoadGuest")
    );
    assert!(module.bytes.contains(&opcodes::OP_CALL));
}

#[test]
fn compiles_signed_load_sign_extension() {
    let block = block(vec![load_instr(Opcode::LdrSign, 1, true, 0, 8)]);

    let module = Wasm64Compiler::compile(&block).expect("compile ldrsb");

    assert_eq!(module.guest_instr_count, 1);
    assert!(module.uses_guest_helpers);
    assert!(module.bytes.contains(&opcodes::OP_I64_SHL));
    assert!(module.bytes.contains(&opcodes::OP_I64_SHR_S));
}

#[test]
fn compiles_post_index_load_writeback() {
    let block = block(vec![load_instr(Opcode::Ldr, 8, true, 1, 24)]);

    let module = Wasm64Compiler::compile(&block).expect("compile post-index ldr");

    assert_eq!(module.guest_instr_count, 1);
    assert!(module.uses_guest_helpers);
    assert!(module.bytes.contains(&opcodes::OP_I64_STORE));
}

#[test]
fn compiles_observed_ldar_as_guest_load() {
    let instr = crate::arch::arm64::decode(0x88df_fe93).expect("decode observed ldar");
    assert_eq!(instr.op, Opcode::Ldar);
    assert_eq!(
        (instr.rd, instr.rn, instr.size, instr.sf),
        (19, 20, 4, false)
    );

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile ldar");

    assert_eq!(module.guest_instr_count, 1);
    assert!(module.uses_guest_helpers);
    assert!(
        module
            .bytes
            .windows(b"jitLoadGuest".len())
            .any(|w| w == b"jitLoadGuest")
    );
}

#[test]
fn compiles_ldar_writeback_form() {
    let instr = crate::arch::arm64::decode(0x99c0_0820).expect("decode ldapr writeback");
    assert_eq!(instr.op, Opcode::Ldar);
    assert_eq!((instr.rd, instr.rn, instr.cond, instr.imm), (0, 1, 2, 4));

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile ldar writeback");

    assert_eq!(module.guest_instr_count, 1);
    assert!(module.uses_guest_helpers);
    assert!(module.bytes.contains(&opcodes::OP_I64_STORE));
}
