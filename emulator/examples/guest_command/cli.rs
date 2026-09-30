use super::budget::{Budget, CHUNK_STEPS, MAX_STEPS, TIMEOUT_SECS, UART_BYTES};
use emulator::boot::DEFAULT_BOOTARGS;
use std::path::PathBuf;

pub struct Options {
    pub kernel: PathBuf,
    pub initrd: PathBuf,
    pub disk: Option<PathBuf>,
    pub command_files: Vec<PathBuf>,
    pub bootargs: String,
    pub ready_marker: String,
    pub required: Vec<String>,
    pub uart_log: PathBuf,
    pub max_steps: u64,
    pub chunk_steps: usize,
    pub timeout_secs: u64,
    pub uart_bytes: usize,
}

impl Options {
    pub fn usage() -> &'static str {
        USAGE
    }

    pub fn parse(arguments: impl Iterator<Item = String>) -> Result<Self, String> {
        let mut kernel = None;
        let mut initrd = None;
        let mut disk = None;
        let mut uart_log = None;
        let mut command_files = Vec::new();
        let mut required = Vec::new();
        let mut bootargs = DEFAULT_BOOTARGS.to_string();
        let mut ready_marker = "WEBBOXVM_GUEST_READY".to_string();
        let (mut max_steps, mut chunk_steps, mut timeout_secs, mut uart_bytes) =
            (MAX_STEPS, CHUNK_STEPS, TIMEOUT_SECS, UART_BYTES);
        let mut arguments = arguments;
        while let Some(option) = arguments.next() {
            if option == "--help" {
                return Err(USAGE.into());
            }
            let value = arguments.next().ok_or_else(|| format!("missing value for {option}\n{USAGE}"))?;
            match option.as_str() {
                "--kernel" => unique(&mut kernel, PathBuf::from(value), &option)?,
                "--initrd" => unique(&mut initrd, PathBuf::from(value), &option)?,
                "--disk" => unique(&mut disk, PathBuf::from(value), &option)?,
                "--uart-log" => unique(&mut uart_log, PathBuf::from(value), &option)?,
                "--command-file" => command_files.push(PathBuf::from(value)),
                "--bootargs" => bootargs = value,
                "--ready-marker" => ready_marker = value,
                "--require" => required.push(value),
                "--max-steps" => max_steps = number(&value, &option)?,
                "--chunk-steps" => chunk_steps = number(&value, &option)?,
                "--timeout-secs" => timeout_secs = number(&value, &option)?,
                "--uart-bytes" => uart_bytes = number(&value, &option)?,
                _ => return Err(format!("unknown option {option}\n{USAGE}")),
            }
        }
        if command_files.is_empty() || command_files.len() > 16 {
            return Err("supply between one and sixteen --command-file paths".into());
        }
        if ready_marker.is_empty()
            || ready_marker.len() > 128
            || ready_marker.bytes().any(|b| b.is_ascii_control())
        {
            return Err("ready marker must be one nonempty bounded text line".into());
        }
        if required.len() > 32
            || required.iter().any(|text| {
                text.is_empty() || text.len() > 4096 || text.bytes().any(|b| b.is_ascii_control())
            })
        {
            return Err("requirements must be complete nonempty text lines (at most 32)".into());
        }
        if bootargs.is_empty() || bootargs.len() > 4096 || bootargs.contains('\0') {
            return Err("bootargs must be nonempty bounded text without NUL".into());
        }
        if !(1..=UART_BYTES).contains(&uart_bytes) {
            return Err("UART cap exceeds standard bounds".into());
        }
        Budget::new(max_steps, chunk_steps, timeout_secs)?;
        Ok(Self {
            kernel: kernel.ok_or("--kernel is required")?,
            initrd: initrd.ok_or("--initrd is required")?,
            disk,
            command_files,
            bootargs,
            ready_marker,
            required,
            uart_log: uart_log.ok_or("--uart-log is required")?,
            max_steps,
            chunk_steps,
            timeout_secs,
            uart_bytes,
        })
    }
}

fn unique<T>(target: &mut Option<T>, value: T, option: &str) -> Result<(), String> {
    if target.is_some() {
        return Err(format!("duplicate option {option}"));
    }
    *target = Some(value);
    Ok(())
}

fn number<T: std::str::FromStr>(value: &str, option: &str) -> Result<T, String> {
    value.parse().map_err(|_| format!("invalid number for {option}"))
}

const USAGE: &str = "guest_command --kernel IMAGE --initrd CPIO --command-file SCRIPT \
    --uart-log NEW_PATH [--command-file SCRIPT ...] [--disk WBDISK] [--bootargs TEXT] \
    [--ready-marker LINE] [--require TEXT ...] [--max-steps N] [--chunk-steps N] \
    [--timeout-secs N] [--uart-bytes N]\nDefaults: 900s, 20 billion steps, 2 million/chunk, 1 MiB UART; bounds may only be reduced.";

#[cfg(test)]
#[path = "tests/cli.rs"]
mod tests;
