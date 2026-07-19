//! Measure installed-disk preparation separately from guest execution.
//!
//! Run:
//!   cargo run -p emulator --example boot_disk_bench --release -- \
//!     output/webboxvm-final-install-compact.wbdisk
//!
//! Set `BOOT_BENCH_STEPS` to execute a bounded guest instruction budget after
//! the direct ARM64 kernel handoff. The default is zero so the reported
//! firmware-fast-boot preparation time is not mixed with Linux boot time.
//! Set `BOOT_BENCH_STAGED_SMP=1` to include the guarded two-core staging path.

use emulator::boot::BootContext;
use emulator::runtime::RunBackend;
use std::env;
use std::fs;
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args()
        .nth(1)
        .unwrap_or_else(|| "output/webboxvm-final-install-compact.wbdisk".to_string());
    let cores = env_usize("BOOT_BENCH_CORES", 2).max(1);
    let steps = env_usize("BOOT_BENCH_STEPS", 0);
    let staged_requested = env::var_os("BOOT_BENCH_STAGED_SMP").is_some();
    let total_start = Instant::now();

    let read_start = Instant::now();
    let snapshot = fs::read(&path)?;
    let read_elapsed = read_start.elapsed();

    let prepare_start = Instant::now();
    let (mut vm, staged_smp) = if staged_requested {
        BootContext::new_from_install_disk_snapshot_with_staged_smp(snapshot, cores, "", true)?
    } else {
        (
            BootContext::new_from_install_disk_snapshot(snapshot, cores)?,
            false,
        )
    };
    let prepare_elapsed = prepare_start.elapsed();
    if env::var_os("BOOT_BENCH_EMPTY_DISK").is_some() {
        vm.set_install_disk_size(vm.install_disk_size_bytes());
    }
    if cores > 1 {
        vm.machine.set_run_backend(RunBackend::NativeThreads);
    }

    println!("WebBoxVM installed-disk boot benchmark");
    println!("source: {path}");
    println!("cores: {cores}");
    println!("staged SMP: {staged_smp}");
    println!("snapshot read: {:.3}s", read_elapsed.as_secs_f64());
    println!(
        "firmware preparation: {:.3}s",
        prepare_elapsed.as_secs_f64()
    );
    println!(
        "runtime disk: {}",
        if env::var_os("BOOT_BENCH_EMPTY_DISK").is_some() {
            "empty control"
        } else {
            "copy-on-write snapshot"
        }
    );
    println!(
        "firmware guest instructions: {}",
        vm.run_efi_phase(usize::MAX)
    );
    println!("kernel entry: {:#018x}", vm.pc());

    if steps > 0 {
        let run_start = Instant::now();
        let ran = vm.run_kernel_phase(steps);
        let run_elapsed = run_start.elapsed();
        println!(
            "guest execution: {ran} steps in {:.3}s ({:.3} MIPS)",
            run_elapsed.as_secs_f64(),
            ran as f64 / run_elapsed.as_secs_f64() / 1_000_000.0
        );
        println!("post-run PC: {:#018x}", vm.pc());
        println!("UART bytes: {}", vm.uart_output_len());
    }

    println!(
        "total host time: {:.3}s",
        total_start.elapsed().as_secs_f64()
    );
    Ok(())
}

fn env_usize(name: &str, default: usize) -> usize {
    env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}
