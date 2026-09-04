use super::VirglContext;
use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::protocol::{RESP_ERR_INVALID_PARAMETER, RESP_ERR_INVALID_RESOURCE_ID};
use crate::devices::virtio_gpu::resource::{BufferBind, GpuResource};

#[derive(Clone)]
pub(super) struct InlineWrite {
    pub(super) resource: u32,
    pub(super) offset: u32,
    pub(super) bytes: Vec<u8>,
}

impl VirtioGpu {
    pub(super) fn validate_virgl_inline_write(
        &self,
        context: &VirglContext,
        write: &InlineWrite,
    ) -> Result<(), u32> {
        let resource = self
            .resources
            .get(&write.resource)
            .ok_or(RESP_ERR_INVALID_RESOURCE_ID)?;
        if !context.is_attached(write.resource) || !self.is_virgl_resource(write.resource) {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        range(resource, write.offset, write.bytes.len()).ok_or(RESP_ERR_INVALID_PARAMETER)?;
        Ok(())
    }

    pub(super) fn apply_virgl_inline_write(&mut self, write: InlineWrite) -> Result<(), u32> {
        let resource = self
            .resources
            .get_mut(&write.resource)
            .ok_or(RESP_ERR_INVALID_RESOURCE_ID)?;
        let range = range(resource, write.offset, write.bytes.len())
            .ok_or(RESP_ERR_INVALID_PARAMETER)?;
        resource.pixels[range].copy_from_slice(&write.bytes);
        Ok(())
    }
}

fn range(resource: &GpuResource, offset: u32, length: usize) -> Option<std::ops::Range<usize>> {
    if !resource.is_buffer_bind(BufferBind::Uniform) || length == 0 {
        return None;
    }
    let start = usize::try_from(offset).ok()?;
    start
        .checked_add(length)
        .filter(|end| *end <= resource.pixels.len())
        .map(|end| start..end)
}
