use emulator::boot::BootContext;
use std::env;
use std::error::Error;
use std::io;

#[derive(Default)]
pub(super) struct ScanoutStats {
    frames: u64,
    payload_bytes: u64,
    last_rect: Option<(u32, u32, u32, u32)>,
}

impl ScanoutStats {
    pub(super) fn poll(&mut self, vm: &mut BootContext) -> Result<(), String> {
        let packet = vm.machine.bus.virtio_gpu.take_scanout_update();
        if packet.is_empty() {
            return Ok(());
        }
        let frame = parse_wbgf(&packet)?;
        self.frames += 1;
        self.payload_bytes += (packet.len() - 32) as u64;
        self.last_rect = Some((frame.x, frame.y, frame.width, frame.height));
        println!(
            "state: WBGF frame {}: {}x{} rect {},{} {}x{} ({} bytes)",
            self.frames,
            frame.scanout_width,
            frame.scanout_height,
            frame.x,
            frame.y,
            frame.width,
            frame.height,
            packet.len()
        );
        Ok(())
    }

    pub(super) fn report(&self) {
        match self.last_rect {
            Some((x, y, width, height)) => println!(
                "WBGF scanout: {} frame(s), {} payload bytes, last rect {x},{y} {width}x{height}",
                self.frames, self.payload_bytes
            ),
            None => println!("WBGF scanout: 0 frames"),
        }
    }

    pub(super) fn has_frame(&self) -> bool {
        self.frames != 0
    }
}

struct Frame {
    scanout_width: u32,
    scanout_height: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

fn parse_wbgf(packet: &[u8]) -> Result<Frame, String> {
    if packet.len() < 32 || packet.get(..4) != Some(b"WBGF") || read_u32(packet, 4) != Some(1) {
        return Err("native scanout returned an invalid WBGF header".into());
    }
    let frame = Frame {
        scanout_width: read_u32(packet, 8).unwrap(),
        scanout_height: read_u32(packet, 12).unwrap(),
        x: read_u32(packet, 16).unwrap(),
        y: read_u32(packet, 20).unwrap(),
        width: read_u32(packet, 24).unwrap(),
        height: read_u32(packet, 28).unwrap(),
    };
    let end_x = frame.x.checked_add(frame.width);
    let end_y = frame.y.checked_add(frame.height);
    let payload = (frame.width as usize)
        .checked_mul(frame.height as usize)
        .and_then(|pixels| pixels.checked_mul(4));
    if frame.scanout_width == 0
        || frame.scanout_height == 0
        || frame.width == 0
        || frame.height == 0
        || end_x.is_none_or(|end| end > frame.scanout_width)
        || end_y.is_none_or(|end| end > frame.scanout_height)
        || payload.and_then(|len| len.checked_add(32)) != Some(packet.len())
    {
        return Err("native scanout returned invalid WBGF dimensions".into());
    }
    Ok(frame)
}

pub(super) fn report_drm_error_windows(uart: &str) {
    let lines: Vec<_> = uart.lines().collect();
    let errors: Vec<_> = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.contains("response 0x1205"))
        .collect();
    println!("DRM response 0x1205 count: {}", errors.len());
    for (number, (index, _)) in errors.iter().enumerate() {
        println!("DRM_ERROR_WINDOW {}", number + 1);
        let start = index.saturating_sub(2);
        let end = (*index + 2).min(lines.len().saturating_sub(1));
        for line in &lines[start..=end] {
            println!("{line}");
        }
    }
}

pub(super) fn report_drm_nodes(uart: &str) {
    let lines: Vec<_> = uart.lines().collect();
    let start = lines
        .iter()
        .position(|line| line == &"GPU_SMOKE_DRM_NODES_BEGIN");
    let end = lines
        .iter()
        .position(|line| line == &"GPU_SMOKE_DRM_NODES_END");
    println!("DRM_NODE_INVENTORY");
    match (start, end) {
        (Some(start), Some(end)) if start < end => {
            for line in &lines[start..=end] {
                println!("{line}");
            }
        }
        _ => println!("DRM node inventory markers missing"),
    }
}

pub(super) fn env_u64(name: &str, default: u64) -> u64 {
    env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

pub(super) fn env_usize(name: &str, default: usize) -> usize {
    env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

pub(super) fn invalid(message: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

pub(super) fn smoke_error(message: impl AsRef<str>, uart: &str) -> Box<dyn Error> {
    invalid(format!(
        "{}\nUART tail:\n{}",
        message.as_ref(),
        uart_tail(uart)
    ))
    .into()
}

pub(super) fn report_uart(vm: &BootContext, offset: &mut usize) {
    let delta = vm.uart_output_since(*offset);
    *offset = vm.uart_output_len();
    if !delta.is_empty() {
        let preview: String = delta.chars().take(240).collect();
        println!("UART +{} bytes: {preview:?}", delta.len());
    }
}

fn uart_tail(uart: &str) -> String {
    let mut chars: Vec<char> = uart.chars().rev().take(2_000).collect();
    chars.reverse();
    chars.into_iter().collect()
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}
