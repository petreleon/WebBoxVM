use super::super::VirglContext;
use crate::devices::virtio_gpu::three_d::virgl::shader::{MAX_SHADER_TEXT_BYTES, ShaderKind};

#[derive(Clone, Debug)]
pub(in crate::devices::virtio_gpu::three_d::virgl::context) struct PendingShader {
    handle: u32,
    token_count: u32,
    total_bytes: usize,
    source: Vec<u8>,
}

impl VirglContext {
    pub(in crate::devices::virtio_gpu::three_d::virgl) fn accept_shader_chunk(
        &mut self,
        handle: u32,
        kind: ShaderKind,
        token_count: u32,
        total_bytes: Option<u32>,
        offset: u32,
        source: Vec<u8>,
    ) -> Option<Option<Vec<u8>>> {
        match total_bytes {
            Some(total_bytes) => self.start_shader(handle, kind, token_count, total_bytes, source),
            None => self.continue_shader(handle, kind, token_count, offset, source),
        }
    }

    pub(super) fn has_pending_handle(&self, handle: u32) -> bool {
        self.pending_vertex_shader
            .as_ref()
            .map(|pending| pending.handle)
            == Some(handle)
            || self
                .pending_fragment_shader
                .as_ref()
                .map(|pending| pending.handle)
                == Some(handle)
    }

    fn start_shader(
        &mut self,
        handle: u32,
        kind: ShaderKind,
        token_count: u32,
        total_bytes: u32,
        mut source: Vec<u8>,
    ) -> Option<Option<Vec<u8>>> {
        let total_bytes = usize::try_from(total_bytes).ok()?;
        let padded_bytes = padded_bytes(total_bytes)?;
        if !(2..=MAX_SHADER_TEXT_BYTES).contains(&total_bytes)
            || source.is_empty()
            || source.len() > padded_bytes
            || self.shaders.contains_key(&handle)
            || self.has_pending_handle(handle)
            || self.pending_shader(kind).is_some()
        {
            return None;
        }
        if source.len() == padded_bytes {
            source.truncate(total_bytes);
            return Some(Some(source));
        }
        *self.pending_shader_mut(kind) = Some(PendingShader {
            handle,
            token_count,
            total_bytes,
            source,
        });
        Some(None)
    }

    fn continue_shader(
        &mut self,
        handle: u32,
        kind: ShaderKind,
        token_count: u32,
        offset: u32,
        source: Vec<u8>,
    ) -> Option<Option<Vec<u8>>> {
        let pending = self.pending_shader(kind).as_ref()?;
        let offset = usize::try_from(offset).ok()?;
        let next = pending.source.len().checked_add(source.len())?;
        if source.is_empty()
            || handle != pending.handle
            || token_count != pending.token_count
            || offset != pending.source.len()
            || next > padded_bytes(pending.total_bytes)?
        {
            return None;
        }
        self.pending_shader_mut(kind)
            .as_mut()?
            .source
            .extend(source);
        if next < padded_bytes(self.pending_shader(kind).as_ref()?.total_bytes)? {
            return Some(None);
        }
        let mut pending = self.pending_shader_mut(kind).take()?;
        pending.source.truncate(pending.total_bytes);
        Some(Some(pending.source))
    }

    fn pending_shader(&self, kind: ShaderKind) -> &Option<PendingShader> {
        match kind {
            ShaderKind::Vertex => &self.pending_vertex_shader,
            ShaderKind::Fragment => &self.pending_fragment_shader,
        }
    }

    fn pending_shader_mut(&mut self, kind: ShaderKind) -> &mut Option<PendingShader> {
        match kind {
            ShaderKind::Vertex => &mut self.pending_vertex_shader,
            ShaderKind::Fragment => &mut self.pending_fragment_shader,
        }
    }
}

fn padded_bytes(bytes: usize) -> Option<usize> {
    bytes.div_ceil(4).checked_mul(4)
}
