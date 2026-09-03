mod chunks;

use super::VirglContext;
#[cfg(test)]
use crate::devices::virtio_gpu::three_d::virgl::shader::ShaderProgram;
use crate::devices::virtio_gpu::three_d::virgl::shader::{Shader, ShaderKind};

pub(in crate::devices::virtio_gpu::three_d::virgl::context) use chunks::PendingShader;

impl VirglContext {
    pub(in crate::devices::virtio_gpu::three_d::virgl) fn create_shader(
        &mut self,
        handle: u32,
        shader: Shader,
    ) -> bool {
        if handle == 0 || self.shaders.contains_key(&handle) || self.has_pending_handle(handle) {
            return false;
        }
        self.shaders.insert(handle, shader);
        true
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn bind_shader(
        &mut self,
        kind: ShaderKind,
        handle: Option<u32>,
    ) -> bool {
        if handle
            .is_some_and(|handle| self.shaders.get(&handle).map(|shader| shader.kind) != Some(kind))
        {
            return false;
        }
        match kind {
            ShaderKind::Vertex => self.bound_vertex_shader = handle,
            ShaderKind::Fragment => self.bound_fragment_shader = handle,
        }
        true
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn destroy_shader(
        &mut self,
        handle: u32,
    ) -> bool {
        if self.has_pending_handle(handle) {
            return false;
        }
        let Some(shader) = self.shaders.remove(&handle) else {
            return false;
        };
        if self.bound_handle(shader.kind) == Some(handle) {
            let _ = self.bind_shader(shader.kind, None);
        }
        true
    }

    #[cfg(test)]
    pub(in crate::devices::virtio_gpu) fn bound_shader(
        &self,
        kind: ShaderKind,
    ) -> Option<ShaderProgram> {
        self.bound_handle(kind)
            .and_then(|handle| self.shaders.get(&handle).map(|shader| shader.program))
    }

    fn bound_handle(&self, kind: ShaderKind) -> Option<u32> {
        match kind {
            ShaderKind::Vertex => self.bound_vertex_shader,
            ShaderKind::Fragment => self.bound_fragment_shader,
        }
    }
}
