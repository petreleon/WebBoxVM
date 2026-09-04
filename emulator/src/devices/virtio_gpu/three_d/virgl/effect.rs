use super::super::Pending3dEffect;
use super::BlendMode;
use crate::devices::virtio_gpu::VirtioGpu;

mod readback;

impl VirtioGpu {
    pub(in crate::devices::virtio_gpu) fn apply_3d_effect(
        &mut self,
        effect: Pending3dEffect,
    ) -> bool {
        let (context_id, generation) = effect_context(&effect);
        if self
            .virgl_contexts
            .get(&context_id)
            .map(|context| context.generation)
            != Some(generation)
        {
            return false;
        }
        match effect {
            Pending3dEffect::VirglClear {
                resource_id,
                rect,
                bgra,
                ..
            } => {
                let cleared = self
                    .resources
                    .get_mut(&resource_id)
                    .is_some_and(|resource| resource.clear_bgra(rect, bgra).is_some());
                if cleared {
                    self.add_damage(resource_id, rect);
                }
                cleared
            }
            Pending3dEffect::VirglDraw {
                resource_id,
                depth_resource,
                depth_state,
                rect,
                clear_bgra,
                material,
                vertices,
                viewport,
                scissor,
                ..
            } => self.apply_virgl_draw(
                resource_id,
                depth_resource,
                depth_state,
                rect,
                clear_bgra,
                material,
                &vertices,
                viewport,
                scissor,
            ),
            Pending3dEffect::VirglBatch {
                resource_id,
                rect,
                clear_bgra,
                works,
                ..
            } => self.apply_virgl_batch(resource_id, rect, clear_bgra, works),
            Pending3dEffect::VirglDepthBatch {
                resource_id,
                depth_resource,
                rect,
                clear_bgra,
                works,
                ..
            } if works.iter().all(|work| work.blend == BlendMode::SourceOver) => {
                self.apply_virgl_depth_batch(resource_id, depth_resource, rect, clear_bgra, works)
            }
            Pending3dEffect::VirglDepthBatch { .. } => false,
            Pending3dEffect::VirglResidentReadback { .. } => false,
            Pending3dEffect::VirglResidentCopy { .. } => false,
        }
    }

}

fn effect_context(effect: &Pending3dEffect) -> (u32, u32) {
    match effect {
        Pending3dEffect::VirglClear {
            context_id,
            generation,
            ..
        }
        | Pending3dEffect::VirglDraw {
            context_id,
            generation,
            ..
        }
        | Pending3dEffect::VirglBatch {
            context_id,
            generation,
            ..
        }
        | Pending3dEffect::VirglDepthBatch {
            context_id,
            generation,
            ..
        }
        | Pending3dEffect::VirglResidentReadback {
            context_id,
            generation,
            ..
        }
        | Pending3dEffect::VirglResidentCopy {
            context_id,
            generation,
            ..
        } => (*context_id, *generation),
    }
}
