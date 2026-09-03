//! Native Linux-driver proof for standard capset-1 VirGL transfer, blend, and draw.

#[path = "virgl_guest_transport_smoke/wire.rs"]
mod wire;

use emulator::boot::BootContext;
use std::env;
use std::error::Error;
use std::fs;
use std::time::{Duration, Instant};
use wire::*;

const MODULE_OK: &str = "VIRGL_SMOKE_MODULE_OK";
const MODULE_FAIL: &str = "VIRGL_SMOKE_MODULE_FAIL";
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Shell,
    Module,
    Packet,
    Result,
}

fn main() -> Result<(), Box<dyn Error>> {
    let disk = env::args()
        .nth(1)
        .unwrap_or_else(|| "output/webboxvm-final-install-compact.wbdisk".into());
    let demo = env::args()
        .nth(2)
        .unwrap_or_else(|| "guest/virgl-clear-demo/build/virgl-clear-demo".into());
    let max_steps = setting("VIRGL_SMOKE_MAX_STEPS", 20_000_000_000);
    let chunk_steps = setting("VIRGL_SMOKE_CHUNK_STEPS", 2_000_000).max(1) as usize;
    let timeout = Duration::from_secs(setting("VIRGL_SMOKE_TIMEOUT_SECS", 900));
    let binary = fs::read(&demo)?;
    let mut vm = BootContext::new_from_install_disk_snapshot_with_extra_bootargs(
        fs::read(&disk)?,
        1,
        "init=/bin/sh",
    )?;
    vm.run_efi_phase(usize::MAX);

    println!(
        "standard VirGL guest transport smoke: {disk}, {} bytes",
        binary.len()
    );
    let start = Instant::now();
    let mut phase = Phase::Shell;
    let mut uart_offset = 0;
    let mut steps = 0u64;
    let mut clear_sequence = None;
    let mut draw_sequence = None;
    let mut texture_sequence = None;
    let mut upload_readback = false;
    let mut clear_completed = false;
    let mut draw_completed = false;
    while steps < max_steps && start.elapsed() < timeout {
        let slice = if phase == Phase::Result {
            10_000
        } else {
            chunk_steps
        };
        let ran = vm.run_kernel_phase(slice.min((max_steps - steps) as usize));
        steps += ran as u64;
        let delta = vm.uart_output_since(uart_offset);
        uart_offset = vm.uart_output_len();
        if !delta.is_empty() {
            println!(
                "UART +{} bytes: {:?}",
                delta.len(),
                delta.chars().take(240).collect::<String>()
            );
        }
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
                        VirglPacket::Clear(sequence) if clear_sequence.is_none() => {
                            println!("VGC1 validated: sequence {sequence}");
                            clear_sequence = Some(sequence);
                        }
                        VirglPacket::Draw(sequence)
                            if clear_completed && draw_sequence.is_none() =>
                        {
                            println!("VGD1 validated: sequence {sequence}");
                            draw_sequence = Some(sequence);
                        }
                        VirglPacket::TexturedDraw(sequence)
                            if draw_completed && texture_sequence.is_none() =>
                        {
                            println!("VGD1 texture validated: sequence {sequence}");
                            texture_sequence = Some(sequence);
                        }
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
            && let Some(sequence) = texture_sequence
        {
            complete(
                &mut vm,
                sequence,
                is_textured_triangle_readback,
                "VGD1 texture",
            )?;
            println!("WBGF textured triangle BGRA readback validated");
            phase = Phase::Result;
        }
        if phase == Phase::Result && uart.contains(PASS) {
            println!(
                "PASS: {PASS}; steps={steps}; seconds={:.3}",
                start.elapsed().as_secs_f64()
            );
            return Ok(());
        }
    }
    Err(format!(
        "VirGL smoke timed out in {phase:?} after {steps} steps:\n{}",
        tail(&vm.uart_output())
    )
    .into())
}

fn setting(name: &str, default: u64) -> u64 {
    env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}
