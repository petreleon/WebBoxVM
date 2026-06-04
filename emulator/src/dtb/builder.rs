use super::*;

pub(super) struct DtbBuilder {
    strings: Vec<u8>,
    struct_block: Vec<u8>,
}

impl DtbBuilder {
    pub(super) fn new() -> Self {
        Self {
            strings: Vec::new(),
            struct_block: Vec::new(),
        }
    }

    pub(super) fn begin_node(&mut self, name: &str) {
        self.push_token(FDT_BEGIN_NODE);
        self.struct_block.extend_from_slice(name.as_bytes());
        self.struct_block.push(0);
        pad_to_4(&mut self.struct_block);
    }

    pub(super) fn end_node(&mut self) {
        self.push_token(FDT_END_NODE);
    }

    pub(super) fn end_tree(&mut self) {
        self.push_token(FDT_END);
    }

    pub(super) fn prop(&mut self, name: &str, value: &[u8]) {
        let nameoff = self.strings.len() as u32;
        self.strings.extend_from_slice(name.as_bytes());
        self.strings.push(0);
        self.push_token(FDT_PROP);
        self.struct_block
            .extend_from_slice(&(value.len() as u32).to_be_bytes());
        self.struct_block.extend_from_slice(&nameoff.to_be_bytes());
        self.struct_block.extend_from_slice(value);
        pad_to_4(&mut self.struct_block);
    }

    pub(super) fn prop_u32(&mut self, name: &str, value: u32) {
        self.prop(name, &value.to_be_bytes());
    }

    pub(super) fn finish(mut self) -> Vec<u8> {
        pad_to_4(&mut self.strings);
        pad_to_4(&mut self.struct_block);
        header::assemble_dtb(self.struct_block, self.strings)
    }

    fn push_token(&mut self, token: u32) {
        self.struct_block.extend_from_slice(&token.to_be_bytes());
    }
}

pub(super) fn append_two_cell_prop(bytes: &mut Vec<u8>, addr: u64, size: u64) {
    bytes.extend_from_slice(&0u32.to_be_bytes());
    bytes.extend_from_slice(&(addr as u32).to_be_bytes());
    bytes.extend_from_slice(&0u32.to_be_bytes());
    bytes.extend_from_slice(&(size as u32).to_be_bytes());
}

pub(super) fn be_u32_array(values: &[u32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(values.len() * 4);
    for value in values {
        bytes.extend_from_slice(&value.to_be_bytes());
    }
    bytes
}

pub(super) fn c_string(value: &str) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(value.len() + 1);
    bytes.extend_from_slice(value.as_bytes());
    bytes.push(0);
    bytes
}

pub(super) fn pad_to_4(bytes: &mut Vec<u8>) {
    while bytes.len() % 4 != 0 {
        bytes.push(0);
    }
}
