use super::*;
use crate::constants::CPIO_HEADER_SIZE;

#[test]
fn roundtrip_single_file() {
    let entries = vec![("init".to_string(), b"#!/bin/sh\necho hello".to_vec(), 0o755)];
    let archive = build_cpio(&entries);
    let parsed = parse_cpio(&archive).unwrap();
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].name, "init");
    assert_eq!(parsed[0].data, b"#!/bin/sh\necho hello");
    assert_eq!(parsed[0].mode, 0o755);
}

#[test]
fn roundtrip_multiple_files() {
    let entries = vec![
        ("init".to_string(), b"#!/bin/sh".to_vec(), 0o755),
        ("etc/motd".to_string(), b"Welcome".to_vec(), 0o644),
    ];
    let archive = build_cpio(&entries);
    let parsed = parse_cpio(&archive).unwrap();
    assert_eq!(parsed.len(), 2);
    assert_eq!(parsed[0].name, "init");
    assert_eq!(parsed[1].name, "etc/motd");
}

#[test]
fn borrowed_entry_lookup_finds_data_without_copying() {
    let archive = build_cpio(&[
        ("first".into(), b"one".to_vec(), 0o644),
        ("second".into(), b"two".to_vec(), 0o644),
    ]);

    assert_eq!(
        find_cpio_entry(&archive, "second").unwrap(),
        Some(&b"two"[..])
    );
    assert_eq!(find_cpio_entry(&archive, "missing").unwrap(), None);
}

#[test]
fn borrowed_entry_lookup_uses_the_last_duplicate_like_initramfs_unpacking() {
    let archive = build_cpio(&[
        ("duplicate".into(), b"stale".to_vec(), 0o644),
        ("duplicate".into(), b"active".to_vec(), 0o644),
    ]);

    assert_eq!(
        find_cpio_entry(&archive, "duplicate").unwrap(),
        Some(&b"active"[..])
    );
}

#[test]
fn borrowed_entry_lookup_validates_concatenated_archives_and_uses_the_last() {
    let mut archive = build_cpio(&[("entry".into(), b"stale".to_vec(), 0o644)]);
    archive.extend_from_slice(&build_cpio(&[
        ("entry".into(), b"active".to_vec(), 0o644),
        ("second".into(), b"two".to_vec(), 0o644),
    ]));

    assert_eq!(
        find_cpio_entry(&archive, "entry").unwrap(),
        Some(&b"active"[..])
    );
    assert_eq!(
        find_cpio_entry(&archive, "second").unwrap(),
        Some(&b"two"[..])
    );
}

#[test]
fn borrowed_entry_lookup_rejects_an_unsupported_concatenated_tail() {
    let mut archive = build_cpio(&[("first".into(), b"one".to_vec(), 0o644)]);
    archive.extend_from_slice(b"not another archive");

    assert_eq!(
        find_cpio_entry(&archive, "first"),
        Err("cpio concatenated header truncated")
    );
}

#[test]
fn borrowed_entry_lookup_rejects_a_trailer_with_data_without_panicking() {
    let mut archive = build_cpio(&[]);
    let trailer = b"TRAILER!!!";
    let trailer_name = archive
        .windows(trailer.len())
        .position(|window| window == trailer)
        .unwrap();
    let header = trailer_name - CPIO_HEADER_SIZE;
    archive[header + 54..header + 62].copy_from_slice(b"00000001");
    archive.push(0);

    assert_eq!(
        find_cpio_entry(&archive, "missing"),
        Err("cpio trailer contains data")
    );
}

#[test]
fn cpio_readers_reject_corrupt_unused_header_fields() {
    let mut archive = build_cpio(&[("init".into(), b"data".to_vec(), 0o644)]);
    archive[6] = b'g';

    assert_eq!(find_cpio_entry(&archive, "init"), Err("bad hex digit"));
    assert_eq!(parse_cpio(&archive), Err("bad hex digit"));
}

#[test]
fn load_initrd_into_memory() {
    use crate::platform::virt::SystemBus;
    let mut bus = SystemBus::new();
    let entries = vec![("hello.txt".to_string(), b"world".to_vec(), 0o644)];
    let archive = build_cpio(&entries);
    load_initrd(&mut bus, 0x4200_0000, &archive);
    // Cpio header starts with "070701"
    assert_eq!(bus.mem.read(0x4200_0000, 1), Some(b'0' as u64));
    assert_eq!(bus.mem.read(0x4200_0001, 1), Some(b'7' as u64));
    assert_eq!(bus.mem.read(0x4200_0005, 1), Some(b'1' as u64));
}
