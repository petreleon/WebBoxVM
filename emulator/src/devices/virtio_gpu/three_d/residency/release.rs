use super::{MAX_RESIDENT_RELEASES, release_packet};
use crate::devices::virtio_gpu::fence::FenceTimeline;
use crate::devices::virtio_gpu::three_d::{BrowserCompletion, Pending3d};
use crate::devices::virtio_gpu::{VirtioGpu, MAX_PENDING_3D_BYTES, MAX_PENDING_3D_SUBMITS};

impl VirtioGpu {
    pub(in crate::devices::virtio_gpu) fn queue_completed_resident_release(
        &mut self, sequence: u32, timeline: FenceTimeline,
    ) {
        if self.resident_releases.len() < MAX_RESIDENT_RELEASES {
            self.resident_releases.push_back(sequence);
            return;
        }
        // The acknowledged job was removed first. Reuse its slot for lossless release
        // transport: new submissions backpressure until polling delivers this packet.
        let packet = release_packet(sequence);
        debug_assert!(self.pending_3d.len() < MAX_PENDING_3D_SUBMITS);
        debug_assert!(self.pending_3d_bytes + packet.len() <= MAX_PENDING_3D_BYTES);
        self.pending_3d_bytes += packet.len();
        self.pending_3d.push(Pending3d {
            sequence, timeline, bytes: packet.len(), packet: Some(packet), completion: None,
            effect: None, browser_completion: BrowserCompletion::ResidentRelease,
        });
    }
}
