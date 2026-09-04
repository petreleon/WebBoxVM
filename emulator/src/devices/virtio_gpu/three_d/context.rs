use super::capset::{is_virgl_capset, supports};
use super::VirglContext;
use crate::devices::virtio_gpu::protocol::*;
use crate::devices::virtio_gpu::{MAX_CONTEXTS, VirtioGpu};

impl VirtioGpu {
    pub(in crate::devices::virtio_gpu) fn context_create(
        &mut self,
        header: CtrlHeader,
        input: &[u8],
    ) -> u32 {
        if header.ctx_id == 0 {
            return RESP_ERR_INVALID_CONTEXT_ID;
        }
        if input.len() != 96 {
            return RESP_ERR_INVALID_PARAMETER;
        }
        let (Some(name_len), Some(context_init)) = (read_u32(input, 24), read_u32(input, 28))
        else {
            return RESP_ERR_INVALID_PARAMETER;
        };
        let capset = context_init & 0xff;
        if name_len > 64 || context_init != capset || (capset != 0 && !supports(capset)) {
            return RESP_ERR_INVALID_PARAMETER;
        }
        if self.contexts.contains_key(&header.ctx_id) {
            return RESP_ERR_INVALID_CONTEXT_ID;
        }
        if self.contexts.len() >= MAX_CONTEXTS {
            return RESP_ERR_OUT_OF_MEMORY;
        }
        let generation = self.allocate_context_generation();
        self.contexts.insert(header.ctx_id, capset);
        self.context_generations.insert(header.ctx_id, generation);
        if is_virgl_capset(capset) {
            let generation = self.allocate_virgl_context_generation();
            self.virgl_contexts
                .insert(header.ctx_id, VirglContext::new(generation));
        }
        RESP_OK_NODATA
    }

    pub(in crate::devices::virtio_gpu) fn context_destroy(&mut self, header: CtrlHeader) -> u32 {
        if !self.contexts.contains_key(&header.ctx_id) {
            return RESP_ERR_INVALID_CONTEXT_ID;
        }
        self.contexts.remove(&header.ctx_id);
        self.context_generations.remove(&header.ctx_id);
        self.virgl_contexts.remove(&header.ctx_id);
        RESP_OK_NODATA
    }

    pub(in crate::devices::virtio_gpu) fn context_resource(
        &mut self,
        header: CtrlHeader,
        input: &[u8],
    ) -> u32 {
        if input.len() != 32 {
            return RESP_ERR_INVALID_PARAMETER;
        }
        let Some(capset) = self.contexts.get(&header.ctx_id).copied() else {
            return RESP_ERR_INVALID_CONTEXT_ID;
        };
        let Some(resource_id) = read_u32(input, 24) else {
            return RESP_ERR_INVALID_PARAMETER;
        };
        if !self.resource_exists(resource_id) {
            return RESP_ERR_INVALID_RESOURCE_ID;
        }
        if !is_virgl_capset(capset) {
            return RESP_OK_NODATA;
        }
        if !self.is_virgl_resource(resource_id) && !self.blobs.contains_key(&resource_id) {
            return RESP_ERR_INVALID_PARAMETER;
        }
        let context = self
            .virgl_contexts
            .get_mut(&header.ctx_id)
            .expect("VirGL context inserted at creation");
        match header.command_type {
            CMD_CTX_ATTACH_RESOURCE => {
                context.attach(resource_id);
                RESP_OK_NODATA
            }
            CMD_CTX_DETACH_RESOURCE if context.detach(resource_id) => RESP_OK_NODATA,
            CMD_CTX_DETACH_RESOURCE => RESP_ERR_INVALID_PARAMETER,
            _ => RESP_ERR_INVALID_PARAMETER,
        }
    }

    fn allocate_context_generation(&mut self) -> u32 {
        let generation = self.next_context_generation.max(1);
        self.next_context_generation = generation.wrapping_add(1).max(1);
        generation
    }
}
