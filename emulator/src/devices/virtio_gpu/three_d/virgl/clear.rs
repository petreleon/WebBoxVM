use super::super::{DeferredSubmit, Pending3dEffect};
use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::protocol::{CtrlHeader, Rect};

impl VirtioGpu {
    pub(in crate::devices::virtio_gpu::three_d::virgl) fn queue_virgl_clear(
        &mut self,
        header: CtrlHeader,
        generation: u32,
        resource_id: u32,
        rect: Rect,
        color: [f32; 4],
    ) -> Result<DeferredSubmit, u32> {
        let sequence = self.virgl_sequence()?;
        let resident = self.resident_target_eligible(resource_id, rect);
        let resident_epoch = self.resident_epoch;
        let resident_predecessor = resident.then(|| self.resident_resources.get(&resource_id)
            .map(|resource| resource.producer_sequence)).flatten();
        self.queue_virgl_packet(
            header,
            sequence,
            packet(sequence, rect.width, rect.height, color, resident, resident_predecessor),
            Pending3dEffect::VirglClear {
                context_id: header.ctx_id, generation, resource_id, rect, resident_epoch,
                resident_predecessor, bgra: super::bgra(color),
            },
        )
    }
}

fn packet(
    sequence: u32,
    width: u32,
    height: u32,
    color: [f32; 4],
    resident: bool,
    predecessor: Option<u32>,
) -> Vec<u8> {
    let mut packet = b"VGC1".to_vec();
    for value in [if resident { 2 } else { 1 }, sequence, width, height] {
        packet.extend_from_slice(&value.to_le_bytes());
    }
    for value in color { packet.extend_from_slice(&value.to_le_bytes()); }
    if resident { packet.extend_from_slice(&predecessor.unwrap_or_default().to_le_bytes()); }
    packet
}
