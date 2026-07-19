use super::*;
use crate::initrd::parse_cpio;
use ruzstd::encoding::{CompressionLevel, compress_to_vec};

#[test]
fn overlay_installs_guarded_late_cpu_drop_in() {
    let entries = parse_cpio(&build_staged_smp_overlay()).unwrap();

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].name, "conf/param.conf");
    assert_eq!(entries[0].mode & 0o777, 0o644);
    let script = String::from_utf8(entries[0].data.clone()).unwrap();
    assert!(script.contains("grep -qs \" ${rootmnt} \" /proc/mounts"));
    assert!(script.contains("serial-getty@ttyAMA0.service.d"));
    assert!(script.contains("ExecStartPre=-/bin/sh -ec"));
    assert!(script.contains("WEBBOXVM_CPU1_ONLINE"));
}

#[test]
fn overlay_appends_without_rewriting_the_installed_initrd() {
    let mut initrd = compatible_initrd();
    let original = initrd.clone();
    let overlay_start = (initrd.len() + 3) & !3;

    append_staged_smp_overlay(&mut initrd);

    assert_eq!(&initrd[..original.len()], original);
    assert!(
        initrd[original.len()..overlay_start]
            .iter()
            .all(|byte| *byte == 0)
    );
    let overlay = parse_cpio(&initrd[overlay_start..]).unwrap();
    assert_eq!(overlay[0].name, PARAM_CONF);
}

#[test]
fn staging_requires_exactly_two_cores_and_known_initramfs_hooks() {
    let initrd = compatible_initrd();

    assert!(staged_smp_supported(&initrd, 2));
    assert!(!staged_smp_supported(&initrd, 1));
    assert!(!staged_smp_supported(&initrd, 3));
    assert!(!staged_smp_supported(b"compressed-or-unknown", 2));
}

#[test]
fn staging_never_replaces_an_existing_param_conf() {
    let mut nodes = compatible_nodes();
    nodes.push(CpioNode::file(PARAM_CONF, b"existing", 0o644));

    assert!(!staged_smp_supported(&build_cpio_nodes(&nodes), 2));
}

#[test]
fn staging_uses_the_last_duplicate_hook_files() {
    let mut nodes = compatible_nodes();
    nodes.push(CpioNode::file(
        "init",
        b"#!/bin/sh\nexec switch_root",
        0o755,
    ));

    assert!(!staged_smp_supported(&build_cpio_nodes(&nodes), 2));
}

#[test]
fn staging_requires_init_bottom_before_run_moves_to_the_real_root() {
    let initrd = build_cpio_nodes(&[
        CpioNode::file(
            "init",
            [RUN_MOVE_HOOK, RUN_INIT_BOTTOM_HOOK].concat(),
            0o755,
        ),
        CpioNode::file(INIT_BOTTOM_ORDER, PARAM_SOURCE_HOOK, 0o644),
    ]);

    assert!(!staged_smp_supported(&initrd, 2));
}

#[test]
fn staging_does_not_accept_hooks_in_comments() {
    let initrd = build_cpio_nodes(&[
        CpioNode::file(
            "init",
            [b"# ", RUN_INIT_BOTTOM_HOOK, b"\n# ", RUN_MOVE_HOOK].concat(),
            0o755,
        ),
        CpioNode::file(
            INIT_BOTTOM_ORDER,
            [b"# ", PARAM_SOURCE_HOOK].concat(),
            0o644,
        ),
    ]);

    assert!(!staged_smp_supported(&initrd, 2));
}

#[test]
fn staging_inspects_a_zstd_archive_after_an_uncompressed_prefix() {
    let compressed = compress_to_vec(compatible_initrd().as_slice(), CompressionLevel::Fastest);
    let mut initrd = build_cpio_nodes(&[]);
    initrd.extend_from_slice(&compressed);

    assert!(staged_smp_supported(&initrd, 2));
}

#[test]
fn staging_rejects_a_malformed_zstd_archive() {
    let mut initrd = build_cpio_nodes(&[]);
    initrd.extend_from_slice(&[0x28, 0xb5, 0x2f, 0xfd, 0xff]);

    assert!(!staged_smp_supported(&initrd, 2));
}

fn compatible_initrd() -> Vec<u8> {
    build_cpio_nodes(&compatible_nodes())
}

fn compatible_nodes() -> Vec<CpioNode> {
    vec![
        CpioNode::file(
            "init",
            [RUN_INIT_BOTTOM_HOOK, b"\n", RUN_MOVE_HOOK].concat(),
            0o755,
        ),
        CpioNode::file(INIT_BOTTOM_ORDER, PARAM_SOURCE_HOOK, 0o644),
    ]
}
