use crate::initrd::{CpioNode, build_cpio_nodes, find_cpio_entries_and_zstd_tail, pad_to_4};
use ruzstd::decoding::StreamingDecoder;
use ruzstd::io::Read;

pub(super) const STAGED_SMP_BOOTARGS: &str = "maxcpus=1";
const MAX_DECOMPRESSED_INITRD_BYTES: usize = 128 * 1024 * 1024;
const INIT_BOTTOM_ORDER: &str = "scripts/init-bottom/ORDER";
const PARAM_CONF: &str = "conf/param.conf";
const PARAM_SOURCE_HOOK: &[u8] = b"[ -e /conf/param.conf ] && . /conf/param.conf";
const RUN_INIT_BOTTOM_HOOK: &[u8] = b"run_scripts /scripts/init-bottom";
const RUN_MOVE_HOOK: &[u8] = b"mount -n -o move /run ${rootmnt}/run";

const LATE_CPU_PARAM: &[u8] = br#"#!/bin/sh
# WebBoxVM staged SMP: keep secondary CPUs offline until serial login.
# initramfs-tools sources this file after every cached ORDER entry.
if [ -n "${rootmnt:-}" ] &&
   grep -qs " ${rootmnt} " /proc/mounts &&
   [ ! -e /run/webboxvm-cpu1-dropin-installed ]; then
    webboxvm_dropin=/run/systemd/system/serial-getty@ttyAMA0.service.d
    if mkdir -p "$webboxvm_dropin"; then
        cat > "$webboxvm_dropin/50-webboxvm-cpu1.conf" <<'EOF'
[Service]
ExecStartPre=-/bin/sh -ec 'grep -qx 1 /sys/devices/system/cpu/cpu1/online || echo 1 > /sys/devices/system/cpu/cpu1/online; grep -qx 1 /sys/devices/system/cpu/cpu1/online; echo WEBBOXVM_CPU1_ONLINE > /dev/ttyAMA0'
EOF
        : > /run/webboxvm-cpu1-dropin-installed
    fi
fi
:
"#;

pub(super) fn append_staged_smp_overlay(initrd: &mut Vec<u8>) {
    pad_to_4(initrd);
    initrd.extend_from_slice(&build_staged_smp_overlay());
}

pub(super) fn staged_smp_supported(initrd: &[u8], num_cores: usize) -> bool {
    if num_cores != 2 {
        return false;
    }
    let Some([param_conf, order, init]) = staged_smp_entries(initrd) else {
        return false;
    };
    param_conf.is_none()
        && order
            .as_deref()
            .is_some_and(|data| has_exact_line(data, PARAM_SOURCE_HOOK))
        && init.as_deref().is_some_and(|data| {
            exact_line_position(data, RUN_INIT_BOTTOM_HOOK)
                .zip(exact_line_position(data, RUN_MOVE_HOOK))
                .is_some_and(|(first, second)| first < second)
        })
}

fn staged_smp_entries(initrd: &[u8]) -> Option<[Option<Vec<u8>>; 3]> {
    let targets = [PARAM_CONF, INIT_BOTTOM_ORDER, "init"];
    let (entries, compressed) = find_cpio_entries_and_zstd_tail(initrd, targets).ok()?;
    let mut selected = entries.map(|entry| entry.map(<[u8]>::to_vec));
    if let Some(compressed) = compressed {
        let decoded = decode_zstd(compressed)?;
        let (entries, nested) = find_cpio_entries_and_zstd_tail(&decoded, targets).ok()?;
        if nested.is_some() {
            return None;
        }
        for (slot, entry) in selected.iter_mut().zip(entries) {
            if let Some(entry) = entry {
                *slot = Some(entry.to_vec());
            }
        }
    }
    Some(selected)
}

fn decode_zstd(data: &[u8]) -> Option<Vec<u8>> {
    let mut source = data;
    let mut decoder = StreamingDecoder::new(&mut source).ok()?;
    let mut decoded = Vec::new();
    let mut chunk = [0u8; 64 * 1024];
    loop {
        let count = decoder.read(&mut chunk).ok()?;
        if count == 0 {
            break;
        }
        if decoded.len().checked_add(count)? > MAX_DECOMPRESSED_INITRD_BYTES {
            return None;
        }
        decoded.extend_from_slice(&chunk[..count]);
    }
    drop(decoder);
    source.is_empty().then_some(decoded)
}

fn build_staged_smp_overlay() -> Vec<u8> {
    build_cpio_nodes(&[CpioNode::file(PARAM_CONF, LATE_CPU_PARAM, 0o644)])
}

fn has_exact_line(data: &[u8], expected: &[u8]) -> bool {
    exact_line_position(data, expected).is_some()
}

fn exact_line_position(data: &[u8], expected: &[u8]) -> Option<usize> {
    data.split(|byte| *byte == b'\n')
        .position(|line| trim_shell_line(line) == expected)
}

fn trim_shell_line(mut line: &[u8]) -> &[u8] {
    while line.first().is_some_and(u8::is_ascii_whitespace) {
        line = &line[1..];
    }
    while line.last().is_some_and(u8::is_ascii_whitespace) {
        line = &line[..line.len() - 1];
    }
    line
}

#[cfg(test)]
mod tests;
