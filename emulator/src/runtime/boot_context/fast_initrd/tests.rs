use super::*;
use crate::initrd::{CpioNode, build_cpio_nodes, parse_cpio};
use ruzstd::encoding::{CompressionLevel, compress_to_vec};

const SUFFIX: &str = "6.12-test-arm64";
const XZ: &[u8] = b"\xfd7zXZ\0fixture";

#[test]
fn fast_initrd_contains_minimal_layout_modules_and_early_smp_script() {
    let archive = build_fast_initrd(supported_spec(module_initrd())).unwrap();
    let entries = parse_cpio(&archive).unwrap();
    let names: Vec<_> = entries.iter().map(|entry| entry.name.as_str()).collect();
    for expected in [
        "init",
        "bin/busybox",
        "bin/sh",
        "bin/mount",
        "bin/insmod",
        "bin/unxz",
        "bin/switch_root",
        "dev/console",
        "lib/modules/crc32c_generic.ko.xz",
        "lib/modules/libcrc32c.ko.xz",
        "lib/modules/ext4.ko.xz",
    ] {
        assert!(names.contains(&expected), "missing {expected}");
    }
    let init = entries.iter().find(|entry| entry.name == "init").unwrap();
    let script = String::from_utf8(init.data.clone()).unwrap();
    assert!(script.contains("rootdev=/dev/vda3"));
    assert!(script.contains("WEBBOXVM_FAST_INITRD_ACTIVE"));
    assert!(script.contains("mount -t ext4 -o rw"));
    assert!(script.contains("WEBBOXVM_CPU1_ONLINE"));
    assert!(script.contains("unxz -c \"$archive\" > \"$output\""));
    assert!(script.contains("switch_root -c /dev/console /newroot /sbin/init"));
    let positions: Vec<_> = modules::NAMES
        .iter()
        .map(|name| script.find(&format!(" {name}")).unwrap())
        .collect();
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn fast_initrd_reads_modules_from_a_zstd_tail() {
    let compressed = compress_to_vec(module_initrd().as_slice(), CompressionLevel::Fastest);
    let mut initrd = build_cpio_nodes(&[]);
    initrd.extend_from_slice(&compressed);

    assert!(build_fast_initrd(supported_spec(initrd)).is_some());
}

#[test]
fn dirty_root_or_missing_kernel_capability_falls_back() {
    let initrd = module_initrd();
    let mut spec = supported_spec(initrd.clone());
    spec.root_clean = false;
    assert!(build_fast_initrd(spec).is_none());

    let mut spec = supported_spec(initrd);
    spec.kernel_supported = false;
    assert!(build_fast_initrd(spec).is_none());
}

#[test]
fn missing_or_mismatched_module_falls_back() {
    let mut nodes = module_nodes();
    nodes.pop();
    assert!(build_fast_initrd(supported_spec(build_cpio_nodes(&nodes))).is_none());

    let mut nodes = module_nodes();
    nodes[0].data = b"not-xz".to_vec();
    assert!(build_fast_initrd(supported_spec(build_cpio_nodes(&nodes))).is_none());
}

fn supported_spec(original: Vec<u8>) -> FastInitrdSpec<'static> {
    FastInitrdSpec {
        original: Box::leak(original.into_boxed_slice()),
        kernel_suffix: Some(SUFFIX),
        root_partition: Some(3),
        kernel_supported: true,
        root_clean: true,
    }
}

fn module_initrd() -> Vec<u8> {
    build_cpio_nodes(&module_nodes())
}

fn module_nodes() -> Vec<CpioNode> {
    let paths = [
        "kernel/drivers/virtio/virtio_mmio.ko.xz",
        "kernel/drivers/block/virtio_blk.ko.xz",
        "kernel/lib/crc16.ko.xz",
        "kernel/crypto/crc32c_generic.ko.xz",
        "kernel/lib/libcrc32c.ko.xz",
        "kernel/fs/mbcache.ko.xz",
        "kernel/fs/jbd2/jbd2.ko.xz",
        "kernel/fs/ext4/ext4.ko.xz",
    ];
    paths
        .into_iter()
        .enumerate()
        .map(|(index, path)| {
            let mut blob = XZ.to_vec();
            blob.push(index as u8);
            CpioNode::file(format!("usr/lib/modules/{SUFFIX}/{path}"), blob, 0o644)
        })
        .collect()
}
