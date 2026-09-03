mod blend;
mod draw;
mod index;
mod pipeline;
mod rasterizer;
mod sampler;
mod shader;
mod vertex;

use super::shader::Shader;
use std::collections::{HashMap, HashSet};

pub(super) use draw::DrawState;
use pipeline::PipelineState;
pub(in crate::devices::virtio_gpu::three_d::virgl) use pipeline::{
    SampledResource, SamplerAddressMode, SamplerState, Viewport,
};
use shader::PendingShader;

pub(in crate::devices::virtio_gpu) use index::IndexBuffer;
pub(super) use vertex::VertexLayout;
use vertex::VertexState;
pub(in crate::devices::virtio_gpu) use vertex::{VertexBuffer, VertexElement};

#[derive(Clone, Debug)]
pub(in crate::devices::virtio_gpu) struct VirglContext {
    pub(in crate::devices::virtio_gpu) generation: u32,
    attached: HashSet<u32>,
    framebuffer: Option<u32>,
    surfaces: HashMap<u32, u32>,
    pipeline: PipelineState,
    vertex: VertexState,
    index_buffer: Option<IndexBuffer>,
    shaders: HashMap<u32, Shader>,
    pending_vertex_shader: Option<PendingShader>,
    pending_fragment_shader: Option<PendingShader>,
    bound_vertex_shader: Option<u32>,
    bound_fragment_shader: Option<u32>,
}

impl VirglContext {
    pub(in crate::devices::virtio_gpu) fn new(generation: u32) -> Self {
        Self {
            generation,
            attached: HashSet::new(),
            framebuffer: None,
            surfaces: HashMap::new(),
            pipeline: PipelineState::new(),
            vertex: VertexState::new(),
            index_buffer: None,
            shaders: HashMap::new(),
            pending_vertex_shader: None,
            pending_fragment_shader: None,
            bound_vertex_shader: None,
            bound_fragment_shader: None,
        }
    }

    pub(in crate::devices::virtio_gpu) fn attach(&mut self, resource_id: u32) {
        self.attached.insert(resource_id);
    }

    pub(in crate::devices::virtio_gpu) fn detach(&mut self, resource_id: u32) -> bool {
        if !self.attached.remove(&resource_id) {
            return false;
        }
        if self
            .framebuffer
            .is_some_and(|handle| self.surface_resource(handle) == Some(resource_id))
        {
            self.framebuffer = None;
        }
        self.surfaces.retain(|_, resource| *resource != resource_id);
        self.remove_sampler_resource(resource_id);
        self.remove_vertex_resource(resource_id);
        self.remove_index_resource(resource_id);
        true
    }

    pub(in crate::devices::virtio_gpu) fn has_surface(&self, handle: u32) -> bool {
        self.surfaces.contains_key(&handle)
    }

    pub(in crate::devices::virtio_gpu) fn is_attached(&self, resource_id: u32) -> bool {
        self.attached.contains(&resource_id)
    }

    pub(in crate::devices::virtio_gpu) fn add_surface(&mut self, handle: u32, resource: u32) {
        self.surfaces.insert(handle, resource);
    }

    pub(in crate::devices::virtio_gpu) fn destroy_surface(&mut self, handle: u32) -> bool {
        if self.surfaces.remove(&handle).is_none() {
            return false;
        }
        if self.framebuffer == Some(handle) {
            self.framebuffer = None;
        }
        true
    }

    pub(in crate::devices::virtio_gpu) fn bind_framebuffer(
        &mut self,
        surface: Option<u32>,
    ) -> bool {
        if surface.is_some_and(|handle| !self.has_surface(handle)) {
            return false;
        }
        self.framebuffer = surface;
        true
    }

    pub(in crate::devices::virtio_gpu) fn surface_resource(&self, handle: u32) -> Option<u32> {
        self.surfaces.get(&handle).copied()
    }

    pub(in crate::devices::virtio_gpu) fn framebuffer_resource(&self) -> Option<u32> {
        self.framebuffer
            .and_then(|handle| self.surface_resource(handle))
    }

    pub(in crate::devices::virtio_gpu) fn remove_resource(&mut self, resource_id: u32) {
        let _ = self.detach(resource_id);
    }
}
