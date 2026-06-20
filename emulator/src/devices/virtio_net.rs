//! Minimal VirtIO-MMIO network adapter.
//!
//! The browser host owns the real transport. This device only exposes the
//! Linux-facing VirtIO-net queues and moves raw Ethernet frames across that
//! host boundary.

mod mmio;
mod queue;
#[cfg(test)]
mod tests;

use crate::memory::PhysicalMemory;
use std::collections::VecDeque;

pub(super) const VIRTIO_MMIO_MAGIC: u64 = 0x7472_6976;
pub(super) const VIRTIO_MMIO_VERSION_2: u64 = 2;
pub(super) const VIRTIO_DEVICE_ID_NET: u64 = 1;
pub(super) const VIRTIO_VENDOR_WEBBOXVM: u64 = 0x5742_564d;

pub(super) const VIRTIO_NET_F_MAC: u64 = 1 << 5;
pub(super) const VIRTIO_NET_F_STATUS: u64 = 1 << 16;
pub(super) const VIRTIO_F_VERSION_1: u64 = 1 << 32;

pub(super) const VIRTQ_DESC_F_NEXT: u16 = 1;
pub(super) const VIRTQ_DESC_F_WRITE: u16 = 2;

pub(super) const QUEUE_RX: usize = 0;
pub(super) const QUEUE_TX: usize = 1;
pub(super) const QUEUE_COUNT: usize = 2;
pub(super) const QUEUE_NUM_MAX: u16 = 64;
pub(super) const VIRTIO_NET_HDR_LEN: usize = 12;

#[derive(Clone, Copy, Debug)]
pub(super) struct Descriptor {
    addr: u64,
    len: u32,
    flags: u16,
    next: u16,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct VirtQueue {
    num: u16,
    ready: bool,
    desc: u64,
    driver: u64,
    device: u64,
    last_avail_idx: u16,
}

impl VirtQueue {
    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }
}

#[derive(Debug, Clone)]
pub struct VirtioNet {
    pub(in crate::devices::virtio_net) device_features_sel: u32,
    pub(in crate::devices::virtio_net) driver_features_sel: u32,
    pub(in crate::devices::virtio_net) queue_sel: u32,
    pub(in crate::devices::virtio_net) queues: [VirtQueue; QUEUE_COUNT],
    pub(in crate::devices::virtio_net) interrupt_status: u32,
    pub(in crate::devices::virtio_net) status: u32,
    mac: [u8; 6],
    rx_frames: VecDeque<Vec<u8>>,
    tx_frames: VecDeque<Vec<u8>>,
    rx_packets: u64,
    tx_packets: u64,
}

impl VirtioNet {
    pub fn new() -> Self {
        Self {
            device_features_sel: 0,
            driver_features_sel: 0,
            queue_sel: 0,
            queues: [VirtQueue::default(), VirtQueue::default()],
            interrupt_status: 0,
            status: 0,
            mac: [0x02, 0x57, 0x42, 0x56, 0x4d, 0x01],
            rx_frames: VecDeque::new(),
            tx_frames: VecDeque::new(),
            rx_packets: 0,
            tx_packets: 0,
        }
    }

    pub fn inject_rx_frame(&mut self, mem: &mut PhysicalMemory, frame: &[u8]) -> bool {
        if frame.is_empty() {
            return false;
        }
        self.rx_frames.push_back(frame.to_vec());
        self.process_rx(mem)
    }

    pub fn pop_tx_frame(&mut self) -> Option<Vec<u8>> {
        self.tx_frames.pop_front()
    }

    pub fn pending_tx_frames(&self) -> usize {
        self.tx_frames.len()
    }

    pub fn pending_rx_frames(&self) -> usize {
        self.rx_frames.len()
    }

    pub fn rx_packet_count(&self) -> u64 {
        self.rx_packets
    }

    pub fn tx_packet_count(&self) -> u64 {
        self.tx_packets
    }

    pub fn mac_address(&self) -> [u8; 6] {
        self.mac
    }

    pub(super) fn selected_queue(&self) -> Option<&VirtQueue> {
        self.queues.get(self.queue_sel as usize)
    }

    pub(super) fn selected_queue_mut(&mut self) -> Option<&mut VirtQueue> {
        self.queues.get_mut(self.queue_sel as usize)
    }

    pub(super) fn reset(&mut self) {
        for queue in &mut self.queues {
            queue.reset();
        }
        self.interrupt_status = 0;
        self.rx_frames.clear();
        self.tx_frames.clear();
    }
}

impl Default for VirtioNet {
    fn default() -> Self {
        Self::new()
    }
}
