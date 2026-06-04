pub(super) const SECTION_TYPE: u8 = 1;
pub(super) const SECTION_IMPORT: u8 = 2;
pub(super) const SECTION_FUNCTION: u8 = 3;
pub(super) const SECTION_EXPORT: u8 = 7;
pub(super) const SECTION_CODE: u8 = 10;

pub(super) const TYPE_I64: u8 = 0x7e;
pub(super) const IMPORT_MEMORY: u8 = 0x02;
pub(super) const EXPORT_FUNC: u8 = 0x00;
pub(super) const LIMITS_MEMORY64: u32 = 0x04;

pub(super) const OP_END: u8 = 0x0b;
pub(super) const OP_LOCAL_GET: u8 = 0x20;
pub(super) const OP_I64_LOAD: u8 = 0x29;
pub(super) const OP_I64_STORE: u8 = 0x37;
pub(super) const OP_I64_CONST: u8 = 0x42;
pub(super) const OP_I64_ADD: u8 = 0x7c;
pub(super) const OP_I64_SUB: u8 = 0x7d;
pub(super) const OP_I64_AND: u8 = 0x83;
pub(super) const OP_I64_OR: u8 = 0x84;
pub(super) const OP_I64_XOR: u8 = 0x85;
pub(super) const OP_I32_WRAP_I64: u8 = 0xa7;
pub(super) const OP_I64_EXTEND_I32_S: u8 = 0xac;
