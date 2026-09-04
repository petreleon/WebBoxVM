use super::effect_context;
use super::super::DrawWork;
use super::super::super::Pending3dEffect;
use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::protocol::Rect;

impl VirtioGpu {
    pub(in crate::devices::virtio_gpu) fn apply_3d_readback(
        &mut self,
        effect: Pending3dEffect,
        format: u32,
        pixels: &[u8],
    ) -> bool {
        let (context_id, generation) = effect_context(&effect);
        if self.virgl_contexts.get(&context_id).map(|context| context.generation) != Some(generation) {
            return false;
        }
        match effect {
            Pending3dEffect::VirglBatch { resource_id, rect, .. } => {
                self.write_gpu_readback(resource_id, rect, format, pixels)
            }
            Pending3dEffect::VirglDepthBatch {
                resource_id, depth_resource, rect, clear_bgra, works, ..
            } => self.apply_depth_readback(resource_id, depth_resource, rect, clear_bgra, works, format, pixels),
            _ => false,
        }
    }

    fn apply_depth_readback(
        &mut self,
        resource_id: u32,
        depth_resource: u32,
        rect: Rect,
        clear: [u8; 4],
        works: Vec<DrawWork>,
        format: u32,
        pixels: &[u8],
    ) -> bool {
        if !self.valid_gpu_readback(resource_id, rect, format, pixels) {
            return false;
        }
        let Some(color_before) = self.resources.get(&resource_id).map(|resource| resource.pixels.clone()) else { return false; };
        let Some(depth_before) = self.resources.get(&depth_resource).map(|resource| resource.pixels.clone()) else { return false; };
        if self.apply_virgl_depth_batch(resource_id, depth_resource, rect, clear, works)
            && self.write_gpu_readback(resource_id, rect, format, pixels)
        {
            return true;
        }
        self.restore_readback_pixels(resource_id, color_before);
        self.restore_readback_pixels(depth_resource, depth_before);
        false
    }

    pub(in crate::devices::virtio_gpu) fn write_gpu_readback(&mut self, resource_id: u32, rect: Rect, format: u32, pixels: &[u8]) -> bool {
        if !self.valid_gpu_readback(resource_id, rect, format, pixels) { return false; }
        let row_bytes = rect.width as usize * 4;
        let Some(resource) = self.resources.get_mut(&resource_id) else { return false; };
        for (row, source) in pixels.chunks_exact(row_bytes).enumerate() {
            let start = ((rect.y as usize + row) * resource.width as usize + rect.x as usize) * 4;
            let destination = &mut resource.pixels[start..start + row_bytes];
            if format == 1 { destination.copy_from_slice(source); } else {
                for (destination, source) in destination.chunks_exact_mut(4).zip(source.chunks_exact(4)) {
                    destination.copy_from_slice(&[source[2], source[1], source[0], source[3]]);
                }
            }
        }
        self.add_damage(resource_id, rect);
        true
    }

    fn valid_gpu_readback(&self, resource_id: u32, rect: Rect, format: u32, pixels: &[u8]) -> bool {
        let bytes = usize::try_from(rect.width).ok().and_then(|width| width.checked_mul(rect.height as usize)).and_then(|pixels| pixels.checked_mul(4));
        matches!(format, 1 | 2) && bytes == Some(pixels.len()) && self.resources.get(&resource_id)
            .is_some_and(|resource| resource.is_texture_2d() && rect.valid_within(resource.width, resource.height))
    }

    fn restore_readback_pixels(&mut self, resource_id: u32, pixels: Vec<u8>) {
        if let Some(resource) = self.resources.get_mut(&resource_id) {
            if resource.pixels.len() == pixels.len() { resource.pixels = pixels; }
        }
    }
}
