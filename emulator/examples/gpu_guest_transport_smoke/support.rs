pub(super) const SHELL_READY: &str = "GPU_SMOKE_SHELL_READY";
pub(super) const MODPROBE_OK: &str = "GPU_SMOKE_MODPROBE_OK";
pub(super) const MODPROBE_FAIL: &str = "GPU_SMOKE_MODPROBE_FAIL:";
pub(super) const DRM_NODES_END: &str = "GPU_SMOKE_DRM_NODES_END";
pub(super) const SHELL_PROBE: &str = "printf 'GPU_SMOKE_%s\\n' SHELL_READY\r";
pub(super) const MODPROBE_COMMAND: &str = concat!(
    "PATH=/usr/sbin:/usr/bin:/sbin:/bin; export PATH; ",
    "mkdir -p /dev /proc /sys /tmp; ",
    "test -e /dev/ttyAMA0 || mount -t devtmpfs devtmpfs /dev; ",
    "test -e /proc/modules || mount -t proc proc /proc; ",
    "test -d /sys/module || mount -t sysfs sysfs /sys; ",
    "modprobe virtio_gpu; r=$?; ",
    "if test \"$r\" -eq 0 && { test -c /dev/dri/renderD128 || test -c /dev/dri/card0; }; ",
    "then printf 'GPU_SMOKE_MODPROBE_%s\\n' OK; ",
    "else printf 'GPU_SMOKE_MODPROBE_%s:%s\\n' FAIL \"$r\"; fi\r"
);
pub(super) const DRM_NODES_COMMAND: &str = concat!(
    "printf 'GPU_SMOKE_DRM_NODES_%s\\n' BEGIN; ",
    "ls -l /sys/class/drm /dev/dri 2>&1; ",
    "for n in /sys/class/drm/*; do test -e \"$n\" && printf 'DRM_NODE %s\\n' \"$n\"; done; ",
    "printf 'GPU_SMOKE_DRM_NODES_%s\\n' END\r"
);
pub(super) const DEMO_PASSES: [&str; 2] = [
    "WEBGPU_DEMO_PASS renderD128 capset=7 cube=8/36",
    "WEBGPU_DEMO_PASS card0 capset=7 cube=8/36",
];
pub(super) const DEMO_FAILURES: [&str; 3] = [
    "WEBGPU_DEMO_FAIL open-drm",
    "WEBGPU_DEMO_FAIL context-init",
    "WEBGPU_DEMO_FAIL execbuffer",
];

const PACKET_BYTES: usize = 408;
const INDEX_OFFSET: usize = 336;

pub(super) fn shell_prompt_ready(uart: &str) -> bool {
    uart.ends_with("# ") || uart.contains("\n# ") || uart.contains("\r\n# ")
}

pub(super) fn embedded_packet(elf: &[u8]) -> Result<Vec<u8>, String> {
    if elf.get(..4) != Some(b"\x7fELF")
        || elf.get(4) != Some(&2)
        || elf.get(5) != Some(&1)
        || read_u16(elf, 18) != Some(183)
    {
        return Err("demo is not a little-endian ELF64 AArch64 artifact".into());
    }
    let mut matches = elf.windows(4).enumerate().filter_map(|(offset, magic)| {
        let packet = elf.get(offset..offset + PACKET_BYTES)?;
        (magic == b"WBG3" && packet_shape_valid(packet)).then(|| packet.to_vec())
    });
    let packet = matches.next().ok_or("demo contains no valid WBG3 cube")?;
    if matches.next().is_some() {
        return Err("demo contains multiple valid WBG3 cubes".into());
    }
    Ok(packet)
}

pub(super) fn validate_transported_packet(packet: &[u8], expected: &[u8]) -> Result<u32, String> {
    if !packet_shape_valid(packet) {
        return Err("device returned a malformed WBG3 packet".into());
    }
    let sequence = read_u32(packet, 12).ok_or("missing WBG3 sequence")?;
    if sequence == 0 {
        return Err("device returned WBG3 sequence zero".into());
    }
    if expected.len() != PACKET_BYTES
        || packet[..12] != expected[..12]
        || packet[16..] != expected[16..]
    {
        return Err("device packet differs from the injected demo cube".into());
    }
    Ok(sequence)
}

pub(super) fn demo_script(binary: &[u8]) -> String {
    let mut script = String::from("base64 -d >/tmp/webgpu-demo <<'WEBBOXVM_GPU_EOF'\r");
    script.push_str(&base64_lines(binary));
    script.push_str("WEBBOXVM_GPU_EOF\rchmod 0755 /tmp/webgpu-demo\r/tmp/webgpu-demo\r");
    script
}

fn packet_shape_valid(packet: &[u8]) -> bool {
    if packet.len() != PACKET_BYTES
        || packet.get(..4) != Some(b"WBG3")
        || read_u32(packet, 4) != Some(1)
        || read_u32(packet, 8) != Some(1)
        || read_u32(packet, 16) != Some(1024)
        || read_u32(packet, 20) != Some(768)
        || read_u32(packet, 24) != Some(8)
        || read_u32(packet, 28) != Some(36)
    {
        return false;
    }
    let finite = packet[32..INDEX_OFFSET]
        .chunks_exact(4)
        .all(|word| f32::from_le_bytes(word.try_into().unwrap()).is_finite());
    finite
        && packet[INDEX_OFFSET..]
            .chunks_exact(2)
            .all(|word| u16::from_le_bytes([word[0], word[1]]) < 8)
}

fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes(
        bytes.get(offset..offset + 2)?.try_into().ok()?,
    ))
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

fn base64_lines(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut raw = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let bits = u32::from(chunk[0]) << 16
            | u32::from(*chunk.get(1).unwrap_or(&0)) << 8
            | u32::from(*chunk.get(2).unwrap_or(&0));
        raw.push(TABLE[((bits >> 18) & 63) as usize] as char);
        raw.push(TABLE[((bits >> 12) & 63) as usize] as char);
        raw.push(if chunk.len() > 1 {
            TABLE[((bits >> 6) & 63) as usize] as char
        } else {
            '='
        });
        raw.push(if chunk.len() > 2 {
            TABLE[(bits & 63) as usize] as char
        } else {
            '='
        });
    }
    let mut wrapped = String::with_capacity(raw.len() + raw.len() / 76 + 2);
    for line in raw.as_bytes().chunks(76) {
        wrapped.push_str(std::str::from_utf8(line).unwrap());
        wrapped.push('\r');
    }
    wrapped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_is_wrapped_for_the_guest_tty() {
        assert_eq!(base64_lines(b"abcde"), "YWJjZGU=\r");
    }

    #[test]
    fn shell_commands_do_not_echo_complete_success_markers() {
        assert!(!SHELL_PROBE.contains(SHELL_READY));
        assert!(!MODPROBE_COMMAND.contains(MODPROBE_OK));
        assert!(!DRM_NODES_COMMAND.contains(DRM_NODES_END));
    }
}
