use super::VirglContext;
use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::protocol::*;
use crate::devices::virtio_gpu::resource::GpuResource;

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
        if !context.is_attached(copy.src_resource)
            || !context.is_attached(copy.dst_resource)
            || !self.is_virgl_resource(copy.src_resource)
            || !self.is_virgl_resource(copy.dst_resource)
            || !source.is_texture_2d()
            || !destination.is_texture_2d()
            || uses_scanout
        {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        let destination_rect = Rect {
            x: copy.dst_x,
            y: copy.dst_y,
            width: copy.src_rect.width,
            height: copy.src_rect.height,
        };
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
        Ok(())
    }

    pub(super) fn apply_virgl_copy(&mut self, copy: CopyRegion) -> Result<(), u32> {
        let pixels = source_pixels(
            self.resources
                .get(&copy.src_resource)
                .expect("copy source validated before application"),
            copy.src_rect,
        )
        .ok_or(RESP_ERR_INVALID_PARAMETER)?;
        let destination_rect = Rect {
            x: copy.dst_x,
            y: copy.dst_y,
            width: copy.src_rect.width,
            height: copy.src_rect.height,
        };
        write_pixels(
            self.resources
                .get_mut(&copy.dst_resource)
                .expect("copy destination validated before application"),
            destination_rect,
            &pixels,
        )
        .ok_or(RESP_ERR_INVALID_PARAMETER)
    }
}

fn source_pixels(resource: &GpuResource, rect: Rect) -> Option<Vec<u8>> {
    let row_len = usize::try_from(rect.width).ok()?.checked_mul(4)?;
    let rows = usize::try_from(rect.height).ok()?;
    let mut pixels = Vec::with_capacity(row_len.checked_mul(rows)?);
    for y in rect.y..rect.y.checked_add(rect.height)? {
        let start = pixel_offset(resource, rect.x, y)?;
        pixels.extend_from_slice(resource.pixels.get(start..start.checked_add(row_len)?)?);
    }
    Some(pixels)
}

fn write_pixels(resource: &mut GpuResource, rect: Rect, pixels: &[u8]) -> Option<()> {
    let row_len = usize::try_from(rect.width).ok()?.checked_mul(4)?;
    if pixels.len() != row_len.checked_mul(usize::try_from(rect.height).ok()?)? {
        return None;
    }
    for (row, y) in (rect.y..rect.y.checked_add(rect.height)?).enumerate() {
        let start = pixel_offset(resource, rect.x, y)?;
        let source = row.checked_mul(row_len)?;
        let end = source.checked_add(row_len)?;
        resource
            .pixels
            .get_mut(start..start.checked_add(row_len)?)?
            .copy_from_slice(&pixels[source..end]);
    }
    Some(())
}

fn pixel_offset(resource: &GpuResource, x: u32, y: u32) -> Option<usize> {
    usize::try_from(y)
        .ok()?
        .checked_mul(usize::try_from(resource.width).ok()?)?
        .checked_add(usize::try_from(x).ok()?)?
        .checked_mul(4)
}
