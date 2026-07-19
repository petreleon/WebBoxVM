use super::systemd::KNOWN_SERIAL_GETTY_EXEC_STARTS;
use super::*;

#[test]
fn serial_getty_template_accepts_the_known_debian_shape() {
    assert!(usable_serial_getty_template(KNOWN_DEBIAN_SERIAL_GETTY));
}

#[test]
fn override_gate_covers_all_persistent_precedence_roots() {
    for root in [
        "/etc/systemd/system.control",
        "/etc/systemd/system.attached",
        "/etc/systemd/system",
        "/usr/local/lib/systemd/system",
    ] {
        assert!(PERSISTENT_SYSTEMD_OVERRIDE_ROOTS.contains(&root));
        assert!(SYSTEMD_UNIT_ROOTS.contains(&root));
    }
    assert!(SERIAL_GETTY_UNIT_NAMES.contains(&"serial-getty@ttyAMA0.service"));
    assert!(SERIAL_GETTY_UNIT_NAMES.contains(&"serial-getty@.service"));
    for root in ["/usr/lib/systemd/system", "/lib/systemd/system"] {
        assert!(SYSTEMD_UNIT_ROOTS.contains(&root));
    }
    assert!(SERIAL_GETTY_SHARED_DROPIN_NAMES.contains(&"service.d"));
    assert!(SERIAL_GETTY_SHARED_DROPIN_NAMES.contains(&"serial-.service.d"));
}

#[test]
fn serial_getty_template_rejects_directives_that_can_block_or_change_the_hook() {
    let unit_condition = KNOWN_DEBIAN_SERIAL_GETTY
        .replace_bytes(b"[Service]", b"ConditionPathExists=/never\n[Service]");
    assert!(!usable_serial_getty_template(&unit_condition));

    for directive in [
        b"ExecCondition=/bin/false".as_slice(),
        b"ExecStartPre=/bin/false".as_slice(),
        b"User=nobody".as_slice(),
        b"ProtectKernelTunables=yes".as_slice(),
    ] {
        let mut template = KNOWN_DEBIAN_SERIAL_GETTY.to_vec();
        let service = template
            .windows(b"[Service]\n".len())
            .position(|window| window == b"[Service]\n")
            .unwrap()
            + b"[Service]\n".len();
        template.splice(service..service, [directive, b"\n"].concat());
        assert!(
            !usable_serial_getty_template(&template),
            "accepted {}",
            String::from_utf8_lossy(directive)
        );
    }
}

#[test]
fn serial_getty_template_rejects_unknown_commands_and_continuations() {
    let changed = KNOWN_DEBIAN_SERIAL_GETTY
        .replace_bytes(KNOWN_SERIAL_GETTY_EXEC_STARTS[0], b"-/sbin/agetty ttyAMA0");
    assert!(!usable_serial_getty_template(&changed));

    let continued = KNOWN_DEBIAN_SERIAL_GETTY.replace_bytes(b"Type=idle", b"Type=idle \\");
    assert!(!usable_serial_getty_template(&continued));
}

#[test]
fn serial_getty_template_requires_the_tty_and_activation_contract() {
    for directive in [
        b"BindsTo=dev-%i.device\n".as_slice(),
        b"StandardInput=tty\n".as_slice(),
        b"StandardOutput=tty\n".as_slice(),
        b"TTYPath=/dev/%I\n".as_slice(),
        b"WantedBy=getty.target\n".as_slice(),
    ] {
        let changed = KNOWN_DEBIAN_SERIAL_GETTY.replace_bytes(directive, b"");
        assert!(
            !usable_serial_getty_template(&changed),
            "accepted a template without {}",
            String::from_utf8_lossy(directive)
        );
    }
}

#[test]
fn serial_getty_template_required_counts_cannot_wrap() {
    let mut template = KNOWN_DEBIAN_SERIAL_GETTY.to_vec();
    let install = template
        .windows(b"[Install]\n".len())
        .position(|window| window == b"[Install]\n")
        .unwrap();
    let directive = [
        b"ExecStart=".as_slice(),
        KNOWN_SERIAL_GETTY_EXEC_STARTS[0],
        b"\n",
    ]
    .concat();
    template.splice(install..install, directive.repeat(256));

    assert!(!usable_serial_getty_template(&template));
}

#[test]
fn hotplug_config_line_is_exact() {
    assert!(has_line(
        b"CONFIG_SMP=y\nCONFIG_HOTPLUG_CPU=y\n",
        b"CONFIG_HOTPLUG_CPU=y"
    ));
    assert!(!has_line(
        b"# CONFIG_HOTPLUG_CPU is not set\n",
        b"CONFIG_HOTPLUG_CPU=y"
    ));
}

#[test]
fn fast_initrd_config_requires_every_kernel_capability() {
    let required = [
        "CONFIG_BLK_DEV_INITRD=y",
        "CONFIG_RD_ZSTD=y",
        "CONFIG_DEVTMPFS=y",
        "CONFIG_MODULES=y",
        "CONFIG_SMP=y",
        "CONFIG_HOTPLUG_CPU=y",
        "CONFIG_VIRTIO=y",
    ];
    let config = required.join("\n");
    assert!(fast_initrd_config(config.as_bytes()));
    for missing in required {
        assert!(!fast_initrd_config(
            config
                .replace(missing, "# capability unavailable")
                .as_bytes()
        ));
    }
}

const KNOWN_DEBIAN_SERIAL_GETTY: &[u8] = br#"[Unit]
Description=Serial Getty on %I
Documentation=man:agetty(8) man:systemd-getty-generator(8)
Documentation=https://0pointer.de/blog/projects/serial-console.html
BindsTo=dev-%i.device
After=dev-%i.device systemd-user-sessions.service plymouth-quit-wait.service getty-pre.target
After=rc-local.service
Before=getty.target
IgnoreOnIsolate=yes
Conflicts=rescue.service
Before=rescue.service
[Service]
ExecStart=-/sbin/agetty -o '-- \\u' --noreset --noclear --keep-baud 115200,57600,38400,9600 - ${TERM}
Type=idle
Restart=always
UtmpIdentifier=%I
StandardInput=tty
StandardOutput=tty
TTYPath=/dev/%I
TTYReset=yes
TTYVHangup=yes
IgnoreSIGPIPE=no
SendSIGHUP=yes
ImportCredential=tty.serial.%I.agetty.*:agetty.
ImportCredential=tty.serial.%I.login.*:login.
ImportCredential=agetty.*
ImportCredential=login.*
ImportCredential=shell.*
[Install]
WantedBy=getty.target
"#;

trait ReplaceBytes {
    fn replace_bytes(&self, from: &[u8], to: &[u8]) -> Vec<u8>;
}

impl ReplaceBytes for [u8] {
    fn replace_bytes(&self, from: &[u8], to: &[u8]) -> Vec<u8> {
        let offset = self
            .windows(from.len())
            .position(|window| window == from)
            .unwrap();
        [&self[..offset], to, &self[offset + from.len()..]].concat()
    }
}
