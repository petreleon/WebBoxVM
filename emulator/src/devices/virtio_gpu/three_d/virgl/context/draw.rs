use super::{VertexBuffer, VertexElement, Viewport, VirglContext};
use crate::devices::virtio_gpu::protocol::Rect;
use crate::devices::virtio_gpu::three_d::virgl::shader::ShaderProgram;

#[derive(Clone, Copy)]
pub(in crate::devices::virtio_gpu::three_d::virgl) struct DrawState {
    pub vertex_buffer: VertexBuffer,
    pub vertex_element: VertexElement,
    pub vertex_program: ShaderProgram,
    pub fragment_program: ShaderProgram,
    pub viewport: Viewport,
    pub scissor: Option<Rect>,
}

impl VirglContext {
    pub(in crate::devices::virtio_gpu::three_d::virgl) fn draw_state(&self) -> Option<DrawState> {
        let blend = self.pipeline.bound_blend_state?;
        self.pipeline.blend_states.contains(&blend).then_some(())?;
        let rasterizer = self
            .pipeline
            .bound_rasterizer
            .and_then(|handle| self.pipeline.rasterizers.get(&handle).copied())?;
        let vertex_element = self
            .bound_vertex_elements
            .and_then(|handle| self.vertex_elements.get(&handle).copied())?;
        let vertex_program = self
            .bound_vertex_shader
            .and_then(|handle| self.shaders.get(&handle).map(|shader| shader.program))?;
        let fragment_program = self
            .bound_fragment_shader
            .and_then(|handle| self.shaders.get(&handle).map(|shader| shader.program))?;
        let scissor = if rasterizer.scissor {
            Some(self.pipeline.scissor?)
        } else {
            None
        };
        Some(DrawState {
            vertex_buffer: self.vertex_buffer?,
            vertex_element,
            vertex_program,
            fragment_program,
            viewport: self.pipeline.viewport?,
            scissor,
        })
    }
}
