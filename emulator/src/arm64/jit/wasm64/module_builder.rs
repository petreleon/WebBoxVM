use super::encoding::{encode_name, encode_u32, encode_u64};
use super::opcodes::*;

const SCRATCH_I64_LOCALS: u32 = 3;

pub(super) fn build_module(expr: Vec<u8>) -> Vec<u8> {
    let mut module = Vec::new();
    module.extend_from_slice(b"\0asm");
    module.extend_from_slice(&[1, 0, 0, 0]);
    append_section(&mut module, SECTION_TYPE, type_section());
    append_section(&mut module, SECTION_IMPORT, import_section());
    append_section(&mut module, SECTION_FUNCTION, function_section());
    append_section(&mut module, SECTION_EXPORT, export_section());
    append_section(&mut module, SECTION_CODE, code_section(expr));
    module
}

fn type_section() -> Vec<u8> {
    let mut section = Vec::new();
    encode_u32(&mut section, 1);
    section.push(0x60);
    encode_u32(&mut section, 1);
    section.push(TYPE_I64);
    encode_u32(&mut section, 1);
    section.push(TYPE_I64);
    section
}

fn import_section() -> Vec<u8> {
    let mut section = Vec::new();
    encode_u32(&mut section, 1);
    encode_name(&mut section, "env");
    encode_name(&mut section, "memory");
    section.push(IMPORT_MEMORY);
    encode_u32(&mut section, LIMITS_MEMORY64);
    encode_u64(&mut section, 0);
    section
}

fn function_section() -> Vec<u8> {
    let mut section = Vec::new();
    encode_u32(&mut section, 1);
    encode_u32(&mut section, 0);
    section
}

fn export_section() -> Vec<u8> {
    let mut section = Vec::new();
    encode_u32(&mut section, 1);
    encode_name(&mut section, "run");
    section.push(EXPORT_FUNC);
    encode_u32(&mut section, 0);
    section
}

fn code_section(expr: Vec<u8>) -> Vec<u8> {
    let mut body = Vec::new();
    encode_u32(&mut body, 1);
    encode_u32(&mut body, SCRATCH_I64_LOCALS);
    body.push(TYPE_I64);
    body.extend_from_slice(&expr);

    let mut section = Vec::new();
    encode_u32(&mut section, 1);
    encode_u32(&mut section, body.len() as u32);
    section.extend_from_slice(&body);
    section
}

fn append_section(module: &mut Vec<u8>, id: u8, section: Vec<u8>) {
    module.push(id);
    encode_u32(module, section.len() as u32);
    module.extend_from_slice(&section);
}
