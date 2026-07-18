use super::encoding::{encode_name, encode_u32};
use super::memory_import::append_memory_type;
use super::opcodes::*;
use std::sync::LazyLock;

const LOAD_HELPER_TYPE_INDEX: u32 = 0;
const STORE_HELPER_TYPE_INDEX: u32 = 1;
const SYSREG_HELPER_TYPE_INDEX: u32 = 2;
const EXCLUSIVE_PAIR_HELPER_TYPE_INDEX: u32 = 3;
const EXCLUSIVE_STORE_HELPER_TYPE_INDEX: u32 = 4;
const FULL_RUN_TYPE_INDEX: u32 = 5;
const PAIR_STORE_HELPER_TYPE_INDEX: u32 = 6;
const PAIR_LOAD_HELPER_TYPE_INDEX: u32 = 7;
const QUAD_STORE_HELPER_TYPE_INDEX: u32 = 8;
const QUAD_LOAD_HELPER_TYPE_INDEX: u32 = 9;
const FULL_RUN_FUNC_INDEX: u32 = 11;

static FULL_PREFIX_BYTES: LazyLock<Vec<u8>> = LazyLock::new(full_prefix);
static MINIMAL_PREFIX_BYTES: LazyLock<Vec<u8>> = LazyLock::new(minimal_prefix);

pub(super) fn module_prefix(imports_helpers: bool) -> &'static [u8] {
    if imports_helpers {
        FULL_PREFIX_BYTES.as_slice()
    } else {
        MINIMAL_PREFIX_BYTES.as_slice()
    }
}

fn full_prefix() -> Vec<u8> {
    let mut module = wasm_header(320);
    append_section(&mut module, SECTION_TYPE, &full_type_section());
    append_section(&mut module, SECTION_IMPORT, &full_import_section());
    append_section(
        &mut module,
        SECTION_FUNCTION,
        &function_section(FULL_RUN_TYPE_INDEX),
    );
    append_section(
        &mut module,
        SECTION_EXPORT,
        &export_section(FULL_RUN_FUNC_INDEX),
    );
    module
}

fn minimal_prefix() -> Vec<u8> {
    let mut module = wasm_header(48);
    append_section(&mut module, SECTION_TYPE, &minimal_type_section());
    append_section(&mut module, SECTION_IMPORT, &memory_import_section(1));
    append_section(&mut module, SECTION_FUNCTION, &function_section(0));
    append_section(&mut module, SECTION_EXPORT, &export_section(0));
    module
}

fn wasm_header(capacity: usize) -> Vec<u8> {
    let mut module = Vec::with_capacity(capacity);
    module.extend_from_slice(b"\0asm");
    module.extend_from_slice(&[1, 0, 0, 0]);
    module
}

fn minimal_type_section() -> Vec<u8> {
    let mut section = Vec::with_capacity(8);
    encode_u32(&mut section, 1);
    append_func_type(&mut section, &[TYPE_I64], &[TYPE_I64]);
    section
}

fn full_type_section() -> Vec<u8> {
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

fn full_import_section() -> Vec<u8> {
    let mut section = memory_import_section(12);
    encode_name(&mut section, "env");
    encode_name(&mut section, "jitLoadGuest");
    section.push(IMPORT_FUNC);
    encode_u32(&mut section, LOAD_HELPER_TYPE_INDEX);
    encode_name(&mut section, "env");
    encode_name(&mut section, "jitStoreGuest");
    section.push(IMPORT_FUNC);
    encode_u32(&mut section, STORE_HELPER_TYPE_INDEX);
    append_helper_imports(&mut section);
    section
}

fn append_helper_imports(section: &mut Vec<u8>) {
    append_func_import(section, "jitReadSysReg", SYSREG_HELPER_TYPE_INDEX);
    append_func_import(
        section,
        "jitStoreExclusivePair",
        EXCLUSIVE_PAIR_HELPER_TYPE_INDEX,
    );
    append_func_import(section, "jitLoadExclusive", LOAD_HELPER_TYPE_INDEX);
    append_func_import(
        section,
        "jitStoreExclusive",
        EXCLUSIVE_STORE_HELPER_TYPE_INDEX,
    );
    append_func_import(section, "jitStorePairGuest", PAIR_STORE_HELPER_TYPE_INDEX);
    append_func_import(section, "jitLoadPairGuest", PAIR_LOAD_HELPER_TYPE_INDEX);
    append_func_import(section, "jitStoreQuadGuest", QUAD_STORE_HELPER_TYPE_INDEX);
    append_func_import(section, "jitLoadQuadGuest", QUAD_LOAD_HELPER_TYPE_INDEX);
    append_func_import(section, "jitLoadExclusivePair", PAIR_LOAD_HELPER_TYPE_INDEX);
}

fn append_func_import(section: &mut Vec<u8>, name: &str, type_index: u32) {
    encode_name(section, "env");
    encode_name(section, name);
    section.push(IMPORT_FUNC);
    encode_u32(section, type_index);
}

fn memory_import_section(import_count: u32) -> Vec<u8> {
    let mut section = Vec::with_capacity(256);
    encode_u32(&mut section, import_count);
    encode_name(&mut section, "env");
    encode_name(&mut section, "memory");
    section.push(IMPORT_MEMORY);
    append_memory_type(&mut section);
    section
}

fn function_section(run_type_index: u32) -> Vec<u8> {
    let mut section = Vec::with_capacity(2);
    encode_u32(&mut section, 1);
    encode_u32(&mut section, run_type_index);
    section
}

fn export_section(run_func_index: u32) -> Vec<u8> {
    let mut section = Vec::with_capacity(8);
    encode_u32(&mut section, 1);
    encode_name(&mut section, "run");
    section.push(EXPORT_FUNC);
    encode_u32(&mut section, run_func_index);
    section
}

fn append_section(module: &mut Vec<u8>, id: u8, section: &[u8]) {
    module.push(id);
    encode_u32(module, section.len() as u32);
    module.extend_from_slice(section);
}
