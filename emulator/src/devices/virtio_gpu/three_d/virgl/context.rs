mod blend;
mod constant;
mod depth;
mod depth_compare;
mod draw;
mod framebuffer;
mod index;
mod pipeline;
mod rasterizer;
mod sampler;
mod shader;
mod vertex;

use super::blob::RendererBlobObject;
use super::shader::Shader;
use crate::devices::virtio_gpu::blob::BlobMemory;
use std::collections::{HashMap, HashSet};
use framebuffer::{Framebuffer, Surface};

pub(in crate::devices::virtio_gpu::three_d::virgl) use constant::{
    FragmentConstants, UniformBinding,
};
pub(super) use draw::DrawState;
use pipeline::PipelineState;
pub(in crate::devices::virtio_gpu::three_d::virgl) use pipeline::{
    SampledResource, SamplerAddressMode, SamplerConfig, SamplerFilter, SamplerState, Viewport,
};
pub(in crate::devices::virtio_gpu) use depth_compare::{DepthCompare, DepthState};
use shader::PendingShader;

pub(in crate::devices::virtio_gpu) use index::IndexBuffer;
use vertex::VertexState;
pub(super) use vertex::{MAX_VIRGL_VERTEX_BUFFERS, VertexLayout};
pub(in crate::devices::virtio_gpu) use vertex::{VertexBuffer, VertexElement};

#[derive(Clone, Debug)]
pub(in crate::devices::virtio_gpu) struct VirglContext {
    pub(in crate::devices::virtio_gpu) generation: u32,
    attached: HashSet<u32>,
    renderer_blobs: HashMap<u64, RendererBlobObject>,
    framebuffer: Option<Framebuffer>,
    surfaces: HashMap<u32, Surface>,
    pipeline: PipelineState,
    vertex: VertexState,
    index_buffer: Option<IndexBuffer>,
    shaders: HashMap<u32, Shader>,
    pending_vertex_shader: Option<PendingShader>,
    pending_fragment_shader: Option<PendingShader>,
    bound_vertex_shader: Option<u32>,
    bound_fragment_shader: Option<u32>,
    vertex_uniform: Option<UniformBinding>,
    fragment_constants: Option<FragmentConstants>,
}

impl VirglContext {
    pub(in crate::devices::virtio_gpu) fn new(generation: u32) -> Self {
        Self {
            generation,
            attached: HashSet::new(),
            renderer_blobs: HashMap::new(),
            framebuffer: None, surfaces: HashMap::new(),
            pipeline: PipelineState::new(),
            vertex: VertexState::new(),
            index_buffer: None,
            shaders: HashMap::new(),
            pending_vertex_shader: None,
            pending_fragment_shader: None,
            bound_vertex_shader: None,
            bound_fragment_shader: None,
            vertex_uniform: None,
            fragment_constants: None,
        }
    }

    pub(in crate::devices::virtio_gpu) fn attach(&mut self, resource_id: u32) {
        self.attached.insert(resource_id);
    }

    pub(in crate::devices::virtio_gpu) fn detach(&mut self, resource_id: u32) -> bool {
        if !self.attached.remove(&resource_id) {
            return false;
        }
        self.remove_framebuffer_resource(resource_id);
        self.surfaces.retain(|_, surface| surface.resource != resource_id);
        self.remove_sampler_resource(resource_id);
        self.remove_vertex_resource(resource_id);
        self.remove_index_resource(resource_id);
        self.remove_constant_resource(resource_id);
        true
    }

    pub(in crate::devices::virtio_gpu) fn is_attached(&self, resource_id: u32) -> bool {
        self.attached.contains(&resource_id)
    }

    pub(in crate::devices::virtio_gpu) fn remove_resource(&mut self, resource_id: u32) {
        let _ = self.detach(resource_id);
    }

    pub(super) fn prepare_renderer_blob(
        &mut self,
        blob_id: u64,
        object: RendererBlobObject,
    ) -> bool {
        if self.renderer_blobs.len() >= super::blob::MAX_RENDERER_BLOB_OBJECTS
            || self.renderer_blobs.contains_key(&blob_id)
        {
            return false;
        }
        self.renderer_blobs.insert(blob_id, object);
        true
    }

    pub(in crate::devices::virtio_gpu) fn has_renderer_blob(
        &self,
        blob_id: u64,
        memory: BlobMemory,
        flags: u32,
        size: usize,
    ) -> bool {
        self.renderer_blobs
            .get(&blob_id)
            .is_some_and(|object| object.matches(memory, flags, size))
    }

    pub(in crate::devices::virtio_gpu) fn consume_renderer_blob(&mut self, blob_id: u64) {
        self.renderer_blobs.remove(&blob_id);
    }
}
