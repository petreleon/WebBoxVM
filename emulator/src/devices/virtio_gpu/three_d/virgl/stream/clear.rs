use super::super::VirglContext;
use crate::devices::virtio_gpu::protocol::{
    RESP_ERR_INVALID_PARAMETER, RESP_ERR_INVALID_RESOURCE_ID, Rect,
};
use crate::devices::virtio_gpu::{Scanout, VirtioGpu};

pub(super) type Clear = (u32, [f32; 4], Rect);

impl VirtioGpu {
    pub(super) fn validate_virgl_clear(
        &self,
        resource: u32,
        color: [f32; 4],
        rect: Rect,
    ) -> Result<(), u32> {
        let Some(target) = self.resources.get(&resource) else {
            return Err(RESP_ERR_INVALID_RESOURCE_ID);
        };
        if !color
            .iter()
            .all(|value| value.is_finite() && (0.0..=1.0).contains(value))
        {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        if !target.is_texture_2d()
            || !rect.valid_within(target.width, target.height)
            || !matches_scanout(self.scanout, resource, rect)
        {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        Ok(())
    }

    pub(super) fn framebuffer_virgl_clear_target(
        &self,
        context: &VirglContext,
        color: [f32; 4],
    ) -> Result<(u32, Rect), u32> {
        let resource = context
            .framebuffer_resource()
            .ok_or(RESP_ERR_INVALID_PARAMETER)?;
        let rect = self
            .scanout
            .filter(|current| current.resource_id == resource)
            .map(|current| current.rect)
            .ok_or(RESP_ERR_INVALID_PARAMETER)?;
        self.validate_virgl_clear(resource, color, rect)?;
        Ok((resource, rect))
    }
}

pub(super) fn set(
    clear: &mut Option<Clear>,
    resource: u32,
    color: [f32; 4],
    rect: Rect,
) -> Result<(), u32> {
    clear
        .replace((resource, color, rect))
        .is_none()
        .then_some(())
        .ok_or(RESP_ERR_INVALID_PARAMETER)
}

fn matches_scanout(scanout: Option<Scanout>, resource: u32, rect: Rect) -> bool {
    scanout.is_some_and(|current| current.resource_id == resource && current.rect == rect)
}
