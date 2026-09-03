mod decode;
mod shader;
mod vertex;

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
        let mut copy = None;
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
                    if !context.destroy_surface(handle) {
                        return Err(RESP_ERR_INVALID_PARAMETER);
                    }
                }
                Command::SetFramebuffer { surface } => {
                    if !context.bind_framebuffer(surface) {
                        return Err(RESP_ERR_INVALID_PARAMETER);
                    }
                }
                Command::Clear { color } => {
                    if copy.is_some() {
                        return Err(RESP_ERR_INVALID_PARAMETER);
                    }
                    let (resource, rect) = self.framebuffer_clear_target(&context, color)?;
                    set_clear(&mut clear, resource, color, rect)?;
                }
                Command::ClearSurface {
                    handle,
                    color,
                    rect,
                } => {
                    if copy.is_some() {
                        return Err(RESP_ERR_INVALID_PARAMETER);
                    }
                    let resource = context
                        .surface_resource(handle)
                        .ok_or(RESP_ERR_INVALID_PARAMETER)?;
                    self.validate_clear(resource, color, rect)?;
                    set_clear(&mut clear, resource, color, rect)?;
                }
                Command::CopyRegion(region) => {
                    if clear.is_some() || copy.replace(region).is_some() {
                        return Err(RESP_ERR_INVALID_PARAMETER);
                    }
                    self.validate_virgl_copy(&context, region)?;
                }
                Command::Vertex(command) => vertex::apply(self, &mut context, command)?,
                Command::Shader(command) => shader::apply(&mut context, command)?,
            }
        }
        if let Some(region) = copy {
            self.apply_virgl_copy(region)?;
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
        if handle == 0 || context.has_surface(handle) || level != 0 || layers != 0 {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        let Some(target) = self.resources.get(&resource) else {
            return Err(RESP_ERR_INVALID_RESOURCE_ID);
        };
        if !context.is_attached(resource)
            || !self.is_virgl_resource(resource)
            || !target.is_texture_2d()
            || target.format != format
        {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        context.add_surface(handle, resource);
        Ok(())
    }

    fn validate_clear(&self, resource: u32, color: [f32; 4], rect: Rect) -> Result<(), u32> {
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

    fn framebuffer_clear_target(
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
        self.validate_clear(resource, color, rect)?;
        Ok((resource, rect))
    }
}

fn set_clear(
    clear: &mut Option<(u32, [f32; 4], Rect)>,
    resource: u32,
    color: [f32; 4],
    rect: Rect,
) -> Result<(), u32> {
    if clear.replace((resource, color, rect)).is_some() {
        return Err(RESP_ERR_INVALID_PARAMETER);
    }
    Ok(())
}

fn matches_scanout(scanout: Option<Scanout>, resource: u32, rect: Rect) -> bool {
    scanout.is_some_and(|current| current.resource_id == resource && current.rect == rect)
}
