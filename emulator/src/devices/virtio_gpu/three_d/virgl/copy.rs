use super::VirglContext;
use super::copy_buffer;
use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::protocol::*;
use crate::devices::virtio_gpu::resource::GpuResource;

mod apply;
use apply::{source_pixels, write_pixels};

#[derive(Clone, Copy)]
pub(in crate::devices::virtio_gpu) struct CopyRegion {
    pub dst_resource: u32,
    pub dst_level: u32,
    pub dst_x: u32,
    pub dst_y: u32,
    pub dst_z: u32,
    pub src_resource: u32,
    pub src_level: u32,
    pub src_rect: Rect,
    pub src_z: u32,
    pub depth: u32,
}

impl VirtioGpu {
    pub(super) fn validate_virgl_copy(
        &self,
        context_id: u32,
        context: &VirglContext,
        copy: CopyRegion,
    ) -> Result<(), u32> {
        let source = self
            .resources
            .get(&copy.src_resource)
            .ok_or(RESP_ERR_INVALID_RESOURCE_ID)?;
        let destination = self
            .resources
            .get(&copy.dst_resource)
            .ok_or(RESP_ERR_INVALID_RESOURCE_ID)?;
        let uses_scanout = self.scanout.is_some_and(|scanout| {
            scanout.resource_id == copy.src_resource || scanout.resource_id == copy.dst_resource
        });
        let destination_rect = Rect {
            x: copy.dst_x, y: copy.dst_y, width: copy.src_rect.width, height: copy.src_rect.height,
        };
        if !context.is_attached(copy.src_resource)
            || !context.is_attached(copy.dst_resource)
            || !self.is_virgl_resource(copy.src_resource)
            || !self.is_virgl_resource(copy.dst_resource)
            || self.resident_copy_in_flight(copy.src_resource)
            || self.resident_copy_in_flight(copy.dst_resource)
            || (!self.resident_resources.contains_key(&copy.src_resource)
                && !self.resident_overwrite_allowed(copy.dst_resource, destination_rect))
            || uses_scanout
        {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        if source.is_buffer() || destination.is_buffer() {
            return copy_buffer::valid(source, destination, copy)
                .then_some(())
                .ok_or(RESP_ERR_INVALID_PARAMETER);
        }
        if !source.is_texture_2d() || !destination.is_texture_2d() {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        if copy.dst_level != 0
            || copy.src_level != 0
            || copy.dst_z != 0
            || copy.src_z != 0
            || copy.depth != 1
            || source.format != destination.format
            || !copy.src_rect.valid_within(source.width, source.height)
            || !destination_rect.valid_within(destination.width, destination.height)
        {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        if self.resident_resources.contains_key(&copy.src_resource)
            && !self.resident_copy_eligible(context_id, context.generation, copy, destination_rect)
        {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        Ok(())
    }

    pub(super) fn queue_virgl_copy(
        &mut self,
        header: CtrlHeader,
        generation: u32,
        copy: CopyRegion,
    ) -> Result<Option<super::super::DeferredSubmit>, u32> {
        if self.resident_resources.contains_key(&copy.src_resource) {
            return self.queue_resident_copy(header, generation, copy).map(Some);
        }
        self.apply_virgl_copy(copy)?;
        Ok(None)
    }

    pub(super) fn apply_virgl_copy(&mut self, copy: CopyRegion) -> Result<(), u32> {
        if self.resident_resources.contains_key(&copy.src_resource) {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        if self
            .resources
            .get(&copy.src_resource)
            .is_some_and(GpuResource::is_buffer)
        {
            let bytes = copy_buffer::source_bytes(
                self.resources
                    .get(&copy.src_resource)
                    .expect("copy source validated before application"),
                copy,
            )
            .ok_or(RESP_ERR_INVALID_PARAMETER)?;
            let result = copy_buffer::write_bytes(
                self.resources
                    .get_mut(&copy.dst_resource)
                    .expect("copy destination validated before application"),
                copy,
                &bytes,
            )
            .ok_or(RESP_ERR_INVALID_PARAMETER);
            if result.is_ok() {
                self.forget_resident(copy.dst_resource);
            }
            return result;
        }
        let pixels = source_pixels(
            self.resources
                .get(&copy.src_resource)
                .expect("copy source validated before application"),
            copy.src_rect,
        )
        .ok_or(RESP_ERR_INVALID_PARAMETER)?;
        let destination_rect = Rect { x: copy.dst_x, y: copy.dst_y, width: copy.src_rect.width, height: copy.src_rect.height };
        let result = write_pixels(
            self.resources
                .get_mut(&copy.dst_resource)
                .expect("copy destination validated before application"),
            destination_rect,
            &pixels,
        )
        .ok_or(RESP_ERR_INVALID_PARAMETER);
        if result.is_ok() {
            self.forget_resident(copy.dst_resource);
        }
        result
    }
}
