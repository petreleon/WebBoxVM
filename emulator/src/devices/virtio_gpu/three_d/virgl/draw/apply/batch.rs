use super::super::{DrawMaterial, DrawWork, MAX_VIRGL_BATCH_DRAWS, raster};
use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::protocol::Rect;

pub(super) fn apply(
    gpu: &mut VirtioGpu,
    resource_id: u32,
    rect: Rect,
    clear: [u8; 4],
    works: Vec<DrawWork>,
) -> bool {
    if works.len() < 2 || works.len() > MAX_VIRGL_BATCH_DRAWS {
        return false;
    }
    {
        let Some(resource) = gpu.resources.get_mut(&resource_id) else { return false; };
        if resource.clear_bgra(rect, clear).is_none() { return false; }
        for work in works {
            let DrawMaterial::Solid(color) = work.material else { return false; };
            if work.depth_resource.is_some()
                || !raster::draw_solid(resource, rect, &work.vertices, color, work.viewport, work.scissor)
            {
                return false;
            }
        }
    }
    gpu.add_damage(resource_id, rect);
    true
}
