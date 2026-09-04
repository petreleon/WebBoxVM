use super::*;

impl SystemBus {
    pub fn new() -> Self {
        Self::with_cpu_count(1)
    }

    pub fn with_cpu_count(num_cpus: usize) -> Self {
        assert!(num_cpus > 0, "SystemBus requires at least one CPU");
        assert!(
            num_cpus <= GICR_MAX_CPUS,
            "SystemBus CPU count exceeds the redistributor MMIO aperture"
        );
        Self {
            mem: PhysicalMemory::new(),
            uart: Pl011Uart::new(),
            gic: Gicv3::with_cpu_count(num_cpus),
            virtio_blk: VirtioBlk::new(),
            virtio_disk: VirtioBlk::writable_sparse(
                crate::devices::virtio_blk::DEFAULT_SPARSE_DISK_SIZE,
                b"webboxvm-disk\0",
            ),
            virtio_net: VirtioNet::new(),
            virtio_gpu: VirtioGpu::new(),
            uart_rx_refresh_needed: false,
            memory_writes: Vec::new(),
            dma_write_during_instruction: false,
            external_dma_write_pending: false,
        }
    }

    pub(crate) fn cold_reset_devices(&mut self, num_cpus: usize) {
        self.uart.cold_reset();
        self.gic = Gicv3::with_cpu_count(num_cpus);
        self.virtio_blk.cold_reset();
        self.virtio_disk.cold_reset();
        self.virtio_net = VirtioNet::new();
        self.virtio_gpu.cold_reset(&mut self.mem);
        self.uart_rx_refresh_needed = false;
        self.memory_writes.clear();
        self.dma_write_during_instruction = false;
        self.external_dma_write_pending = false;
    }
}

impl Default for SystemBus {
    fn default() -> Self {
        Self::new()
    }
}
