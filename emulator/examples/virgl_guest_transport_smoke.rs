//! Native Linux-driver proof for standard capset-1 VirGL transfer, draw, UBOs, and batches.
#[path = "virgl_guest_transport_smoke/wire.rs"] mod wire;
use emulator::boot::BootContext; use std::{env, error::Error, fs, time::{Duration, Instant}}; use wire::*;
const MODULE_OK: &str = "VIRGL_SMOKE_MODULE_OK"; const MODULE_FAIL: &str = "VIRGL_SMOKE_MODULE_FAIL";
const MODULE_COMMAND: &str = concat!(
    "PATH=/usr/sbin:/usr/bin:/sbin:/bin; export PATH; ",
    "mkdir -p /dev /proc /sys /tmp; ",
    "test -e /dev/ttyAMA0 || mount -t devtmpfs devtmpfs /dev; ",
    "test -e /proc/modules || mount -t proc proc /proc; ",
    "test -d /sys/module || mount -t sysfs sysfs /sys; ",
    "modprobe virtio_gpu; r=$?; ",
    "if test \"$r\" -eq 0 && test -c /dev/dri/card0; then printf 'VIRGL_SMOKE_MODULE_OK\\n'; ",
    "else printf 'VIRGL_SMOKE_MODULE_FAIL:%s\\n' \"$r\"; fi\r"
);
#[derive(Clone, Copy, Debug, PartialEq, Eq)] enum Phase { Shell, Module, Packet, Result }
fn main() -> Result<(), Box<dyn Error>> {
    let disk = env::args().nth(1).unwrap_or_else(|| "output/webboxvm-final-install-compact.wbdisk".into());
    let demo = env::args().nth(2).unwrap_or_else(|| "guest/virgl-clear-demo/build/virgl-clear-demo".into());
    let max_steps = setting("VIRGL_SMOKE_MAX_STEPS", 20_000_000_000);
    let chunk_steps = setting("VIRGL_SMOKE_CHUNK_STEPS", 2_000_000).max(1) as usize;
    let timeout = Duration::from_secs(setting("VIRGL_SMOKE_TIMEOUT_SECS", 900));
    let binary = fs::read(&demo)?;
    let mut vm = BootContext::new_from_install_disk_snapshot_with_extra_bootargs(fs::read(&disk)?, 1, "init=/bin/sh")?;
    vm.run_efi_phase(usize::MAX);
    println!("standard VirGL guest transport smoke: {disk}, {} bytes", binary.len());
    let start = Instant::now();
    let mut phase = Phase::Shell;
    let mut uart_offset = 0; let mut steps = 0u64;
    let (mut clear_sequence, mut draw_sequence, mut texture_pair_sequence, mut vertex_color_sequence, mut texture_color_sequence, mut uniform_sequence, mut depth_sequence, mut batch_sequence, mut depth_batch_sequence, mut depth_equal_sequence, mut depth_equal_batch_sequence, mut depth_mixed_batch_sequence, mut depth_write_mask_batch_sequence, mut depth_vertex_color_sequence, mut depth_texture_sequence) = (None, None, None, None, None, None, None, None, None, None, None, None, None, None, None); let mut texture_sequence = None;
    let (mut upload_readback, mut clear_completed) = (false, false); let (mut draw_completed, mut depth_completed, mut batch_completed, mut depth_batch_completed, mut depth_equal_completed, mut depth_equal_batch_completed, mut depth_mixed_batch_completed, mut depth_write_mask_completed) = (false, false, false, false, false, false, false, false);
    let (mut repeat_completed, mut linear_completed, mut texture_pair_completed, mut vertex_color_completed, mut texture_color_completed, mut uniform_completed, mut depth_vertex_color_completed) = (false, false, false, false, false, false, false);
    while steps < max_steps && start.elapsed() < timeout {
        let slice = if phase == Phase::Result { 10_000 } else { chunk_steps };
        let ran = vm.run_kernel_phase(slice.min((max_steps - steps) as usize));
        steps += ran as u64;
        let delta = vm.uart_output_since(uart_offset);
        uart_offset = vm.uart_output_len();
        if !delta.is_empty() { println!("UART +{} bytes: {:?}", delta.len(), delta.chars().take(240).collect::<String>()); }
        let uart = vm.uart_output();
        if output_starts(&uart, MODULE_FAIL) || output_starts(&uart, FAIL) {
            return Err(format!("guest reported failure:\n{}", tail(&uart)).into());
        }
        match phase {
            Phase::Shell if shell_ready(&uart) => {
                vm.feed_uart_input(MODULE_COMMAND);
                phase = Phase::Module;
            }
            Phase::Module if output_line(&uart, MODULE_OK) => {
                vm.feed_uart_input(&demo_script(&binary));
                phase = Phase::Packet;
            }
            Phase::Packet => {
                let packet = vm.machine.bus.virtio_gpu.take_3d_update();
                if !packet.is_empty() {
                    match virgl_packet(&packet)? {
                        VirglPacket::Clear(sequence) if clear_sequence.is_none() => { println!("VGC1 validated: sequence {sequence}"); clear_sequence = Some(sequence); }
                        VirglPacket::Draw(sequence)
                            if clear_completed && draw_sequence.is_none() =>
                        {
                            println!("VGD1 validated: sequence {sequence}");
                            draw_sequence = Some(sequence);
                        }
                        VirglPacket::TexturedDraw(sequence, mode)
                            if draw_completed && texture_sequence.is_none() =>
                        {
                            let expected = [TextureMode::Repeat, TextureMode::Linear]
                                [repeat_completed as usize];
                            if mode != expected {
                                return Err("guest emitted textured samplers out of order".into());
                            }
                            println!("VGD1 {mode:?} texture validated: sequence {sequence}");
                            texture_sequence = Some((sequence, mode));
                        }
                        VirglPacket::TexturePairDraw(sequence)
                            if linear_completed && texture_pair_sequence.is_none() =>
                        {
                            println!("VGD1 texture pair validated: sequence {sequence}");
                            texture_pair_sequence = Some(sequence);
                        }
                        VirglPacket::VertexColorDraw(sequence)
                            if texture_pair_completed && vertex_color_sequence.is_none() =>
                        {
                            println!("VGD1 vertex-color validated: sequence {sequence}");
                            vertex_color_sequence = Some(sequence);
                        }
                        VirglPacket::TextureColorDraw(sequence) if vertex_color_completed && texture_color_sequence.is_none() => { println!("VGD1 texture-color validated: sequence {sequence}"); texture_color_sequence = Some(sequence); }
                        VirglPacket::UniformDraw(sequence) if texture_color_completed && uniform_sequence.is_none() => { println!("VGD1 uniform-buffer validated: sequence {sequence}"); uniform_sequence = Some(sequence); }
                        VirglPacket::DepthDraw(sequence) if uniform_completed && depth_sequence.is_none() => { println!("VGD1 depth validated: sequence {sequence}"); depth_sequence = Some(sequence); }
                        VirglPacket::SolidBatch(sequence) if depth_completed && batch_sequence.is_none() => { println!("VGB1 solid-batch validated: sequence {sequence}"); batch_sequence = Some(sequence); }
                        VirglPacket::DepthBatch(sequence) if batch_completed && depth_batch_sequence.is_none() => { println!("VGB1 depth-batch validated: sequence {sequence}"); depth_batch_sequence = Some(sequence); }
                        VirglPacket::DepthEqualDraw(sequence) if depth_batch_completed && depth_equal_sequence.is_none() => { println!("VGD1 depth-equal validated: sequence {sequence}"); depth_equal_sequence = Some(sequence); }
                        VirglPacket::DepthEqualBatch(sequence) if depth_equal_completed && depth_equal_batch_sequence.is_none() => { println!("VGB1 depth-equal-batch validated: sequence {sequence}"); depth_equal_batch_sequence = Some(sequence); }
                        VirglPacket::DepthMixedBatch(sequence) if depth_equal_batch_completed && depth_mixed_batch_sequence.is_none() => { println!("VGB1 depth-mixed-batch validated: sequence {sequence}"); depth_mixed_batch_sequence = Some(sequence); }
                        VirglPacket::DepthWriteMaskBatch(sequence) if depth_mixed_batch_completed && depth_write_mask_batch_sequence.is_none() => { println!("VGB1 depth-write-mask-batch validated: sequence {sequence}"); depth_write_mask_batch_sequence = Some(sequence); }
                        VirglPacket::DepthVertexColorDraw(sequence) if depth_write_mask_completed && depth_vertex_color_sequence.is_none() => { println!("VGD1 depth-vertex-color validated: sequence {sequence}"); depth_vertex_color_sequence = Some(sequence); }
                        VirglPacket::DepthTextureDraw(sequence) if depth_vertex_color_completed && depth_texture_sequence.is_none() => { println!("VGD1 depth-texture validated: sequence {sequence}"); depth_texture_sequence = Some(sequence); }
                        _ => return Err("guest emitted an unexpected VirGL packet".into()),
                    }
                }
            }
            _ => {}
        }
        let frame = vm.machine.bus.virtio_gpu.take_scanout_update();
        if phase == Phase::Packet && !frame.is_empty() && is_upload_readback(&frame) {
            upload_readback = true;
            println!("WBGF standard VirGL upload readback validated");
        }
        if phase == Phase::Packet
            && !clear_completed
            && upload_readback
            && let Some(sequence) = clear_sequence
        {
            complete(&mut vm, sequence, is_clear_readback, "VGC1")?;
            println!("VGC1 completed after upload readback: sequence {sequence}");
            println!("WBGF full-scanout BGRA readback validated");
            clear_completed = true;
        }
        if phase == Phase::Packet
            && clear_completed
            && !draw_completed
            && let Some(sequence) = draw_sequence
        {
            complete(&mut vm, sequence, is_triangle_readback, "VGD1")?;
            println!("WBGF triangle BGRA readback validated");
            draw_completed = true;
        }
        if phase == Phase::Packet
            && draw_completed
            && let Some((sequence, mode)) = texture_sequence.take()
        {
            complete(
                &mut vm,
                sequence,
                |frame| is_textured_triangle_readback(frame, mode),
                "VGD1 texture",
            )?;
            println!("WBGF {mode:?} textured triangle BGRA readback validated");
            if mode == TextureMode::Repeat { repeat_completed = true; } else { linear_completed = true; }
        }
        if phase == Phase::Packet && linear_completed && let Some(sequence) = texture_pair_sequence.take() {
            complete(&mut vm, sequence, is_texture_pair_readback, "VGD1 texture pair")?;
            println!("WBGF independent sampler texture-pair BGRA readback validated");
            texture_pair_completed = true;
        }
        if phase == Phase::Packet && texture_pair_completed && let Some(sequence) = vertex_color_sequence.take() {
            complete(&mut vm, sequence, is_vertex_color_readback, "VGD1 vertex-color")?;
            println!("WBGF interpolated vertex-color BGRA readback validated");
            vertex_color_completed = true;
        }
        if phase == Phase::Packet && vertex_color_completed && let Some(sequence) = texture_color_sequence.take() {
            complete(&mut vm, sequence, is_texture_color_readback, "VGD1 texture-color")?;
            println!("WBGF texture-modulated vertex-color BGRA readback validated");
            texture_color_completed = true;
        }
        if phase == Phase::Packet && texture_color_completed && let Some(sequence) = uniform_sequence.take() {
            complete(&mut vm, sequence, is_uniform_readback, "VGD1 uniform-buffer")?;
            println!("WBGF uniform-buffer triangle BGRA readback validated");
            uniform_completed = true;
        }
        if phase == Phase::Packet && uniform_completed && let Some(sequence) = depth_sequence.take() {
            complete(&mut vm, sequence, is_depth_readback, "VGD1 depth")?;
            println!("WBGF depth-tested triangle BGRA readback validated");
            depth_completed = true;
        }
        if phase == Phase::Packet && depth_completed && let Some(sequence) = batch_sequence.take() { complete(&mut vm, sequence, is_solid_batch_readback, "VGB1 solid batch")?; println!("WBGF ordered solid-batch BGRA readback validated"); batch_completed = true; }
        if phase == Phase::Packet && batch_completed && let Some(sequence) = depth_batch_sequence.take() { complete(&mut vm, sequence, is_depth_batch_readback, "VGB1 depth batch")?; println!("WBGF depth-tested batch BGRA readback validated"); depth_batch_completed = true; }
        if phase == Phase::Packet && depth_batch_completed && let Some(sequence) = depth_equal_sequence.take() { complete(&mut vm, sequence, is_depth_equal_readback, "VGD1 depth equal")?; println!("WBGF depth-equal BGRA readback validated"); depth_equal_completed = true; }
        if phase == Phase::Packet && depth_equal_completed && let Some(sequence) = depth_equal_batch_sequence.take() { complete(&mut vm, sequence, is_depth_equal_batch_readback, "VGB1 depth equal batch")?; println!("WBGF depth-equal batch BGRA readback validated"); depth_equal_batch_completed = true; }
        if phase == Phase::Packet && depth_equal_batch_completed && let Some(sequence) = depth_mixed_batch_sequence.take() { complete(&mut vm, sequence, is_depth_mixed_batch_readback, "VGB1 depth mixed batch")?; println!("WBGF depth-mixed batch BGRA readback validated"); depth_mixed_batch_completed = true; }
        if phase == Phase::Packet && depth_mixed_batch_completed && let Some(sequence) = depth_write_mask_batch_sequence.take() { complete(&mut vm, sequence, is_depth_write_mask_batch_readback, "VGB1 depth write-mask batch")?; println!("WBGF depth-write-mask batch BGRA readback validated"); depth_write_mask_completed = true; }
        if phase == Phase::Packet && depth_write_mask_completed && let Some(sequence) = depth_vertex_color_sequence.take() { complete(&mut vm, sequence, is_vertex_color_readback, "VGD1 depth vertex-color")?; println!("WBGF depth-tested vertex-color BGRA readback validated"); depth_vertex_color_completed = true; }
        if phase == Phase::Packet && depth_vertex_color_completed && let Some(sequence) = depth_texture_sequence.take() { complete(&mut vm, sequence, is_depth_texture_readback, "VGD1 depth texture")?; println!("WBGF depth-tested texture BGRA readback validated"); phase = Phase::Result; }
        if phase == Phase::Result && uart.contains(PASS) {
            println!("PASS: {PASS}; steps={steps}; seconds={:.3}", start.elapsed().as_secs_f64());
            return Ok(());
        }
    }
    Err(format!("VirGL smoke timed out in {phase:?} after {steps} steps:\n{}", tail(&vm.uart_output())).into())
}
fn setting(name: &str, default: u64) -> u64 { env::var(name).ok().and_then(|value| value.parse().ok()).unwrap_or(default) }
