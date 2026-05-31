use crate::constants::*;

use super::{CpioNode, pad_to_4};

/// Build a cpio `newc` archive from a list of (name, data, mode) tuples.
pub fn build_cpio(entries: &[(String, Vec<u8>, u32)]) -> Vec<u8> {
    let nodes: Vec<CpioNode> = entries
        .iter()
        .map(|(name, data, mode)| CpioNode {
            name: name.clone(),
            data: data.clone(),
            mode: *mode,
            nlink: 1,
            devmajor: 0,
            devminor: 0,
            rdevmajor: 0,
            rdevminor: 0,
        })
        .collect();

    build_cpio_nodes(&nodes)
}

/// Build a cpio `newc` archive from richer nodes, including directories,
/// symlinks, and device files.
pub fn build_cpio_nodes(entries: &[CpioNode]) -> Vec<u8> {
    let mut out = Vec::new();
    for (ino, node) in entries.iter().enumerate() {
        push_header(&mut out, node, ino as u32 + 1);
        out.extend_from_slice(node.name.as_bytes());
        out.push(0);
        pad_to_4(&mut out);
        out.extend_from_slice(&node.data);
        pad_to_4(&mut out);
    }

    push_trailer(&mut out);
    out.extend_from_slice(CPIO_TRAILER_NAME.as_bytes());
    out.push(0);
    pad_to_4(&mut out);
    out
}

fn push_header(out: &mut Vec<u8>, node: &CpioNode, ino: u32) {
    let namesize = node.name.len() + 1;
    out.extend_from_slice(CPIO_NEWC_MAGIC.as_bytes());
    out.extend_from_slice(format!("{ino:08x}").as_bytes());
    out.extend_from_slice(format!("{:08x}", node.mode).as_bytes());
    out.extend_from_slice(format!("{:08x}", 0).as_bytes());
    out.extend_from_slice(format!("{:08x}", 0).as_bytes());
    out.extend_from_slice(format!("{:08x}", node.nlink).as_bytes());
    out.extend_from_slice(format!("{:08x}", 0).as_bytes());
    out.extend_from_slice(format!("{:08x}", node.data.len()).as_bytes());
    out.extend_from_slice(format!("{:08x}", node.devmajor).as_bytes());
    out.extend_from_slice(format!("{:08x}", node.devminor).as_bytes());
    out.extend_from_slice(format!("{:08x}", node.rdevmajor).as_bytes());
    out.extend_from_slice(format!("{:08x}", node.rdevminor).as_bytes());
    out.extend_from_slice(format!("{namesize:08x}").as_bytes());
    out.extend_from_slice(format!("{:08x}", 0).as_bytes());
}

fn push_trailer(out: &mut Vec<u8>) {
    let trailer = CpioNode {
        name: CPIO_TRAILER_NAME.to_string(),
        data: Vec::new(),
        mode: 0,
        nlink: 1,
        devmajor: 0,
        devminor: 0,
        rdevmajor: 0,
        rdevminor: 0,
    };
    push_header(out, &trailer, 0);
}
