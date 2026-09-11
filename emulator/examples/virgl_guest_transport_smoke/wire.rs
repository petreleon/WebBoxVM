use emulator::boot::BootContext;
pub(super) const PASS: &str = "VIRGL_TEXTURE_DEMO_PASS card0 capset=1 rings=2:ring1-clear mesh=2x-constant-uniform-triangle constant=121,115,134,255 blob=guest+host-map+default-shadow+renderer-local texture=10,20,30,255 linear=25,35,45,255 pair=55,65,75,255 vertex=64,64,127,255 modulate=32,32,64,255 uniform-inline-vertex=147,141,58,255 depth-less=58,102,20,255 solid-batch=0,128,64,255 depth-batch=0,0,128,255 depth-equal=128,0,0,255 depth-equal-batch=128,0,64,255 depth-mixed-batch=0,128,64,255 depth-write-mask-batch=0,128,64,255 depth-vertex-color=64,64,127,255 depth-texture=10,20,30,255 depth-texture-color=32,32,64,255 depth-material-constant-offset=64,64,64,255";
pub(super) const FAIL: &str = "VIRGL_CLEAR_DEMO_FAIL";
pub(super) enum VirglPacket { Clear(u32), Draw(u32), UniformDraw(u32), DepthDraw(u32), DepthEqualDraw(u32), DepthEqualBatch(u32), DepthMixedBatch(u32), DepthWriteMaskBatch(u32), DepthVertexColorDraw(u32), DepthTextureDraw(u32), DepthTextureColorDraw(u32), MaterialBatch(u32), SolidBatch(u32), DepthBatch(u32), TexturedDraw(u32, TextureMode), TexturePairDraw(u32), VertexColorDraw(u32), TextureColorDraw(u32) }
pub(super) fn demo_script(binary: &[u8]) -> String {
    let mut script = String::from("base64 -d >/tmp/virgl-clear-demo <<'WEBBOXVM_VIRGL_EOF'\r");
    script.push_str(&base64_lines(binary));
    script
        .push_str("WEBBOXVM_VIRGL_EOF\rchmod 0755 /tmp/virgl-clear-demo\r/tmp/virgl-clear-demo\r");
    script
}
pub(super) fn virgl_packet(packet: &[u8]) -> Result<VirglPacket, String> {
    match packet.get(..4) {
        Some(magic) if magic == b"VGC1" => vgc1_sequence(packet).map(VirglPacket::Clear),
        Some(magic) if magic == b"VGM1" && read_u32(packet, 4) == Some(2) => match resident_material_sequence(packet)? {
            ResidentMaterial::Texture(sequence, mode) => Ok(VirglPacket::TexturedDraw(sequence, mode)),
            ResidentMaterial::Pair(sequence) => Ok(VirglPacket::TexturePairDraw(sequence)),
            ResidentMaterial::VertexColor(sequence) => Ok(VirglPacket::VertexColorDraw(sequence)),
            ResidentMaterial::TextureColor(sequence) => Ok(VirglPacket::TextureColorDraw(sequence)),
        },
        Some(magic) if magic == b"VGM1" => material_batch_sequence(packet).map(VirglPacket::MaterialBatch),
        Some(magic) if magic == b"VGB1" && read_u32(packet, 4) == Some(1) => batch_sequence(packet).map(VirglPacket::SolidBatch),
        Some(magic) if magic == b"VGB1" && read_u32(packet, 4) == Some(6) => match packet.len() {
            204 => match resident_solid_packet(packet)? {
                ResidentSolid::Draw(sequence) => Ok(VirglPacket::Draw(sequence)),
                ResidentSolid::Uniform(sequence) => Ok(VirglPacket::UniformDraw(sequence)),
            },
            264 => batch_sequence(packet).map(VirglPacket::SolidBatch),
            _ => Err("guest emitted an unsupported initial resident VGB1 size".into()),
        },
        Some(magic) if magic == b"VGB1" && read_u32(packet, 4) == Some(2) => depth_batch_sequence(packet).map(VirglPacket::DepthBatch),
        Some(magic) if magic == b"VGB1" && read_u32(packet, 4) == Some(3) => depth_equal_batch_sequence(packet).map(VirglPacket::DepthEqualBatch),
        Some(magic) if magic == b"VGB1" && read_u32(packet, 4) == Some(4) => depth_mixed_batch_sequence(packet).map(VirglPacket::DepthMixedBatch),
        Some(magic) if magic == b"VGB1" && read_u32(packet, 4) == Some(5) => depth_write_mask_batch_sequence(packet).map(VirglPacket::DepthWriteMaskBatch),
        Some(magic) if magic == b"VGD1" => match read_u32(packet, 4) {
            Some(2) => uniform_sequence(packet).map(VirglPacket::UniformDraw)
                .or_else(|_| vgd1_sequence(packet).map(VirglPacket::Draw)),
            Some(5) => vgt1_sequence(packet)
                .map(|(sequence, mode)| VirglPacket::TexturedDraw(sequence, mode)),
            Some(6) => vtp1_sequence(packet).map(VirglPacket::TexturePairDraw),
            Some(7) => vvc1_sequence(packet).map(VirglPacket::VertexColorDraw),
            Some(8) => vtc1_sequence(packet).map(VirglPacket::TextureColorDraw),
            Some(9) => depth_sequence(packet).map(VirglPacket::DepthDraw),
            Some(10) => depth_equal_sequence(packet).map(VirglPacket::DepthEqualDraw),
            Some(12) => depth_vertex_color_sequence(packet).map(VirglPacket::DepthVertexColorDraw),
            Some(13) => depth_texture_sequence(packet).map(VirglPacket::DepthTextureDraw), Some(14) => depth_texture_color_sequence(packet).map(VirglPacket::DepthTextureColorDraw),
            _ => Err("guest emitted an unsupported VGD1 packet version".into()),
        },
        _ => Err(format!(
            "guest emitted an unsupported VirGL browser packet: magic={:?}, version={:?}, bytes={}",
            packet.get(..4), read_u32(packet, 4), packet.len(),
        )),
    }
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
pub(super) fn complete(
    vm: &mut BootContext,
    sequence: u32,
    expected: impl FnOnce(&[u8]) -> bool,
    label: &str,
) -> Result<(), String> {
    if !vm.machine.bus.complete_gpu_3d(sequence, true) {
        return Err("standard VirGL completion was rejected".into());
    }
    if !expected(&vm.machine.bus.virtio_gpu.take_scanout_update()) {
        return Err(format!(
            "{label} did not produce the expected WBGF readback"
        ));
    }
    println!("{label} completed: sequence {sequence}");
    Ok(())
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
#[path = "wire/batch.rs"] mod batch; #[path = "wire/clear.rs"] mod clear; #[path = "wire/draw.rs"] mod draw;
#[path = "wire/resident_material.rs"] mod resident_material; #[path = "wire/resident_solid.rs"] mod resident_solid;
#[path = "wire/depth.rs"] mod depth; #[path = "wire/depth_texture.rs"] mod depth_texture; #[path = "wire/depth_texture_color.rs"] mod depth_texture_color; #[path = "wire/depth_equal.rs"] mod depth_equal;
#[path = "wire/depth_equal_batch.rs"] mod depth_equal_batch; #[path = "wire/depth_vertex_color.rs"] mod depth_vertex_color;
#[path = "wire/material_batch.rs"] mod material_batch;
#[path = "wire/texture.rs"] mod texture; #[path = "wire/texture_pair.rs"] mod texture_pair; #[path = "wire/vertex_color.rs"] mod vertex_color; #[path = "wire/texture_color.rs"] mod texture_color;

pub(crate) use batch::{is_depth_batch_readback, is_solid_batch_readback};
use batch::{batch_sequence, depth_batch_sequence};
use clear::vgc1_sequence;
pub(crate) use draw::{is_triangle_readback, is_uniform_readback};
use draw::{uniform_sequence, vgd1_sequence};
use resident_material::{sequence as resident_material_sequence, ResidentMaterial};
use resident_solid::{packet as resident_solid_packet, ResidentSolid};
pub(crate) use depth::is_depth_readback;
use depth::depth_sequence;
pub(crate) use depth_texture::is_depth_texture_readback; pub(crate) use depth_texture_color::is_depth_texture_color_readback;
use depth_texture::depth_texture_sequence; use depth_texture_color::depth_texture_color_sequence;
pub(crate) use material_batch::is_material_batch_readback;
use material_batch::material_batch_sequence;
pub(crate) use depth_equal::is_depth_equal_readback;
use depth_equal::depth_equal_sequence;
use depth_vertex_color::depth_vertex_color_sequence;
pub(crate) use depth_equal_batch::{is_depth_equal_batch_readback, is_depth_mixed_batch_readback, is_depth_write_mask_batch_readback};
use depth_equal_batch::{depth_equal_batch_sequence, depth_mixed_batch_sequence, depth_write_mask_batch_sequence};
pub(crate) use texture::{TextureMode, is_textured_triangle_readback};
use texture::vgt1_sequence;
pub(crate) use texture_pair::is_texture_pair_readback;
use texture_pair::vtp1_sequence;
pub(crate) use vertex_color::is_vertex_color_readback;
use vertex_color::vvc1_sequence;
pub(crate) use texture_color::is_texture_color_readback;
use texture_color::vtc1_sequence;
