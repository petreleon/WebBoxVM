use super::encoding::{encode_name, encode_u32, encode_u64};
use super::opcodes::*;

const SCRATCH_I64_LOCALS: u32 = 7;
const LOAD_HELPER_TYPE_INDEX: u32 = 0;
const STORE_HELPER_TYPE_INDEX: u32 = 1;
const SYSREG_HELPER_TYPE_INDEX: u32 = 2;
const EXCLUSIVE_PAIR_HELPER_TYPE_INDEX: u32 = 3;
const RUN_TYPE_INDEX: u32 = 4;
const RUN_FUNC_INDEX: u32 = 4;

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
    encode_u32(&mut section, 5);
    append_func_type(&mut section, &[TYPE_I64, TYPE_I32], &[TYPE_I64]);
    append_func_type(&mut section, &[TYPE_I64, TYPE_I32, TYPE_I64], &[]);
    append_func_type(&mut section, &[TYPE_I32], &[TYPE_I64]);
    append_func_type(
        &mut section,
        &[TYPE_I64, TYPE_I32, TYPE_I64, TYPE_I64],
        &[TYPE_I64],
    );
    append_func_type(&mut section, &[TYPE_I64], &[TYPE_I64]);
    section
}

fn append_func_type(section: &mut Vec<u8>, params: &[u8], results: &[u8]) {
    section.push(0x60);
    encode_u32(section, params.len() as u32);
    section.extend_from_slice(params);
    encode_u32(section, results.len() as u32);
    section.extend_from_slice(results);
}

fn import_section() -> Vec<u8> {
    let mut section = Vec::new();
    encode_u32(&mut section, 5);
    encode_name(&mut section, "env");
    encode_name(&mut section, "memory");
    section.push(IMPORT_MEMORY);
    encode_u32(&mut section, LIMITS_MEMORY64);
    encode_u64(&mut section, 0);
    encode_name(&mut section, "env");
    encode_name(&mut section, "jitLoadGuest");
    section.push(IMPORT_FUNC);
    encode_u32(&mut section, LOAD_HELPER_TYPE_INDEX);
    encode_name(&mut section, "env");
    encode_name(&mut section, "jitStoreGuest");
    section.push(IMPORT_FUNC);
    encode_u32(&mut section, STORE_HELPER_TYPE_INDEX);
    encode_name(&mut section, "env");
    encode_name(&mut section, "jitReadSysReg");
    section.push(IMPORT_FUNC);
    encode_u32(&mut section, SYSREG_HELPER_TYPE_INDEX);
    encode_name(&mut section, "env");
    encode_name(&mut section, "jitStoreExclusivePair");
    section.push(IMPORT_FUNC);
    encode_u32(&mut section, EXCLUSIVE_PAIR_HELPER_TYPE_INDEX);
    section
}

fn function_section() -> Vec<u8> {
    let mut section = Vec::new();
    encode_u32(&mut section, 1);
    encode_u32(&mut section, RUN_TYPE_INDEX);
    section
}

fn export_section() -> Vec<u8> {
    let mut section = Vec::new();
    encode_u32(&mut section, 1);
    encode_name(&mut section, "run");
    section.push(EXPORT_FUNC);
    encode_u32(&mut section, RUN_FUNC_INDEX);
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
