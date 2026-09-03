use super::{VertexBuffer, VertexElement, VirglContext};
use crate::devices::virtio_gpu::three_d::virgl::shader::ShaderProgram;

#[derive(Clone, Copy)]
pub(in crate::devices::virtio_gpu) struct DrawState {
    pub vertex_buffer: VertexBuffer,
    pub vertex_element: VertexElement,
    pub vertex_program: ShaderProgram,
    pub fragment_program: ShaderProgram,
}

impl VirglContext {
    pub(in crate::devices::virtio_gpu::three_d::virgl) fn draw_state(&self) -> Option<DrawState> {
        let vertex_element = self
            .bound_vertex_elements
            .and_then(|handle| self.vertex_elements.get(&handle).copied())?;
        let vertex_program = self
            .bound_vertex_shader
            .and_then(|handle| self.shaders.get(&handle).map(|shader| shader.program))?;
        let fragment_program = self
            .bound_fragment_shader
            .and_then(|handle| self.shaders.get(&handle).map(|shader| shader.program))?;
        Some(DrawState {
            vertex_buffer: self.vertex_buffer?,
            vertex_element,
            vertex_program,
            fragment_program,
        })
    }
}
