pub(super) fn emit_a64(code: &mut Vec<u8>, instr: u32) {
    code.extend_from_slice(&instr.to_le_bytes());
}

pub(super) fn emit_mov(code: &mut Vec<u8>, rd: u8, rm: u8) {
    emit_a64(code, 0xAA0003E0 | ((rm as u32) << 16) | (rd as u32));
}

pub(super) fn encode_ldp_offset(off: usize) -> u32 {
    ((off as u32 / 8) & 0x7F) << 15
}

pub(super) fn emit_prologue(code: &mut Vec<u8>) {
    emit_a64(code, 0xA9BF7BFD);
    emit_a64(code, 0x910003FD);
    emit_a64(code, 0xA9BF4FF3);
    emit_a64(code, 0xA9BF57F5);
    emit_a64(code, 0xA9BF5FF7);
    emit_a64(code, 0xA9BF67F9);
    emit_a64(code, 0xA9BF6FFB);
}

pub(super) fn emit_epilogue(code: &mut Vec<u8>) {
    emit_a64(code, 0xA8C16FFB);
    emit_a64(code, 0xA8C167F9);
    emit_a64(code, 0xA8C15FF7);
    emit_a64(code, 0xA8C157F5);
    emit_a64(code, 0xA8C14FF3);
    emit_a64(code, 0xA8C17BFD);
    emit_a64(code, 0xD65F03C0);
}
