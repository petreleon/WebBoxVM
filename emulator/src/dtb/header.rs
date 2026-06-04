use super::*;

pub(super) fn assemble_dtb(struct_block: Vec<u8>, strings: Vec<u8>) -> Vec<u8> {
    let header_size: u32 = 40;
    let mem_rsvmap_size: u32 = 16;
    let off_mem_rsvmap = header_size;
    let off_dt_struct = off_mem_rsvmap + mem_rsvmap_size;
    let off_dt_strings = off_dt_struct + struct_block.len() as u32;
    let totalsize = off_dt_strings + strings.len() as u32;

    let mut dtb = Vec::new();
    dtb.extend_from_slice(&FDT_MAGIC.to_be_bytes());
    dtb.extend_from_slice(&totalsize.to_be_bytes());
    dtb.extend_from_slice(&off_dt_struct.to_be_bytes());
    dtb.extend_from_slice(&off_dt_strings.to_be_bytes());
    dtb.extend_from_slice(&off_mem_rsvmap.to_be_bytes());
    dtb.extend_from_slice(&FDT_VERSION.to_be_bytes());
    dtb.extend_from_slice(&FDT_LAST_COMP_VERSION.to_be_bytes());
    dtb.extend_from_slice(&0u32.to_be_bytes());
    dtb.extend_from_slice(&(strings.len() as u32).to_be_bytes());
    dtb.extend_from_slice(&(struct_block.len() as u32).to_be_bytes());
    dtb.extend_from_slice(&0u64.to_be_bytes());
    dtb.extend_from_slice(&0u64.to_be_bytes());
    dtb.extend_from_slice(&struct_block);
    dtb.extend_from_slice(&strings);
    dtb
}
