use super::super::MAX_VIRGL_FRAGMENT_SAMPLERS;
use super::{
    IndexBuffer, MAX_VIRGL_VERTEX_BUFFERS, SampledResource, VertexBuffer, VertexLayout,
    Viewport, VirglContext,
};
use crate::devices::virtio_gpu::protocol::Rect;
use crate::devices::virtio_gpu::three_d::virgl::shader::ShaderProgram;

#[derive(Clone, Copy)]
pub(in crate::devices::virtio_gpu::three_d::virgl) struct DrawState {
    pub vertex_buffers: [Option<VertexBuffer>; MAX_VIRGL_VERTEX_BUFFERS],
    pub vertex_layout: VertexLayout,
    pub index_buffer: Option<IndexBuffer>,
    pub vertex_program: ShaderProgram,
    pub fragment_program: ShaderProgram,
    pub fragment_constants: Option<[u32; 4]>,
    pub viewport: Viewport,
    pub scissor: Option<Rect>,
    pub sampled_resources: [Option<SampledResource>; MAX_VIRGL_FRAGMENT_SAMPLERS],
}

impl VirglContext {
    pub(in crate::devices::virtio_gpu::three_d::virgl) fn draw_state(&self) -> Option<DrawState> {
        let blend = self.pipeline.bound_blend_state?;
        self.pipeline.blend_states.contains(&blend).then_some(())?;
        let rasterizer = self
            .pipeline
            .bound_rasterizer
            .and_then(|handle| self.pipeline.rasterizers.get(&handle).copied())?;
        let (vertex_buffers, vertex_layout) = self.draw_vertex_state()?;
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
            vertex_buffers,
            vertex_layout,
            index_buffer: self.index_buffer,
            vertex_program,
            fragment_program,
            fragment_constants: self.fragment_constants,
            viewport: self.pipeline.viewport?,
            scissor,
            sampled_resources: self.sampled_resources(),
        })
    }
}
