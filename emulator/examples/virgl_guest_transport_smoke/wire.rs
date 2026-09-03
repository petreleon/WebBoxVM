pub(super) const PASS: &str = "VIRGL_TRIANGLE_DEMO_PASS card0 capset=1 triangle=0,255,0,255";
pub(super) const FAIL: &str = "VIRGL_CLEAR_DEMO_FAIL";

pub(super) enum VirglPacket {
    Clear(u32),
    Draw(u32),
}

pub(super) fn demo_script(binary: &[u8]) -> String {
    let mut script = String::from("base64 -d >/tmp/virgl-clear-demo <<'WEBBOXVM_VIRGL_EOF'\r");
    script.push_str(&base64_lines(binary));
    script
        .push_str("WEBBOXVM_VIRGL_EOF\rchmod 0755 /tmp/virgl-clear-demo\r/tmp/virgl-clear-demo\r");
    script
}

pub(super) fn vgc1_sequence(packet: &[u8]) -> Result<u32, String> {
    if packet.len() != 36
        || packet.get(..4) != Some(b"VGC1")
        || read_u32(packet, 4) != Some(1)
        || read_u32(packet, 12) != Some(1024)
        || read_u32(packet, 16) != Some(768)
        || !words_are(
            packet,
            20,
            &[0x3e80_0000, 0x3f00_0000, 0x3f40_0000, 0x3f80_0000],
        )
    {
        return Err("guest emitted an invalid standard VirGL clear packet".into());
    }
    read_u32(packet, 8)
        .filter(|sequence| *sequence != 0)
        .ok_or_else(|| "VGC1 packet has no nonzero sequence".into())
}

pub(super) fn virgl_packet(packet: &[u8]) -> Result<VirglPacket, String> {
    match packet.get(..4) {
        Some(magic) if magic == b"VGC1" => vgc1_sequence(packet).map(VirglPacket::Clear),
        Some(magic) if magic == b"VGD1" => vgd1_sequence(packet).map(VirglPacket::Draw),
        _ => Err("guest emitted an unsupported VirGL browser packet".into()),
    }
}

fn vgd1_sequence(packet: &[u8]) -> Result<u32, String> {
    let vertices = [
        0,
        0x3f40_0000,
        0,
        0x3f80_0000,
        0xbf40_0000,
        0xbf40_0000,
        0,
        0x3f80_0000,
        0x3f40_0000,
        0xbf40_0000,
        0,
        0x3f80_0000,
    ];
    if packet.len() != 104
        || packet.get(..4) != Some(b"VGD1")
        || [4, 12, 16, 20]
            .into_iter()
            .zip([1, 1024, 768, 3])
            .any(|(at, want)| read_u32(packet, at) != Some(want))
        || !words_are(
            packet,
            24,
            &[0x3e80_0000, 0x3f00_0000, 0x3f40_0000, 0x3f80_0000],
        )
        || !words_are(packet, 40, &[0, 0x3f80_0000, 0, 0x3f80_0000])
        || !words_are(packet, 56, &vertices)
    {
        return Err("guest emitted an invalid standard VirGL draw packet".into());
    }
    read_u32(packet, 8)
        .filter(|sequence| *sequence != 0)
        .ok_or_else(|| "VGD1 packet has no nonzero sequence".into())
}

pub(super) fn is_clear_readback(packet: &[u8]) -> bool {
    frame_pixels(packet).is_some_and(|pixels| {
        pixels
            .chunks_exact(4)
            .all(|pixel| pixel == [191, 128, 64, 255])
    })
}

pub(super) fn is_upload_readback(packet: &[u8]) -> bool {
    let Some(pixels) = frame_pixels(packet) else {
        return false;
    };
    let offset = (1024 + 1) * 4;
    pixels[..offset].iter().all(|byte| *byte == 0)
        && pixels[offset..offset + 4] == [10, 20, 30, 255]
        && pixels[offset + 4..offset + 8] == [40, 50, 60, 255]
        && pixels[offset + 8..].iter().all(|byte| *byte == 0)
}

pub(super) fn is_triangle_readback(packet: &[u8]) -> bool {
    frame_pixels(packet).is_some_and(|pixels| {
        let center = (384 * 1024 + 512) * 4;
        pixels[..4] == [191, 128, 64, 255] && pixels[center..center + 4] == [0, 255, 0, 255]
    })
}

pub(super) fn shell_ready(uart: &str) -> bool {
    uart.ends_with("# ") || uart.contains("\n# ") || uart.contains("\r\n# ")
}

pub(super) fn output_line(uart: &str, marker: &str) -> bool {
    uart.lines().any(|line| line.trim() == marker)
}

pub(super) fn output_starts(uart: &str, marker: &str) -> bool {
    uart.lines().any(|line| line.trim().starts_with(marker))
}

pub(super) fn tail(text: &str) -> String {
    let mut chars: Vec<_> = text.chars().rev().take(2_000).collect();
    chars.reverse();
    chars.into_iter().collect()
}

fn frame_pixels(packet: &[u8]) -> Option<&[u8]> {
    (packet.len() == 32 + 1024 * 768 * 4
        && packet.get(..4) == Some(b"WBGF")
        && [4, 8, 12, 16, 20, 24, 28]
            .into_iter()
            .zip([1, 1024, 768, 0, 0, 1024, 768])
            .all(|(offset, expected)| read_u32(packet, offset) == Some(expected)))
    .then_some(&packet[32..])
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

fn words_are(packet: &[u8], offset: usize, expected: &[u32]) -> bool {
    expected
        .iter()
        .enumerate()
        .all(|(index, value)| read_u32(packet, offset + index * 4) == Some(*value))
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
    raw.as_bytes()
        .chunks(76)
        .map(|line| format!("{}\r", std::str::from_utf8(line).unwrap()))
        .collect()
}
