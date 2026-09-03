mod blend;
mod clear;
mod decode;
mod sampler;
mod shader;
mod state;
mod vertex;

use super::VirglContext;
use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::protocol::*;
use clear::set;
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
        let mut draw = None;
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
                    if copy.is_some() || draw.is_some() {
                        return Err(RESP_ERR_INVALID_PARAMETER);
                    }
                    let (resource, rect) = self.framebuffer_virgl_clear_target(&context, color)?;
                    set(&mut clear, resource, color, rect)?;
                }
                Command::ClearSurface {
                    handle,
                    color,
                    rect,
                } => {
                    if copy.is_some() || draw.is_some() {
                        return Err(RESP_ERR_INVALID_PARAMETER);
                    }
                    let resource = context
                        .surface_resource(handle)
                        .ok_or(RESP_ERR_INVALID_PARAMETER)?;
                    self.validate_virgl_clear(resource, color, rect)?;
                    set(&mut clear, resource, color, rect)?;
                }
                Command::CopyRegion(region) => {
                    if clear.is_some() || draw.is_some() || copy.replace(region).is_some() {
                        return Err(RESP_ERR_INVALID_PARAMETER);
                    }
                    self.validate_virgl_copy(&context, region)?;
                }
                Command::Draw(call) => {
                    if copy.is_some() || draw.is_some() {
                        return Err(RESP_ERR_INVALID_PARAMETER);
                    }
                    let (resource, _, rect) = clear.ok_or(RESP_ERR_INVALID_PARAMETER)?;
                    draw = Some(self.prepare_virgl_draw(&context, resource, rect, call)?);
                }
                Command::Blend(command) => blend::apply(&mut context, command)?,
                Command::Vertex(command) => vertex::apply(self, &mut context, command)?,
                Command::Sampler(command) => sampler::apply(self, &mut context, command)?,
                Command::Shader(command) => shader::apply(&mut context, command)?,
                Command::State(command) => state::apply(&mut context, command)?,
            }
        }
        if let Some(region) = copy {
            self.apply_virgl_copy(region)?;
        }
        let deferred = match (clear, draw) {
            (Some((resource, color, rect)), Some(work)) => Some(self.queue_virgl_draw(
                header,
                context.generation,
                resource,
                rect,
                color,
                work,
            )?),
            (Some((resource, color, rect)), None) => {
                Some(self.queue_virgl_clear(header, context.generation, resource, rect, color)?)
            }
            (None, None) => None,
            (None, Some(_)) => return Err(RESP_ERR_INVALID_PARAMETER),
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
}
