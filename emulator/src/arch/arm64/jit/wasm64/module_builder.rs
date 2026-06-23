use super::encoding::{encode_name, encode_u32, encode_u64};
use super::opcodes::*;
use std::sync::LazyLock;

const SCRATCH_I64_LOCALS: u32 = 7;
const LOAD_HELPER_TYPE_INDEX: u32 = 0;
const STORE_HELPER_TYPE_INDEX: u32 = 1;
const SYSREG_HELPER_TYPE_INDEX: u32 = 2;
const EXCLUSIVE_PAIR_HELPER_TYPE_INDEX: u32 = 3;
const EXCLUSIVE_STORE_HELPER_TYPE_INDEX: u32 = 4;
const RUN_TYPE_INDEX: u32 = 5;
const PAIR_STORE_HELPER_TYPE_INDEX: u32 = 6;
const PAIR_LOAD_HELPER_TYPE_INDEX: u32 = 7;
const QUAD_STORE_HELPER_TYPE_INDEX: u32 = 8;
const QUAD_LOAD_HELPER_TYPE_INDEX: u32 = 9;
const RUN_FUNC_INDEX: u32 = 10;

static MODULE_PREFIX_BYTES: LazyLock<Vec<u8>> = LazyLock::new(module_prefix);

pub(super) fn build_module(expr: Vec<u8>) -> Vec<u8> {
    let prefix = MODULE_PREFIX_BYTES.as_slice();
    let mut module = Vec::with_capacity(prefix.len() + expr.len() + 16);
    module.extend_from_slice(prefix);
    append_code_section(&mut module, expr);
    module
}

fn module_prefix() -> Vec<u8> {
    let mut module = Vec::with_capacity(320);
    module.extend_from_slice(b"\0asm");
    module.extend_from_slice(&[1, 0, 0, 0]);
    append_section(&mut module, SECTION_TYPE, &type_section());
    append_section(&mut module, SECTION_IMPORT, &import_section());
    append_section(&mut module, SECTION_FUNCTION, &function_section());
    append_section(&mut module, SECTION_EXPORT, &export_section());
    module
}

fn type_section() -> Vec<u8> {
    let mut section = Vec::with_capacity(80);
    encode_u32(&mut section, 10);
    append_func_type(&mut section, &[TYPE_I64, TYPE_I32], &[TYPE_I64]);
    append_func_type(&mut section, &[TYPE_I64, TYPE_I32, TYPE_I64], &[]);
    append_func_type(&mut section, &[TYPE_I32], &[TYPE_I64]);
    append_func_type(
        &mut section,
        &[TYPE_I64, TYPE_I32, TYPE_I64, TYPE_I64],
        &[TYPE_I64],
    );
    append_func_type(&mut section, &[TYPE_I64, TYPE_I32, TYPE_I64], &[TYPE_I64]);
    append_func_type(&mut section, &[TYPE_I64], &[TYPE_I64]);
    append_func_type(&mut section, &[TYPE_I64, TYPE_I32, TYPE_I64, TYPE_I64], &[]);
    append_func_type(&mut section, &[TYPE_I64, TYPE_I32], &[TYPE_I64, TYPE_I64]);
    append_func_type(
        &mut section,
        &[TYPE_I64, TYPE_I32, TYPE_I64, TYPE_I64, TYPE_I64, TYPE_I64],
        &[],
    );
    append_func_type(
        &mut section,
        &[TYPE_I64, TYPE_I32],
        &[TYPE_I64, TYPE_I64, TYPE_I64, TYPE_I64],
    );
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
    let mut section = Vec::with_capacity(256);
    encode_u32(&mut section, 11);
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
    encode_name(&mut section, "env");
    encode_name(&mut section, "jitLoadExclusive");
    section.push(IMPORT_FUNC);
    encode_u32(&mut section, LOAD_HELPER_TYPE_INDEX);
    encode_name(&mut section, "env");
    encode_name(&mut section, "jitStoreExclusive");
    section.push(IMPORT_FUNC);
    encode_u32(&mut section, EXCLUSIVE_STORE_HELPER_TYPE_INDEX);
    encode_name(&mut section, "env");
    encode_name(&mut section, "jitStorePairGuest");
    section.push(IMPORT_FUNC);
    encode_u32(&mut section, PAIR_STORE_HELPER_TYPE_INDEX);
    encode_name(&mut section, "env");
    encode_name(&mut section, "jitLoadPairGuest");
    section.push(IMPORT_FUNC);
    encode_u32(&mut section, PAIR_LOAD_HELPER_TYPE_INDEX);
    encode_name(&mut section, "env");
    encode_name(&mut section, "jitStoreQuadGuest");
    section.push(IMPORT_FUNC);
    encode_u32(&mut section, QUAD_STORE_HELPER_TYPE_INDEX);
    encode_name(&mut section, "env");
    encode_name(&mut section, "jitLoadQuadGuest");
    section.push(IMPORT_FUNC);
    encode_u32(&mut section, QUAD_LOAD_HELPER_TYPE_INDEX);
    section
}

fn function_section() -> Vec<u8> {
    let mut section = Vec::with_capacity(2);
    encode_u32(&mut section, 1);
    encode_u32(&mut section, RUN_TYPE_INDEX);
    section
}

fn export_section() -> Vec<u8> {
    let mut section = Vec::with_capacity(8);
    encode_u32(&mut section, 1);
    encode_name(&mut section, "run");
    section.push(EXPORT_FUNC);
    encode_u32(&mut section, RUN_FUNC_INDEX);
    section
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

fn append_section(module: &mut Vec<u8>, id: u8, section: &[u8]) {
    module.push(id);
    encode_u32(module, section.len() as u32);
    module.extend_from_slice(section);
}
