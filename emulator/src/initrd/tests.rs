use super::*;

#[test]
fn roundtrip_single_file() {
    let entries = vec![(
        "init".to_string(),
        b"#!/bin/sh\necho hello".to_vec(),
        0o755,
    )];
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
fn load_initrd_into_memory() {
    use crate::bus::SystemBus;
    let mut bus = SystemBus::new();
    let entries = vec![("hello.txt".to_string(), b"world".to_vec(), 0o644)];
    let archive = build_cpio(&entries);
    load_initrd(&mut bus, 0x4200_0000, &archive);
    // Cpio header starts with "070701"
    assert_eq!(bus.mem.read(0x4200_0000, 1), Some(b'0' as u64));
    assert_eq!(bus.mem.read(0x4200_0001, 1), Some(b'7' as u64));
    assert_eq!(bus.mem.read(0x4200_0005, 1), Some(b'1' as u64));
}
