use emulator::boot::BootContext;
use emulator::constants::INITRD_BASE;
use std::env;
use std::path::Path;

pub(super) fn kernel_path() -> String {
    env::args().nth(1).unwrap_or_else(|| {
        env::var("WEBBOXVM_KERNEL").unwrap_or_else(|_| ".artifacts/Image".to_string())
    })
}

pub(super) fn is_iso_path(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("iso"))
}

pub(super) fn env_usize(name: &str, default: usize) -> usize {
    env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

pub(super) fn normalize_boot_test_commands(script: &str) -> String {
    let mut normalized = script.replace("\\r", "\r").replace("\\n", "\n");
    normalized = normalized.replace('\n', "\r");
    if !normalized.ends_with('\r') {
        normalized.push('\r');
    }
    normalized
}

pub(super) fn first_initrd_mismatch(ctx: &BootContext, expected: &[u8]) -> Option<usize> {
    let mut guest = vec![0; expected.len()];
    ctx.machine.bus.mem.read_bytes(INITRD_BASE, &mut guest)?;
    guest
        .iter()
        .zip(expected)
        .position(|(actual, expected)| actual != expected)
}
