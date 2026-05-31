use crate::initrd::{build_cpio_nodes, CpioNode};

pub const DEFAULT_BOOTARGS: &str =
    "earlycon=pl011,0x09000000 console=ttyAMA0,115200n8 rdinit=/init loglevel=7";

pub const DEFAULT_BUSYBOX_AARCH64: &[u8] = include_bytes!("../../../.artifacts/busybox-aarch64");

const INIT_SCRIPT: &[u8] = br#"#!/bin/sh
export PATH=/bin
export HOME=/root
export TERM=linux
export PS1='webboxvm# '

/bin/mount -t devtmpfs devtmpfs /dev 2>/dev/null || true
/bin/mount -t proc proc /proc 2>/dev/null || true
/bin/mount -t sysfs sysfs /sys 2>/dev/null || true

echo
echo '=== WebBoxVM BusyBox ==='
echo 'Serial console ready.'

if [ -x /bin/cttyhack ] && [ -x /bin/setsid ]; then
    exec /bin/setsid /bin/cttyhack /bin/sh -i
fi

exec /bin/sh -i </dev/console >/dev/console 2>&1
"#;

const PROFILE: &[u8] = br#"export PATH=/bin
export HOME=/root
export TERM=linux
export PS1='webboxvm# '
"#;

const BUSYBOX_APPLETS: &[&str] = &[
    "ash", "cat", "clear", "cttyhack", "dmesg", "echo", "ls", "mkdir", "mount", "ps", "setsid",
    "sh", "stty", "uname",
];

pub fn build_default_initrd() -> Vec<u8> {
    build_busybox_initrd(DEFAULT_BUSYBOX_AARCH64)
        .expect("embedded BusyBox asset must be a 64-bit little-endian AArch64 ELF")
}

pub fn build_busybox_initrd(busybox: &[u8]) -> Result<Vec<u8>, String> {
    validate_busybox_aarch64(busybox)?;

    let mut nodes = vec![
        CpioNode::dir("bin", 0o755),
        CpioNode::dir("dev", 0o755),
        CpioNode::dir("etc", 0o755),
        CpioNode::dir("proc", 0o555),
        CpioNode::dir("root", 0o700),
        CpioNode::dir("sys", 0o555),
        CpioNode::dir("tmp", 0o1777),
        CpioNode::file("init", INIT_SCRIPT, 0o755),
        CpioNode::file("etc/profile", PROFILE, 0o644),
        CpioNode::file("bin/busybox", busybox.to_vec(), 0o755),
        CpioNode::char_device("dev/console", 0o600, 5, 1),
        CpioNode::char_device("dev/tty", 0o666, 5, 0),
        CpioNode::char_device("dev/null", 0o666, 1, 3),
        CpioNode::char_device("dev/zero", 0o666, 1, 5),
    ];

    for applet in BUSYBOX_APPLETS {
        if *applet != "busybox" {
            nodes.push(CpioNode::symlink(
                format!("bin/{applet}"),
                b"busybox".to_vec(),
            ));
        }
    }

    Ok(build_cpio_nodes(&nodes))
}

fn validate_busybox_aarch64(binary: &[u8]) -> Result<(), String> {
    if binary.len() < 64 {
        return Err("BusyBox binary is too small to be an ELF executable".to_string());
    }
    if &binary[0..4] != b"\x7fELF" {
        return Err("BusyBox binary is not an ELF executable".to_string());
    }
    if binary[4] != 2 {
        return Err("BusyBox binary must be ELF64".to_string());
    }
    if binary[5] != 1 {
        return Err("BusyBox binary must be little-endian".to_string());
    }

    let machine = u16::from_le_bytes([binary[18], binary[19]]);
    if machine != 183 {
        return Err(format!(
            "BusyBox binary must target AArch64, found e_machine={machine}"
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::initrd::parse_cpio;

    #[test]
    fn default_initrd_contains_busybox_shell_layout() {
        let archive = build_default_initrd();
        let entries = parse_cpio(&archive).unwrap();
        let names: Vec<_> = entries.iter().map(|entry| entry.name.as_str()).collect();

        assert!(names.contains(&"init"));
        assert!(names.contains(&"bin/busybox"));
        assert!(names.contains(&"bin/sh"));
        assert!(names.contains(&"dev/console"));
        assert!(entries
            .iter()
            .any(|entry| entry.name == "bin/busybox" && entry.data.starts_with(b"\x7fELF")));
    }
}
