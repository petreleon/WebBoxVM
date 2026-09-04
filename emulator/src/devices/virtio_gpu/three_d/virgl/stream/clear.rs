use super::super::VirglContext;
use crate::devices::virtio_gpu::protocol::{
    RESP_ERR_INVALID_PARAMETER, RESP_ERR_INVALID_RESOURCE_ID, Rect,
};
use crate::devices::virtio_gpu::{Scanout, VirtioGpu};

#[derive(Clone, Copy)]
pub(super) struct Clear {
    pub(super) resource: u32,
    pub(super) depth_resource: Option<u32>,
    pub(super) color: [f32; 4],
    pub(super) rect: Rect,
}

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
        depth: bool,
    ) -> Result<Clear, u32> {
        let resource = context
            .framebuffer_resource()
            .ok_or(RESP_ERR_INVALID_PARAMETER)?;
        let rect = self
            .scanout
            .filter(|current| current.resource_id == resource)
            .map(|current| current.rect)
            .ok_or(RESP_ERR_INVALID_PARAMETER)?;
        self.validate_virgl_clear(resource, color, rect)?;
        let depth_resource = context.framebuffer_depth_resource();
        if depth != depth_resource.is_some() {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        if let Some(depth_resource) = depth_resource {
            let depth = self.resources.get(&depth_resource).ok_or(RESP_ERR_INVALID_RESOURCE_ID)?;
            if !depth.is_depth_texture_2d()
                || depth.width != rect.width
                || depth.height != rect.height
                || depth_resource == resource
            {
                return Err(RESP_ERR_INVALID_PARAMETER);
            }
        }
        Ok(Clear { resource, depth_resource, color, rect })
    }
}

pub(super) fn set(clear: &mut Option<Clear>, target: Clear) -> Result<(), u32> {
    clear
        .replace(target)
        .is_none()
        .then_some(())
        .ok_or(RESP_ERR_INVALID_PARAMETER)
}

fn matches_scanout(scanout: Option<Scanout>, resource: u32, rect: Rect) -> bool {
    scanout.is_some_and(|current| current.resource_id == resource && current.rect == rect)
}
