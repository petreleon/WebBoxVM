use crate::initrd::{CpioNode, build_cpio_nodes};

const DEPMOD_WRAPPER: &[u8] = b"#!/bin/sh
# WebBoxVM fast path: Debian ethdetect runs depmod before checking NICs.
for dir in /lib/modules/*; do
    [ -d \"$dir\" ] || continue
    for mod in \\
        kernel/net/core/failover.ko* \\
        kernel/drivers/net/net_failover.ko* \\
        kernel/drivers/net/virtio_net.ko*
    do
        for path in \"$dir\"/$mod; do
            [ -e \"$path\" ] || continue
            insmod \"$path\" >/dev/null 2>&1 || true
        done
    done
done
exit 0
";

const EXT4_PARTMAN_HOOK: &[u8] = b"#!/bin/sh
# Debian partman may reach target mounting before ext4 is loadable.
grep -qw ext4 /proc/filesystems && exit 0
anna-install ext4-modules >/dev/null 2>&1 || true
for dir in /lib/modules/*; do
    [ -d \"$dir\" ] || continue
    for mod in \\
        kernel/lib/crc16.ko* \\
        kernel/crypto/crc32c_generic.ko* \\
        kernel/lib/libcrc32c.ko* \\
        kernel/fs/mbcache.ko* \\
        kernel/fs/jbd2/jbd2.ko* \\
        kernel/fs/ext4/ext4.ko*
    do
        for path in \"$dir\"/$mod; do
            [ -e \"$path\" ] || continue
            insmod \"$path\" >/dev/null 2>&1 || true
        done
    done
done
exit 0
";

pub(super) fn append_installer_network_overlay(initrd: &mut Vec<u8>) {
    initrd.extend_from_slice(&build_installer_network_overlay());
}

pub(super) fn build_installer_network_overlay() -> Vec<u8> {
    build_cpio_nodes(&[
        CpioNode::file("sbin/depmod", DEPMOD_WRAPPER, 0o755),
        CpioNode::file(
            "lib/partman/finish.d/05webboxvm_ext4",
            EXT4_PARTMAN_HOOK,
            0o755,
        ),
    ])
}
