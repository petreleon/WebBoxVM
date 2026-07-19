use super::*;

#[test]
fn picks_highest_matching_kernel_and_initrd_suffix() {
    let names = vec![
        "initrd.img-6.1".to_string(),
        "vmlinuz-6.1".to_string(),
        "vmlinuz-6.12".to_string(),
        "initrd.img-6.12".to_string(),
    ];

    assert_eq!(
        select_versioned_pair(&names, "/boot"),
        Some((
            "/boot/vmlinuz-6.12".to_string(),
            "/boot/initrd.img-6.12".to_string(),
            "6.12".to_string()
        ))
    );
}

#[test]
fn kernel_suffix_must_match_the_exact_selected_initrd() {
    assert_eq!(
        matching_pair_suffix("boot/vmlinuz-6.12.9-arm64", "boot/initrd.img-6.12.9-arm64"),
        Some("6.12.9-arm64")
    );
    assert_eq!(
        matching_pair_suffix("boot/vmlinuz-6.12.9-arm64", "boot/initrd.img-6.12.8-arm64"),
        None
    );
    assert_eq!(matching_pair_suffix("/vmlinuz", "/initrd.img"), None);
}

#[test]
fn bootargs_prefers_root_uuid_over_device_name() {
    let args = bootargs(Some(&(3, "abcd".to_string(), true)), 2);

    assert!(args.contains("root=UUID=abcd"));
    assert!(args.contains("console=ttyAMA0,115200n8"));
}

#[test]
fn bootargs_fallback_uses_single_installed_disk_name() {
    let args = bootargs(None, 2);

    assert!(args.contains("root=/dev/vda3"));
}
