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
            if work.depth_resource.is_some() || work.depth_compare.is_some()
                || !raster::draw_solid(resource, rect, &work.vertices, color, work.viewport, work.scissor)
            {
                return false;
            }
        }
    }
    gpu.add_damage(resource_id, rect);
    true
}

pub(super) fn apply_depth(
    gpu: &mut VirtioGpu,
    resource_id: u32,
    depth_resource: u32,
    rect: Rect,
    clear: [u8; 4],
    works: Vec<DrawWork>,
) -> bool {
    if works.len() < 2 || works.len() > MAX_VIRGL_BATCH_DRAWS { return false; }
    let Some(mut values) = gpu.depth_values(resource_id, depth_resource) else { return false; };
    let drawn = {
        let Some(resource) = gpu.resources.get_mut(&resource_id) else { return false; };
        if resource.clear_bgra(rect, clear).is_none() { return false; }
        works.into_iter().all(|work| {
            let DrawMaterial::Solid(color) = work.material else { return false; };
            let Some(compare) = work.depth_compare else { return false; };
            work.depth_resource == Some(depth_resource)
                && raster::draw_depth_solid(
                    resource, rect, &work.vertices, color, work.viewport, work.scissor,
                    compare, &mut values,
                )
        })
    };
    if !drawn || !gpu.store_depth(Some((depth_resource, values))) { return false; }
    gpu.add_damage(resource_id, rect);
    true
}
