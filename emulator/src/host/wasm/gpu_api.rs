use super::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl Emulator {
    /// Consume the latest coalesced VirtIO-GPU scanout damage packet.
    ///
    /// The 32-byte little-endian header is `WBGF`, version, scanout dimensions,
    /// and damage rectangle, followed by tightly packed BGRA8 pixels.
    pub fn gpu_scanout_update(&mut self) -> Vec<u8> {
        let _access = self.require_parallel_idle();
        if let Some(ref mut boot) = self.boot {
            boot.machine.bus.virtio_gpu.take_scanout_update()
        } else {
            self.machine.bus.virtio_gpu.take_scanout_update()
        }
    }

    /// Consume the next private-capset WBG3 packet submitted by the guest.
    pub fn gpu_3d_update(&mut self) -> Vec<u8> {
        let _access = self.require_parallel_idle();
        if let Some(ref mut boot) = self.boot {
            boot.machine.bus.virtio_gpu.take_3d_update()
        } else {
            self.machine.bus.virtio_gpu.take_3d_update()
        }
    }

    /// Observe guest-initiated VirtIO-GPU resets so browser presentation can reset too.
    pub fn gpu_reset_generation(&self) -> u32 {
        let _access = self.require_parallel_idle();
        if let Some(ref boot) = self.boot {
            boot.machine.bus.virtio_gpu.reset_generation()
        } else {
            self.machine.bus.virtio_gpu.reset_generation()
        }
    }

    /// Complete a WBG3 submission after the browser GPU queue settles.
    pub fn gpu_3d_complete(&mut self, sequence: u32, success: bool) -> bool {
        let _access = self.require_parallel_idle();
        if let Some(ref mut boot) = self.boot {
            boot.machine.bus.complete_gpu_3d(sequence, success)
        } else {
            self.machine.bus.complete_gpu_3d(sequence, success)
        }
    }
}
