use super::budget::{Budget, INPUT_BYTES};
use super::cli::Options;
use super::protocol::Protocol;
use super::script::Scripts;
use emulator::boot::BootContext;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

pub fn run(options: Options) -> Result<(), String> {
    let token = format!(
        "{:x}{:x}",
        SystemTime::now().duration_since(UNIX_EPOCH).map_err(|e| e.to_string())?.as_nanos(),
        std::process::id()
    );
    let mut commands = Vec::new();
    let mut input_size = 0;
    for path in &options.command_files {
        let mut bytes = Vec::new();
        File::open(path)
            .map_err(|e| format!("{}: {e}", path.display()))?
            .take((INPUT_BYTES - input_size) as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| e.to_string())?;
        input_size += bytes.len();
        if input_size > INPUT_BYTES {
            return Err("guest command files exceed input cap".into());
        }
        commands.push(String::from_utf8(bytes).map_err(|_| format!("{} is not UTF-8", path.display()))?);
    }
    let scripts = Scripts::new(&token, &commands)?;
    if let Some(parent) = options.uart_log.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut log = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&options.uart_log)
        .map_err(|e| format!("cannot create new UART log {}: {e}", options.uart_log.display()))?;
    let start = Instant::now();
    let kernel = fs::read(&options.kernel).map_err(|e| e.to_string())?;
    let initrd = fs::read(&options.initrd).map_err(|e| e.to_string())?;
    let mut vm = BootContext::new_with_initrd_and_bootargs(&kernel, 1, &initrd, &options.bootargs)?;
    if let Some(disk) = &options.disk {
        vm.restore_install_disk(&fs::read(disk).map_err(|e| e.to_string())?)?;
    }
    let mut budget = Budget::new(options.max_steps, options.chunk_steps, options.timeout_secs)?;
    println!(
        "guest bounds: steps={}, chunk_steps={}, timeout_secs={}, uart_bytes={}",
        options.max_steps, options.chunk_steps, options.timeout_secs, options.uart_bytes
    );
    let mut protocol =
        Protocol::new(token, options.ready_marker, commands.len(), options.required, options.uart_bytes);
    let result = execute(&mut vm, &mut budget, &mut protocol, &scripts, &mut log, &start);
    log.flush().map_err(|e| e.to_string())?;
    println!(
        "guest command result: {}; commands={}; steps={}; chunks={}; seconds={:.3}; uart_bytes={}; phase={}",
        if result.is_ok() { "PASS" } else { "FAIL" },
        commands.len(),
        budget.steps,
        budget.chunks,
        start.elapsed().as_secs_f64(),
        vm.uart_output_len(),
        protocol.phase()
    );
    result
}

trait Guest {
    fn run(&mut self, steps: usize) -> usize;
    fn output(&self) -> &[u8];
    fn input(&mut self, text: &str);
    fn pc(&self) -> u64;
}

impl Guest for BootContext {
    fn run(&mut self, steps: usize) -> usize {
        self.run_kernel_phase(steps)
    }
    fn output(&self) -> &[u8] {
        &self.machine.bus.uart.output
    }
    fn input(&mut self, text: &str) {
        self.feed_uart_input(text);
    }
    fn pc(&self) -> u64 {
        BootContext::pc(self)
    }
}

fn execute(
    vm: &mut impl Guest,
    budget: &mut Budget,
    protocol: &mut Protocol,
    scripts: &Scripts,
    log: &mut impl Write,
    start: &Instant,
) -> Result<(), String> {
    let mut offset = 0;
    loop {
        let requested = budget.next(start.elapsed())?;
        let executed = vm.run(requested);
        budget.record(requested, executed)?;
        let output = &vm.output()[offset..];
        log.write_all(output).map_err(|e| e.to_string())?;
        offset = vm.output().len();
        // Process the entire chunk before accepting success, including trailing failures.
        protocol.feed(output)?;
        budget.check_time(start.elapsed())?;
        if protocol.finished() {
            return Ok(());
        }
        if let Some(action) = protocol.take_action() {
            vm.input(scripts.input(action));
        }
        if budget.chunks % 100 == 0 {
            println!(
                "guest progress: steps={}, chunks={}, phase={}, PC={:#x}",
                budget.steps,
                budget.chunks,
                protocol.phase(),
                vm.pc()
            );
        }
    }
}

#[cfg(test)]
#[path = "tests/run.rs"]
mod tests;
