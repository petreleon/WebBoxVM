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

    #[cfg(test)]
    pub(crate) fn clear_exclusive_overlaps(&mut self, pa: u64, size: u8) {
        self.clear_exclusive_overlaps_from(None, pa, size);
    }

    #[cfg(feature = "wasm")]
    pub(crate) fn clear_guest_exclusive_overlaps(&mut self, writer: usize, pa: u64, size: u8) {
        self.clear_exclusive_overlaps_from(Some(writer), pa, size);
    }

    #[cfg(any(test, feature = "wasm"))]
    fn clear_exclusive_overlaps_from(&mut self, writer: Option<usize>, pa: u64, size: u8) {
        for (core, cpu) in self.cpus.iter_mut().enumerate() {
            let monitored = cpu.exclusive.is_some();
            cpu.clear_exclusive_if_overlaps(pa, size);
            if monitored && cpu.exclusive.is_none() && Some(core) != writer {
                super::events::signal_cpu_event(cpu);
            }
        }
    }

    pub(crate) fn apply_memory_write_invalidations(&mut self, writer: usize) {
        let (cpus, bus) = (&mut self.cpus, &mut self.bus);
        let wrote_memory = bus.dma_write_during_instruction() || !bus.memory_writes().is_empty();
        if !wrote_memory {
            bus.finish_cpu_instruction();
            return;
        }
        if bus.dma_write_during_instruction() {
            for (core, cpu) in cpus.iter_mut().enumerate() {
                let monitored = cpu.exclusive.is_some();
                cpu.clear_exclusive();
                if monitored && core != writer {
                    super::events::signal_cpu_event(cpu);
                }
            }
        } else {
            for write in bus.memory_writes() {
                for (core, cpu) in cpus.iter_mut().enumerate() {
                    let monitored = cpu.exclusive.is_some();
                    cpu.clear_exclusive_range_if_overlaps(write.addr, write.len);
                    if monitored && cpu.exclusive.is_none() && core != writer {
                        super::events::signal_cpu_event(cpu);
                    }
                }
            }
        }
        self.memory_epoch = self.memory_epoch.wrapping_add(1);
        bus.finish_cpu_instruction();
    }

    pub(crate) fn apply_external_dma_write_invalidations(&mut self) {
        if self.bus.take_external_dma_write() {
            for cpu in &mut self.cpus {
                let monitored = cpu.exclusive.is_some();
                cpu.clear_exclusive();
                if monitored {
                    super::events::signal_cpu_event(cpu);
                }
            }
        }
    }
}
