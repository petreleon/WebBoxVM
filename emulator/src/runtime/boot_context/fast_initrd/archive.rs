use super::modules::NAMES;
use crate::initrd::CpioNode;

const INIT_SCRIPT: &str = r#"#!/bin/sh
export PATH=/bin

fail() {
    echo "WEBBOXVM_FAST_INITRD_FAILED: $1" > /dev/console
    exec /bin/sh </dev/console >/dev/console 2>&1
}

/bin/mount -t devtmpfs devtmpfs /dev || fail devtmpfs
echo WEBBOXVM_FAST_INITRD_ACTIVE > /dev/ttyAMA0 || fail fast-marker
/bin/mount -t proc proc /proc || fail proc
/bin/mount -t sysfs sysfs /sys || fail sysfs
/bin/mount -t tmpfs -o mode=0755 tmpfs /run || fail run
/bin/mkdir -p /run/modules || fail module-layout

for module in virtio_mmio virtio_blk crc16 crc32c_generic libcrc32c mbcache jbd2 ext4; do
    archive="/lib/modules/${module}.ko.xz"
    output="/run/modules/${module}.ko"
    /bin/unxz -c "$archive" > "$output" || fail "unxz-$module"
    /bin/insmod "$output" || fail "insmod-$module"
done

rootdev=@ROOT_DEVICE@
tries=0
while [ ! -b "$rootdev" ] && [ "$tries" -lt 100 ]; do
    /bin/sleep 0.05
    tries=$((tries + 1))
done
[ -b "$rootdev" ] || fail root-device
/bin/mount -t ext4 -o rw "$rootdev" /newroot || fail root-mount

cpu1=/sys/devices/system/cpu/cpu1/online
[ -e "$cpu1" ] || fail cpu1-missing
echo 1 > "$cpu1" || fail cpu1-hotplug
read cpu1_online < "$cpu1" || fail cpu1-read
[ "$cpu1_online" = 1 ] || fail cpu1-offline
echo WEBBOXVM_CPU1_ONLINE > /dev/ttyAMA0

/bin/mkdir -p /newroot/dev /newroot/proc /newroot/sys /newroot/run || fail root-layout
/bin/mount -o move /proc /newroot/proc || fail move-proc
/bin/mount -o move /sys /newroot/sys || fail move-sys
/bin/mount -o move /run /newroot/run || fail move-run
/bin/mount -o move /dev /newroot/dev || fail move-dev
exec /bin/switch_root -c /dev/console /newroot /sbin/init
"#;

const APPLETS: [&str; 7] = [
    "sh",
    "mount",
    "insmod",
    "unxz",
    "switch_root",
    "mkdir",
    "sleep",
];

pub(super) fn nodes(busybox: &[u8], root_partition: u32, modules: [Vec<u8>; 8]) -> Vec<CpioNode> {
    let script = INIT_SCRIPT
        .replace("@ROOT_DEVICE@", &format!("/dev/vda{root_partition}"))
        .into_bytes();
    let mut nodes = vec![
        CpioNode::dir("bin", 0o755),
        CpioNode::dir("dev", 0o755),
        CpioNode::dir("lib", 0o755),
        CpioNode::dir("lib/modules", 0o755),
        CpioNode::dir("newroot", 0o755),
        CpioNode::dir("proc", 0o555),
        CpioNode::dir("run", 0o755),
        CpioNode::dir("sys", 0o555),
        CpioNode::file("init", script, 0o755),
        CpioNode::file("bin/busybox", busybox.to_vec(), 0o755),
        CpioNode::char_device("dev/console", 0o600, 5, 1),
        CpioNode::char_device("dev/tty", 0o666, 5, 0),
        CpioNode::char_device("dev/null", 0o666, 1, 3),
    ];
    for applet in APPLETS {
        nodes.push(CpioNode::symlink(format!("bin/{applet}"), b"busybox"));
    }
    for (name, module) in NAMES.into_iter().zip(modules) {
        nodes.push(CpioNode::file(
            format!("lib/modules/{name}.ko.xz"),
            module,
            0o644,
        ));
    }
    nodes
}
