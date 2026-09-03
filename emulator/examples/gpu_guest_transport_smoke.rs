//! Native Linux-driver-to-virtio-gpu transport smoke for the private WBG3 capset.
//! Run via `scripts/gpu_guest_transport_smoke.sh` so the guest artifact is verified.

#[path = "gpu_guest_transport_smoke/diagnostics.rs"]
mod diagnostics;
#[path = "gpu_guest_transport_smoke/support.rs"]
mod support;

use diagnostics::*;
use emulator::boot::BootContext;
use std::env;
use std::error::Error;
use std::fs;
use std::time::{Duration, Instant};
use support::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Shell,
    ShellReply,
    ModuleReply,
    NodesReply,
    Packet,
    Pass,
    Scanout,
}

fn main() -> Result<(), Box<dyn Error>> {
    let disk_path = env::args()
        .nth(1)
        .unwrap_or_else(|| "output/webboxvm-final-install-compact.wbdisk".into());
    let demo_path = env::args()
        .nth(2)
        .unwrap_or_else(|| "guest/webgpu-demo/build/webgpu-demo".into());
    let max_steps = env_u64("GPU_SMOKE_MAX_STEPS", 20_000_000_000);
    let chunk_steps = env_usize("GPU_SMOKE_CHUNK_STEPS", 2_000_000).max(1);
    let timeout = Duration::from_secs(env_u64("GPU_SMOKE_TIMEOUT_SECS", 900));

    let demo = fs::read(&demo_path)?;
    let expected = embedded_packet(&demo).map_err(invalid)?;
    let snapshot = fs::read(&disk_path)?;
    let mut vm = BootContext::new_from_install_disk_snapshot_with_extra_bootargs(
        snapshot,
        1,
        "init=/bin/sh",
    )
    .map_err(invalid)?;
    vm.run_efi_phase(usize::MAX);

    println!("WebBoxVM native GPU guest transport smoke");
    println!("disk: {disk_path}");
    println!("demo: {demo_path} ({} bytes)", demo.len());
    println!(
        "bounds: {max_steps} guest steps, {}s host time",
        timeout.as_secs()
    );

    let start = Instant::now();
    let max_chunks = max_steps.div_ceil(chunk_steps as u64);
    let mut phase = Phase::Shell;
    let mut chunks = 0u64;
    let mut executed = 0u64;
    let mut last_uart = 0usize;
    let mut scanout = ScanoutStats::default();

    // Termination invariant: chunks strictly increases and is bounded by max_chunks;
    // each run is also capped by the remaining instruction and wall-clock budgets.
    while chunks < max_chunks && start.elapsed() < timeout {
        let remaining = max_steps.saturating_sub(executed);
        if remaining == 0 {
            break;
        }
        let requested = chunk_steps.min(remaining as usize);
        executed = executed.saturating_add(vm.run_kernel_phase(requested) as u64);
        chunks += 1;
        report_uart(&vm, &mut last_uart);
        scanout
            .poll(&mut vm)
            .map_err(|why| smoke_error(why, &vm.uart_output()))?;

        let uart = vm.uart_output();
        if let Some(marker) = DEMO_FAILURES.iter().find(|marker| uart.contains(**marker)) {
            return Err(smoke_error(format!("guest reported {marker}"), &uart));
        }
        if uart.contains(MODPROBE_FAIL) {
            return Err(smoke_error("modprobe virtio_gpu failed", &uart));
        }

        match phase {
            Phase::Shell if shell_prompt_ready(&uart) => {
                vm.feed_uart_input(SHELL_PROBE);
                phase = Phase::ShellReply;
                println!("state: shell prompt observed; readiness probe sent");
            }
            Phase::ShellReply if uart.contains(SHELL_READY) => {
                vm.feed_uart_input(MODPROBE_COMMAND);
                phase = Phase::ModuleReply;
                println!("state: shell handshake complete; modprobe sent");
            }
            Phase::ModuleReply if uart.contains(MODPROBE_OK) => {
                vm.feed_uart_input(DRM_NODES_COMMAND);
                phase = Phase::NodesReply;
                println!("state: virtio_gpu loaded; DRM node inventory requested");
            }
            Phase::NodesReply if uart.contains(DRM_NODES_END) => {
                vm.feed_uart_input(&demo_script(&demo));
                phase = Phase::Packet;
                println!("state: DRM nodes captured; verified demo injected and run");
            }
            Phase::Packet => {
                let packet = vm.machine.bus.virtio_gpu.take_3d_update();
                if !packet.is_empty() {
                    let sequence = validate_transported_packet(&packet, &expected)
                        .map_err(|why| smoke_error(why, &uart))?;
                    if !vm.machine.bus.complete_gpu_3d(sequence, true) {
                        return Err(smoke_error("public GPU completion rejected", &uart));
                    }
                    phase = Phase::Pass;
                    println!("state: WBG3 packet validated; sequence {sequence} completed");
                }
            }
            Phase::Pass => {
                if let Some(marker) = DEMO_PASSES.iter().find(|marker| uart.contains(**marker)) {
                    if scanout.has_frame() {
                        report_success(marker, &scanout, &uart, executed, &start);
                        return Ok(());
                    }
                    phase = Phase::Scanout;
                    println!("state: exact guest PASS observed; waiting for nonempty WBGF");
                }
            }
            Phase::Scanout if scanout.has_frame() => {
                let marker = DEMO_PASSES
                    .iter()
                    .find(|marker| uart.contains(**marker))
                    .expect("Pass phase requires a guest marker");
                report_success(marker, &scanout, &uart, executed, &start);
                return Ok(());
            }
            _ => {}
        }

        if chunks % 100 == 0 {
            println!(
                "progress: {executed} steps, phase {phase:?}, PC={:#x}",
                vm.pc()
            );
        }
    }

    let uart = vm.uart_output();
    scanout.report();
    report_drm_nodes(&uart);
    report_drm_error_windows(&uart);
    Err(smoke_error(
        format!("budget exhausted after {executed} steps in phase {phase:?}"),
        &uart,
    ))
}

fn report_success(
    marker: &str,
    scanout: &ScanoutStats,
    uart: &str,
    executed: u64,
    start: &Instant,
) {
    println!("PASS: {marker}");
    scanout.report();
    report_drm_nodes(uart);
    report_drm_error_windows(uart);
    println!(
        "guest steps: {executed}; host time: {:.3}s",
        start.elapsed().as_secs_f64()
    );
}
