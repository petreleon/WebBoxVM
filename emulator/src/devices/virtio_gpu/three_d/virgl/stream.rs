mod decode;

use super::VirglContext;
use crate::devices::virtio_gpu::protocol::*;
use crate::devices::virtio_gpu::{Scanout, VirtioGpu};
use decode::{Command, decode_stream};

impl VirtioGpu {
    pub(in crate::devices::virtio_gpu) fn submit_virgl(
        &mut self,
        header: CtrlHeader,
        input: &[u8],
    ) -> Result<Option<super::DeferredSubmit>, u32> {
        let commands = decode_stream(input).ok_or(RESP_ERR_INVALID_PARAMETER)?;
        let mut context = self
            .virgl_contexts
            .get(&header.ctx_id)
            .cloned()
            .ok_or(RESP_ERR_INVALID_CONTEXT_ID)?;
        let mut clear = None;
        for command in commands {
            match command {
                Command::Nop => {}
                Command::CreateSurface {
                    handle,
                    resource,
                    format,
                    level,
                    layers,
                } => {
                    self.create_surface(&mut context, handle, resource, format, level, layers)?;
                }
                Command::DestroySurface { handle } => {
                    context
                        .surfaces
                        .remove(&handle)
                        .ok_or(RESP_ERR_INVALID_PARAMETER)?;
                }
                Command::ClearSurface {
                    handle,
                    color,
                    rect,
                } => {
                    if clear.is_some() {
                        return Err(RESP_ERR_INVALID_PARAMETER);
                    }
                    clear = Some((
                        self.validate_clear(&context, handle, color, rect)?,
                        color,
                        rect,
                    ));
                }
            }
        }
        let deferred = match clear {
            Some((resource, color, rect)) => {
                Some(self.queue_virgl_clear(header, context.generation, resource, rect, color)?)
            }
            None => None,
        };
        self.virgl_contexts.insert(header.ctx_id, context);
        Ok(deferred)
    }

    fn create_surface(
        &self,
        context: &mut VirglContext,
        handle: u32,
        resource: u32,
        format: u32,
        level: u32,
        layers: u32,
    ) -> Result<(), u32> {
        if handle == 0 || context.surfaces.contains_key(&handle) || level != 0 || layers != 0 {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        let Some(target) = self.resources.get(&resource) else {
            return Err(RESP_ERR_INVALID_RESOURCE_ID);
        };
        if !context.attached.contains(&resource)
            || !self.is_virgl_resource(resource)
            || target.format != format
        {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        context.surfaces.insert(handle, resource);
        Ok(())
    }

    fn validate_clear(
        &self,
        context: &VirglContext,
        handle: u32,
        color: [f32; 4],
        rect: Rect,
    ) -> Result<u32, u32> {
        let resource = *context
            .surfaces
            .get(&handle)
            .ok_or(RESP_ERR_INVALID_PARAMETER)?;
        let Some(target) = self.resources.get(&resource) else {
            return Err(RESP_ERR_INVALID_RESOURCE_ID);
        };
        if !color
            .iter()
            .all(|value| value.is_finite() && (0.0..=1.0).contains(value))
        {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        if !rect.valid_within(target.width, target.height)
            || !matches_scanout(self.scanout, resource, rect)
        {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        Ok(resource)
    }
}

fn matches_scanout(scanout: Option<Scanout>, resource: u32, rect: Rect) -> bool {
    scanout.is_some_and(|current| current.resource_id == resource && current.rect == rect)
}
