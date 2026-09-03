//! Host-neutral VirtIO-GPU 2D device with bounded 3D transport profiles.
//!
//! The device implements the Linux-facing control queue and retains normalized
//! BGRA8 host resources. Browser presentation is deliberately outside this
//! module; hosts consume coalesced scanout updates through `take_scanout_update`.

mod backing;
mod commands;
mod completion;
mod frame;
mod mmio;
mod protocol;
mod queue;
mod resource;
mod resource_transfer;
mod three_d;

#[cfg(test)]
mod tests;

use protocol::{BackingEntry, Rect};
use resource::GpuResource;
use std::collections::HashMap;
use std::collections::HashSet;
use three_d::{Pending3d, VirglContext};

pub const SCANOUT_WIDTH: u32 = 1024;
pub const SCANOUT_HEIGHT: u32 = 768;

pub(super) const VIRTIO_MMIO_MAGIC: u64 = 0x7472_6976;
pub(super) const VIRTIO_MMIO_VERSION_2: u64 = 2;
pub(super) const VIRTIO_DEVICE_ID_GPU: u64 = 16;
pub(super) const VIRTIO_VENDOR_WEBBOXVM: u64 = 0x5742_564d;
pub(super) const VIRTIO_F_VERSION_1: u64 = 1 << 32;
pub(super) const VIRTIO_GPU_F_VIRGL: u64 = 1 << 0;
pub(super) const VIRTIO_GPU_F_CONTEXT_INIT: u64 = 1 << 4;
pub(super) const QUEUE_COUNT: usize = 2;
pub(super) const QUEUE_NUM_MAX: u16 = 256;
pub(super) const MAX_RESOURCES: usize = 64;
pub(super) const MAX_RESOURCE_BYTES: usize = 64 * 1024 * 1024;
pub(super) const MAX_TOTAL_RESOURCE_BYTES: usize = 128 * 1024 * 1024;
pub(super) const MAX_BACKING_ENTRIES: usize = MAX_RESOURCE_BYTES / 4096;
pub(super) const MAX_CONTEXTS: usize = 64;
pub(super) const MAX_PENDING_3D_SUBMITS: usize = 16;
pub(super) const MAX_PENDING_3D_BYTES: usize = 2 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct VirtQueue {
    num: u16,
    ready: bool,
    desc: u64,
    driver: u64,
    device: u64,
    last_avail_idx: u16,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Scanout {
    resource_id: u32,
    rect: Rect,
}

#[derive(Debug, Clone)]
pub struct VirtioGpu {
    device_features_sel: u32,
    driver_features_sel: u32,
    queue_sel: u32,
    queues: [VirtQueue; QUEUE_COUNT],
    interrupt_status: u32,
    status: u32,
    resources: HashMap<u32, GpuResource>,
    allocated_resource_bytes: usize,
    scanout: Option<Scanout>,
    pending_damage: Option<Rect>,
    contexts: HashMap<u32, u32>,
    virgl_contexts: HashMap<u32, VirglContext>,
    virgl_resources: HashSet<u32>,
    pending_3d: Vec<Pending3d>,
    pending_3d_bytes: usize,
    next_3d_sequence: u32,
    next_virgl_context_generation: u32,
    reset_generation: u32,
}

impl VirtioGpu {
    pub fn new() -> Self {
        Self {
            device_features_sel: 0,
            driver_features_sel: 0,
            queue_sel: 0,
            queues: [VirtQueue::default(); QUEUE_COUNT],
            interrupt_status: 0,
            status: 0,
            resources: HashMap::new(),
            allocated_resource_bytes: 0,
            scanout: None,
            pending_damage: None,
            contexts: HashMap::new(),
            virgl_contexts: HashMap::new(),
            virgl_resources: HashSet::new(),
            pending_3d: Vec::new(),
            pending_3d_bytes: 0,
            next_3d_sequence: 1,
            next_virgl_context_generation: 1,
            reset_generation: 0,
        }
    }

    pub fn cold_reset(&mut self) {
        let next_3d_sequence = self.next_3d_sequence;
        let reset_generation = self.reset_generation.wrapping_add(1);
        *self = Self::new();
        self.next_3d_sequence = next_3d_sequence;
        self.reset_generation = reset_generation;
    }

    pub fn reset_generation(&self) -> u32 {
        self.reset_generation
    }

    pub fn take_scanout_update(&mut self) -> Vec<u8> {
        self.encode_pending_scanout()
    }

    fn selected_queue(&self) -> Option<&VirtQueue> {
        self.queues.get(self.queue_sel as usize)
    }

    fn selected_queue_mut(&mut self) -> Option<&mut VirtQueue> {
        self.queues.get_mut(self.queue_sel as usize)
    }

    fn detach_scanout_resource(&mut self, resource_id: u32) {
        if self
            .scanout
            .is_some_and(|scanout| scanout.resource_id == resource_id)
        {
            self.scanout = None;
            self.pending_damage = None;
        }
    }
}

impl Default for VirtioGpu {
    fn default() -> Self {
        Self::new()
    }
}
