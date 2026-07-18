#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct MemoryWrite {
    pub(crate) addr: u64,
    pub(crate) len: u64,
}

use super::SystemBus;

impl SystemBus {
    pub(crate) fn begin_cpu_instruction(&mut self) {
        self.memory_writes.clear();
        self.dma_write_during_instruction = false;
    }

    pub(crate) fn memory_writes(&self) -> &[MemoryWrite] {
        &self.memory_writes
    }

    pub(crate) fn dma_write_during_instruction(&self) -> bool {
        self.dma_write_during_instruction
    }

    pub(crate) fn take_external_dma_write(&mut self) -> bool {
        std::mem::take(&mut self.external_dma_write_pending)
    }

    pub(crate) fn finish_cpu_instruction(&mut self) {
        self.memory_writes.clear();
        self.dma_write_during_instruction = false;
    }

    pub(super) fn record_memory_write(&mut self, addr: u64, len: u64) {
        if len != 0 {
            self.memory_writes.push(MemoryWrite { addr, len });
        }
    }

    pub(super) fn record_dma_write(&mut self) {
        self.dma_write_during_instruction = true;
    }

    pub(super) fn record_external_dma_write(&mut self) {
        self.external_dma_write_pending = true;
    }
}
