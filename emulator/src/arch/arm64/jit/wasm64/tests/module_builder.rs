use super::super::{module_builder::build_module, opcodes};

#[test]
fn repeated_modules_keep_same_invariant_sections() {
    let first = build_module(vec![opcodes::OP_END]);
    let second = build_module(vec![opcodes::OP_END]);

    assert_eq!(first, second);
    assert_eq!(
        section_ids(&first),
        vec![
            opcodes::SECTION_TYPE,
            opcodes::SECTION_IMPORT,
            opcodes::SECTION_FUNCTION,
            opcodes::SECTION_EXPORT,
            opcodes::SECTION_CODE,
        ]
    );
}

#[test]
fn direct_code_section_length_tracks_large_expression() {
    let mut expr = Vec::with_capacity(20_001);
    for _ in 0..10_000 {
        expr.push(opcodes::OP_I64_CONST);
        expr.push(0);
    }
    expr.push(opcodes::OP_END);

    let module = build_module(expr.clone());
    let code = section_payload(&module, opcodes::SECTION_CODE).expect("code section");
    let mut offset = 0usize;
    assert_eq!(read_u32(code, &mut offset), 1);
    let body_len = read_u32(code, &mut offset) as usize;
    assert_eq!(body_len, code.len() - offset);
    assert_eq!(read_u32(code, &mut offset), 1);
    assert_eq!(read_u32(code, &mut offset), 7);
    assert_eq!(code[offset], opcodes::TYPE_I64);
    offset += 1;
    assert_eq!(&code[offset..], expr.as_slice());
}

fn section_ids(module: &[u8]) -> Vec<u8> {
    let mut ids = Vec::new();
    let mut offset = 8usize;
    while offset < module.len() {
        ids.push(module[offset]);
        offset += 1;
        let len = read_u32(module, &mut offset) as usize;
        offset += len;
    }
    ids
}

fn section_payload(module: &[u8], wanted: u8) -> Option<&[u8]> {
    let mut offset = 8usize;
    while offset < module.len() {
        let id = module[offset];
        offset += 1;
        let len = read_u32(module, &mut offset) as usize;
        let end = offset + len;
        assert!(end <= module.len());
        if id == wanted {
            return Some(&module[offset..end]);
        }
        offset = end;
    }
    None
}

fn read_u32(bytes: &[u8], offset: &mut usize) -> u32 {
    let mut value = 0u32;
    let mut shift = 0u32;
    loop {
        let byte = bytes[*offset];
        *offset += 1;
        value |= ((byte & 0x7f) as u32) << shift;
        if byte < 0x80 {
            return value;
        }
        shift += 7;
    }
}
