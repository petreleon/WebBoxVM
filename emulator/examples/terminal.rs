//! Interactive serial terminal for WebBoxVM.
//!
//! Run: cargo run -p emulator --example terminal --release -- .artifacts/Image

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use emulator::host::native::{NativeBootSource, boot_from_image};
use std::env;
use std::error::Error;
use std::fs;
use std::io::{self, Write};
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn Error>> {
    let kernel_path = env::args()
        .nth(1)
        .unwrap_or_else(|| ".artifacts/Image".to_string());
    let image = fs::read(&kernel_path)?;
    let boot = boot_from_image(&kernel_path, &image, 1, None)?;
    if let NativeBootSource::Iso(info) = &boot.source {
        eprintln!("iso kernel: {}", info.kernel_path);
        eprintln!("iso initrd: {}", info.initrd_paths.join(", "));
        eprintln!("iso bootargs: {}", info.bootargs);
    }
    let mut ctx = boot.context;

    eprintln!("WebBoxVM terminal");
    eprintln!("image: {kernel_path}");
    eprintln!("exit: Ctrl-]");
    eprintln!();

    enable_raw_mode()?;
    let _raw_mode = RawModeGuard;

    let mut stdout = io::stdout();
    let mut last_uart = 0usize;
    let mut last_status = Instant::now();

    loop {
        ctx.run_kernel_phase(100_000);

        let new_output = ctx.uart_output_since(last_uart);
        if !new_output.is_empty() {
            last_uart = ctx.uart_output_len();
            stdout.write_all(new_output.as_bytes())?;
            stdout.flush()?;
        }

        while event::poll(Duration::ZERO)? {
            let Event::Key(key) = event::read()? else {
                continue;
            };
            if key.kind != KeyEventKind::Press {
                continue;
            }
            if key.modifiers.contains(KeyModifiers::CONTROL)
                && matches!(key.code, KeyCode::Char(']'))
            {
                writeln!(stdout, "\r\n")?;
                return Ok(());
            }
            if let Some(bytes) = key_to_uart_bytes(key) {
                ctx.feed_uart_bytes(&bytes);
            }
        }

        if last_uart == 0 && last_status.elapsed() >= Duration::from_secs(2) {
            write!(
                stdout,
                "\rbooting... {}M steps, PC=0x{:016x}",
                ctx.total_steps() / 1_000_000,
                ctx.pc()
            )?;
            stdout.flush()?;
            last_status = Instant::now();
        }
    }
}

fn key_to_uart_bytes(key: KeyEvent) -> Option<Vec<u8>> {
    match key.code {
        KeyCode::Backspace => Some(vec![0x7f]),
        KeyCode::Enter => Some(vec![b'\r']),
        KeyCode::Tab => Some(vec![b'\t']),
        KeyCode::Esc => Some(vec![0x1b]),
        KeyCode::Left => Some(b"\x1b[D".to_vec()),
        KeyCode::Right => Some(b"\x1b[C".to_vec()),
        KeyCode::Up => Some(b"\x1b[A".to_vec()),
        KeyCode::Down => Some(b"\x1b[B".to_vec()),
        KeyCode::Delete => Some(b"\x1b[3~".to_vec()),
        KeyCode::Home => Some(b"\x1b[H".to_vec()),
        KeyCode::End => Some(b"\x1b[F".to_vec()),
        KeyCode::Char(c) if key.modifiers.contains(KeyModifiers::CONTROL) => {
            let lower = c.to_ascii_lowercase();
            if lower.is_ascii_lowercase() {
                Some(vec![(lower as u8) & 0x1f])
            } else {
                None
            }
        }
        KeyCode::Char(c) => {
            let mut buf = [0u8; 4];
            Some(c.encode_utf8(&mut buf).as_bytes().to_vec())
        }
        _ => None,
    }
}

struct RawModeGuard;

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
    }
}
