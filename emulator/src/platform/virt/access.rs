use super::*;
use crate::api::{AccessWidth, PhysAddr};

impl SystemBus {
    pub fn read_phys(&mut self, addr: PhysAddr, width: AccessWidth) -> Option<u64> {
        self.read(addr.get(), width.bytes())
    }

    pub fn write_phys(&mut self, addr: PhysAddr, width: AccessWidth, value: u64) {
        self.write(addr.get(), width.bytes(), value);
    }

    pub fn write_bytes(&mut self, addr: u64, bytes: &[u8]) -> Option<()> {
        if addr < LOW_REGION_END && overlaps_device_range(addr, bytes.len()) {
            return None;
        }
        let result = self.mem.write_bytes(addr, bytes);
        if result.is_some() {
            self.record_memory_write(addr, bytes.len() as u64);
        }
        result
    }

    pub fn overlaps_device_range(&self, addr: u64, len: usize) -> bool {
        overlaps_device_range(addr, len)
    }

    pub fn inject_network_frame(&mut self, frame: &[u8]) {
        if self.virtio_net.inject_rx_frame(&mut self.mem, frame) {
            self.record_external_dma_write();
            self.gic.set_pending(VIRTIO_NET_IRQ_ID);
        }
    }

    pub fn complete_gpu_3d(&mut self, sequence: u32, success: bool) -> bool {
        if !self
            .virtio_gpu
            .complete_3d(&mut self.mem, sequence, success)
        {
            return false;
        }
        self.record_external_dma_write();
        self.gic.set_pending(VIRTIO_GPU_IRQ_ID);
        true
    }

    pub fn complete_gpu_3d_readback(&mut self, sequence: u32, format: u32, pixels: &[u8]) -> bool {
        if !self.virtio_gpu.complete_3d_readback(&mut self.mem, sequence, format, pixels) {
            return false;
        }
        self.record_external_dma_write();
        self.gic.set_pending(VIRTIO_GPU_IRQ_ID);
        true
    }

    pub fn complete_gpu_3d_resident(&mut self, sequence: u32) -> bool {
        if !self.virtio_gpu.complete_3d_resident(&mut self.mem, sequence) {
            return false;
        }
        self.record_external_dma_write();
        self.gic.set_pending(VIRTIO_GPU_IRQ_ID);
        true
    }
}
