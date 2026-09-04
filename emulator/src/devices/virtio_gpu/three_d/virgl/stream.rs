mod batch;
mod blend;
mod clear;
mod constant;
mod depth;
mod decode;
mod index;
mod sampler;
mod shader;
mod state;
mod uniform;
mod vertex;

use super::VirglContext;
use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::protocol::*;
use batch::Batch;
use clear::{Clear, set};
use decode::{Command, decode_stream};

impl VirtioGpu {
    pub(in crate::devices::virtio_gpu) fn submit_virgl(
        &mut self,
        header: CtrlHeader,
        input: &[u8],
    ) -> Result<Option<super::DeferredSubmit>, u32> {
        let mut context = self
            .virgl_contexts
            .get(&header.ctx_id)
            .cloned()
            .ok_or(RESP_ERR_INVALID_CONTEXT_ID)?;
        if let Some(result) = super::blob::prepare(&mut context, input) {
            result?;
            self.virgl_contexts.insert(header.ctx_id, context);
            return Ok(None);
        }
        let commands = decode_stream(input).ok_or(RESP_ERR_INVALID_PARAMETER)?;
        let inline_writes = commands
            .iter()
            .filter(|command| matches!(command, Command::InlineWrite(_)))
            .count();
        if inline_writes != 0
            && (inline_writes != 1
                || commands
                    .iter()
                    .any(|command| !matches!(command, Command::Nop | Command::InlineWrite(_))))
        {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        let mut clear = None;
        let mut copy = None;
        let mut draws = Batch::default();
        let mut inline_write = None;
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
                Command::SetFramebuffer { color, depth } => {
                    if !context.bind_framebuffer(color, depth) {
                        return Err(RESP_ERR_INVALID_PARAMETER);
                    }
                }
                Command::Clear { color, depth } => {
                    if copy.is_some() || !draws.is_empty() {
                        return Err(RESP_ERR_INVALID_PARAMETER);
                    }
                    let target = self.framebuffer_virgl_clear_target(&context, color, depth)?;
                    set(&mut clear, target)?;
                }
                Command::ClearSurface {
                    handle,
                    color,
                    rect,
                } => {
                    if copy.is_some() || !draws.is_empty() {
                        return Err(RESP_ERR_INVALID_PARAMETER);
                    }
                    let resource = context
                        .surface_resource(handle)
                        .ok_or(RESP_ERR_INVALID_PARAMETER)?;
                    self.validate_virgl_clear(resource, color, rect)?;
                    set(&mut clear, Clear { resource, depth_resource: None, color, rect })?;
                }
                Command::CopyRegion(region) => {
                    if clear.is_some() || !draws.is_empty() || copy.replace(region).is_some() {
                        return Err(RESP_ERR_INVALID_PARAMETER);
                    }
                    self.validate_virgl_copy(&context, region)?;
                }
                Command::InlineWrite(write) => {
                    self.validate_virgl_inline_write(&context, &write)?;
                    inline_write = Some(write);
                }
                Command::Draw(call) => {
                    if copy.is_some() {
                        return Err(RESP_ERR_INVALID_PARAMETER);
                    }
                    let target = clear.ok_or(RESP_ERR_INVALID_PARAMETER)?;
                    draws.push(self.prepare_virgl_draw(
                        &context, target.resource, target.depth_resource, target.rect, call,
                    )?)?;
                }
                Command::Blend(command) => blend::apply(&mut context, command)?,
                Command::Constant(command) => constant::apply(&mut context, command),
                Command::Depth(command) => depth::apply(&mut context, command)?,
                Command::Uniform(command) => uniform::apply(self, &mut context, command)?,
                Command::Vertex(command) => vertex::apply(self, &mut context, command)?,
                Command::Index(command) => index::apply(self, &mut context, command)?,
                Command::Sampler(command) => sampler::apply(self, &mut context, command)?,
                Command::Shader(command) => shader::apply(&mut context, command)?,
                Command::State(command) => state::apply(&mut context, command)?,
            }
        }
        if let Some(region) = copy {
            self.apply_virgl_copy(region)?;
        }
        if let Some(write) = inline_write {
            self.apply_virgl_inline_write(write)?;
        }
        let deferred = batch::deferred(self, header, context.generation, clear, draws)?;
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
            || (!target.is_texture_2d() && !target.is_depth_texture_2d())
            || target.format != format
        {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        context.add_surface(handle, resource, target.is_depth_texture_2d());
        Ok(())
    }
}
