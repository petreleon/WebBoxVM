use super::super::fast_boot::decode_zstd;
use crate::initrd::find_cpio_entries_and_zstd_tail;

pub(super) const NAMES: [&str; 8] = [
    "virtio_mmio",
    "virtio_blk",
    "crc16",
    "crc32c_generic",
    "libcrc32c",
    "mbcache",
    "jbd2",
    "ext4",
];
const RELATIVE_PATHS: [&str; 8] = [
    "kernel/drivers/virtio/virtio_mmio.ko.xz",
    "kernel/drivers/block/virtio_blk.ko.xz",
    "kernel/lib/crc16.ko.xz",
    "kernel/crypto/crc32c_generic.ko.xz",
    "kernel/lib/libcrc32c.ko.xz",
    "kernel/fs/mbcache.ko.xz",
    "kernel/fs/jbd2/jbd2.ko.xz",
    "kernel/fs/ext4/ext4.ko.xz",
];
const XZ_MAGIC: &[u8] = b"\xfd7zXZ\0";

pub(super) fn extract(initrd: &[u8], suffix: &str) -> Option<[Vec<u8>; 8]> {
    let paths = module_paths(suffix);
    let targets = paths.each_ref().map(String::as_str);
    let (entries, compressed) = find_cpio_entries_and_zstd_tail(initrd, targets).ok()?;
    let mut selected = choose_variants(entries);
    if let Some(compressed) = compressed {
        let decoded = decode_zstd(compressed)?;
        let targets = paths.each_ref().map(String::as_str);
        let (entries, nested) = find_cpio_entries_and_zstd_tail(&decoded, targets).ok()?;
        if nested.is_some() {
            return None;
        }
        merge_variants(&mut selected, entries);
    }
    let [
        Some(a),
        Some(b),
        Some(c),
        Some(d),
        Some(e),
        Some(f),
        Some(g),
        Some(h),
    ] = selected
    else {
        return None;
    };
    let modules = [a, b, c, d, e, f, g, h];
    modules
        .iter()
        .all(|module| module.starts_with(XZ_MAGIC))
        .then_some(modules)
}

fn module_paths(suffix: &str) -> [String; 16] {
    std::array::from_fn(|index| {
        let root = if index < 8 { "usr/lib" } else { "lib" };
        format!("{root}/modules/{suffix}/{}", RELATIVE_PATHS[index % 8])
    })
}

fn choose_variants(entries: [Option<&[u8]>; 16]) -> [Option<Vec<u8>>; 8] {
    std::array::from_fn(|index| entries[index].or(entries[index + 8]).map(<[u8]>::to_vec))
}

fn merge_variants(selected: &mut [Option<Vec<u8>>; 8], entries: [Option<&[u8]>; 16]) {
    for index in 0..8 {
        if let Some(module) = entries[index].or(entries[index + 8]) {
            selected[index] = Some(module.to_vec());
        }
    }
}
