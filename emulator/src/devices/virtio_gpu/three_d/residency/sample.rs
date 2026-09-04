use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::protocol::Rect;
use crate::devices::virtio_gpu::three_d::Pending3dEffect;
use crate::devices::virtio_gpu::three_d::virgl::DrawWork;

impl VirtioGpu {
    pub(in crate::devices::virtio_gpu) fn resident_sample_in_flight(
        &self,
        resource_id: u32,
    ) -> bool {
        self.pending_3d.iter().any(|pending| matches!(pending.effect.as_ref(),
            Some(Pending3dEffect::VirglBatch { works, .. })
                if works.iter().any(|work| work.resident_texture().is_some_and(|source| source.resource_id == resource_id))))
    }

    pub(in crate::devices::virtio_gpu) fn resident_sample_eligible(
        &self,
        context_id: u32,
        generation: u32,
        resource_id: u32,
        rect: Rect,
        works: &[DrawWork],
    ) -> bool {
        let [work] = works else {
            return false;
        };
        let Some(source) = work.resident_sample_source() else {
            return false;
        };
        let owner = self.resident_resources.get(&source.resource_id);
        source.resource_id != resource_id
            && !self.resident_resource_in_flight(source.resource_id)
            && !self.resident_resources.contains_key(&resource_id)
            && self.resident_target_eligible(resource_id, rect)
            && owner.is_some_and(|owner| {
                owner.context_id == context_id
                    && owner.generation == generation
                    && owner.producer_sequence == source.producer_sequence
            })
            && self.virgl_contexts.get(&context_id).is_some_and(|context| {
                context.generation == generation
                    && context.is_attached(source.resource_id)
                    && context.is_attached(resource_id)
            })
            && self
                .resources
                .get(&source.resource_id)
                .is_some_and(|resource| {
                    resource.is_texture_2d()
                        && resource.is_sampled()
                        && resource.width == source.width
                        && resource.height == source.height
                })
            && !self
                .scanout
                .is_some_and(|scanout| scanout.resource_id == source.resource_id)
    }

    pub(in crate::devices::virtio_gpu) fn resident_sample_effect_valid(
        &self,
        effect: &Pending3dEffect,
    ) -> bool {
        let Pending3dEffect::VirglBatch {
            context_id,
            generation,
            resource_id,
            rect,
            works,
            ..
        } = effect
        else {
            return false;
        };
        self.resident_sample_eligible(*context_id, *generation, *resource_id, *rect, works)
    }
}
