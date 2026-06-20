use super::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl Emulator {
    pub fn inject_network_frame(&mut self, frame: Vec<u8>) -> bool {
        if let Some(ref mut boot) = self.boot {
            boot.machine.bus.inject_network_frame(&frame);
            true
        } else {
            self.machine.bus.inject_network_frame(&frame);
            true
        }
    }

    pub fn network_tx_frame(&mut self) -> Vec<u8> {
        if let Some(ref mut boot) = self.boot {
            boot.machine
                .bus
                .virtio_net
                .pop_tx_frame()
                .unwrap_or_default()
        } else {
            self.machine
                .bus
                .virtio_net
                .pop_tx_frame()
                .unwrap_or_default()
        }
    }

    pub fn network_tx_pending(&self) -> usize {
        if let Some(ref boot) = self.boot {
            boot.machine.bus.virtio_net.pending_tx_frames()
        } else {
            self.machine.bus.virtio_net.pending_tx_frames()
        }
    }

    pub fn network_rx_pending(&self) -> usize {
        if let Some(ref boot) = self.boot {
            boot.machine.bus.virtio_net.pending_rx_frames()
        } else {
            self.machine.bus.virtio_net.pending_rx_frames()
        }
    }

    pub fn network_rx_packets(&self) -> u64 {
        if let Some(ref boot) = self.boot {
            boot.machine.bus.virtio_net.rx_packet_count()
        } else {
            self.machine.bus.virtio_net.rx_packet_count()
        }
    }

    pub fn network_tx_packets(&self) -> u64 {
        if let Some(ref boot) = self.boot {
            boot.machine.bus.virtio_net.tx_packet_count()
        } else {
            self.machine.bus.virtio_net.tx_packet_count()
        }
    }
}
