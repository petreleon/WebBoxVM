use super::encoding::encode_u32;
use super::module_prefix::module_prefix;
use super::opcodes::*;

const SCRATCH_I64_LOCALS: u32 = 7;

pub(super) fn build_module(expr: Vec<u8>, imports_helpers: bool) -> Vec<u8> {
    let prefix = module_prefix(imports_helpers);
    let mut module = Vec::with_capacity(prefix.len() + expr.len() + 16);
    module.extend_from_slice(prefix);
    append_code_section(&mut module, expr);
    module
}

fn append_code_section(module: &mut Vec<u8>, expr: Vec<u8>) {
    let local_decls_len = encoded_u32_len(1) + encoded_u32_len(SCRATCH_I64_LOCALS) + 1;
    let body_len = local_decls_len + expr.len();
    let section_len = encoded_u32_len(1) + encoded_u32_len(body_len as u32) + body_len;

    module.push(SECTION_CODE);
    encode_u32(module, section_len as u32);
    encode_u32(module, 1);
    encode_u32(module, body_len as u32);
    encode_u32(module, 1);
    encode_u32(module, SCRATCH_I64_LOCALS);
    module.push(TYPE_I64);
    module.extend_from_slice(&expr);
}

fn encoded_u32_len(mut value: u32) -> usize {
    let mut len = 1;
    while value >= 0x80 {
        value >>= 7;
        len += 1;
    }
    len
}
