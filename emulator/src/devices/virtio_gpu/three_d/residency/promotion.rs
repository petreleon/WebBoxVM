use super::ResidentResource;
use super::super::Pending3dEffect;
use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::protocol::Rect;

impl VirtioGpu {
    pub(in crate::devices::virtio_gpu) fn resident_candidate(
        &self,
        packet: &[u8],
        effect: &Pending3dEffect,
    ) -> bool {
        let supported = match effect {
            Pending3dEffect::VirglClear { .. } => version(packet, b"VGC1") == Some(2),
            Pending3dEffect::VirglBatch { .. } => matches!(version(packet, b"VGB1"), Some(6 | 7 | 14 | 15))
                || matches!(version(packet, b"VGM1"), Some(2 | 3 | 10 | 11)),
            _ => false,
        };
        supported && self.resident_effect_valid(effect)
    }

    pub(in crate::devices::virtio_gpu) fn promote_resident(
        &mut self,
        sequence: u32,
        effect: Pending3dEffect,
    ) -> bool {
        if !self.resident_effect_valid(&effect) { return false; }
        let Some((context_id, generation, resource_id, ..)) = target(&effect) else { return false; };
        self.resident_resources.insert(resource_id, ResidentResource {
            context_id, generation, producer_sequence: sequence,
        });
        if self.scanout.is_some_and(|scanout| scanout.resource_id == resource_id) {
            self.pending_damage = None;
        }
        true
    }

    fn resident_effect_valid(&self, effect: &Pending3dEffect) -> bool {
        let Some((context_id, generation, resource_id, rect, epoch, predecessor)) = target(effect) else {
            return false;
        };
        self.virgl_contexts.get(&context_id).is_some_and(|context| context.generation == generation)
            && self.resident_target_eligible(resource_id, rect)
            && self.resident_epoch == epoch
            && self.resident_resources.get(&resource_id).map(|resident| resident.producer_sequence) == predecessor
    }
}

fn target(effect: &Pending3dEffect) -> Option<(u32, u32, u32, Rect, u64, Option<u32>)> {
    match effect {
        Pending3dEffect::VirglClear {
            context_id, generation, resource_id, rect, resident_epoch, resident_predecessor, ..
        }
        | Pending3dEffect::VirglBatch {
            context_id, generation, resource_id, rect, resident_epoch, resident_predecessor, ..
        } => Some((*context_id, *generation, *resource_id, *rect, *resident_epoch, *resident_predecessor)),
        _ => None,
    }
}

fn version(packet: &[u8], magic: &[u8; 4]) -> Option<u32> {
    let bytes: [u8; 4] = packet.get(4..8)?.try_into().ok()?;
    (packet.get(..4) == Some(magic)).then_some(u32::from_le_bytes(bytes))
}
