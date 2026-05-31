use super::BootContext;
use crate::constants::PL011_UART_IRQ_ID;

impl BootContext {
    /// No-op: EFI stub is skipped. We boot via the standard ARM64 protocol.
    pub fn run_efi_phase(&mut self, _max_steps: usize) -> usize {
        0
    }

    /// Run the multi-core kernel phase with round-robin scheduling.
    pub fn run_kernel_phase(&mut self, max_steps: usize) -> usize {
        self.machine.run(max_steps)
    }

    pub fn uart_output(&self) -> String {
        self.machine.bus.uart.output_string()
    }

    pub fn uart_output_len(&self) -> usize {
        self.machine.bus.uart.output.len()
    }

    pub fn uart_output_since(&self, offset: usize) -> String {
        let output = &self.machine.bus.uart.output;
        String::from_utf8_lossy(&output[offset.min(output.len())..]).to_string()
    }

    pub fn feed_uart_input(&mut self, input: &str) {
        self.feed_uart_bytes(input.as_bytes());
    }

    pub fn feed_uart_bytes(&mut self, bytes: &[u8]) {
        if bytes.is_empty() {
            return;
        }
        self.machine.bus.uart.feed_input_bytes(bytes);
        self.machine.inject_irq(PL011_UART_IRQ_ID);
    }

    pub fn total_steps(&self) -> u64 {
        self.machine.total_steps
    }

    pub fn pc(&self) -> u64 {
        self.machine.cpus[0].regs.pc
    }

    pub fn allocated_pages(&self) -> usize {
        self.machine.bus.mem.allocated_pages()
    }

    pub fn install_disk_allocated_bytes(&self) -> u64 {
        self.machine.bus.virtio_disk.allocated_storage_bytes()
    }

    pub fn install_disk_size_bytes(&self) -> u64 {
        self.machine.bus.virtio_disk.sparse_disk_size_bytes()
    }

    pub fn install_disk_generation(&self) -> u64 {
        self.machine.bus.virtio_disk.storage_generation()
    }

    pub fn install_disk_snapshot(&self) -> Result<Vec<u8>, String> {
        self.machine.bus.virtio_disk.snapshot_sparse_disk()
    }

    pub fn restore_install_disk(&mut self, snapshot: &[u8]) -> Result<(), String> {
        self.machine.bus.virtio_disk.restore_sparse_disk(snapshot)
    }
}
