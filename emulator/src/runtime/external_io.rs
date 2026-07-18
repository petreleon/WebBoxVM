use super::*;

impl Machine {
    pub fn inject_irq(&mut self, int_id: u32) {
        self.bus.set_irq_pending(int_id);
    }

    pub fn feed_uart_input(&mut self, input: &str) {
        self.feed_uart_bytes(input.as_bytes());
    }

    pub fn feed_uart_bytes(&mut self, bytes: &[u8]) {
        self.bus.feed_uart_bytes(bytes);
    }

    pub fn inject_network_frame(&mut self, frame: &[u8]) {
        self.bus.inject_network_frame(frame);
        self.apply_external_dma_write_invalidations();
    }

    #[cfg(any(test, feature = "wasm"))]
    pub(crate) fn clear_exclusive_overlaps(&mut self, pa: u64, size: u8) {
        for cpu in &mut self.cpus {
            cpu.clear_exclusive_if_overlaps(pa, size);
        }
    }

    pub(crate) fn apply_memory_write_invalidations(&mut self) {
        let (cpus, bus) = (&mut self.cpus, &mut self.bus);
        let wrote_memory = bus.dma_write_during_instruction() || !bus.memory_writes().is_empty();
        if bus.dma_write_during_instruction() {
            for cpu in cpus.iter_mut() {
                cpu.clear_exclusive();
            }
        } else {
            for write in bus.memory_writes() {
                for cpu in cpus.iter_mut() {
                    cpu.clear_exclusive_range_if_overlaps(write.addr, write.len);
                }
            }
        }
        if wrote_memory {
            self.memory_epoch = self.memory_epoch.wrapping_add(1);
        }
        bus.finish_cpu_instruction();
    }

    pub(crate) fn apply_external_dma_write_invalidations(&mut self) {
        if self.bus.take_external_dma_write() {
            for cpu in &mut self.cpus {
                cpu.clear_exclusive();
            }
        }
    }
}
