use super::*;
use crate::constants::SYSREG_SP_EL0;

#[test]
fn compiles_observed_msr_sp_el0_as_staged_state_write() {
    let instr = crate::arch::arm64::decode(0xd518_411c).expect("decode observed MSR SP_EL0");
    assert_eq!(instr.op, Opcode::Msr);
    assert_eq!((instr.rd, instr.imm as u16), (28, SYSREG_SP_EL0));

    let module = Wasm64Compiler::compile(&block(vec![instr])).expect("compile MSR SP_EL0");

    assert_eq!(module.guest_instr_count, 1);
    assert!(!module.uses_guest_helpers);
    assert!(!run_body(&module.bytes).contains(&opcodes::OP_CALL));
    assert!(run_body(&module.bytes).contains(&opcodes::OP_I64_STORE));
}

#[test]
fn msr_sp_el0_stops_block_after_boundary() {
    let msr = crate::arch::arm64::decode(0xd518_411c).expect("decode observed MSR SP_EL0");
    let block = block(vec![
        instr(Opcode::Movz, 28, 0, 0, 5, true),
        msr,
        instr(Opcode::AddImm, 1, 0, 0, 7, true),
    ]);

    let module = Wasm64Compiler::compile(&block).expect("compile MSR SP_EL0 prefix");

    assert_eq!(module.guest_instr_count, 2);
    assert_eq!(module.exit_pc, 0x1008);
}

fn run_body(module: &[u8]) -> &[u8] {
    let mut offset = 8;
    while offset < module.len() {
        let section = module[offset];
        offset += 1;
        let (len, used) = read_leb(&module[offset..]);
        offset += used;
        if section == opcodes::SECTION_CODE {
            return &module[offset..offset + len as usize];
        }
        offset += len as usize;
    }
    &[]
}

fn read_leb(bytes: &[u8]) -> (u32, usize) {
    let mut value = 0u32;
    let mut shift = 0;
    for (index, byte) in bytes.iter().enumerate() {
        value |= ((byte & 0x7f) as u32) << shift;
        if byte & 0x80 == 0 {
            return (value, index + 1);
        }
        shift += 7;
    }
    panic!("unterminated leb");
}
