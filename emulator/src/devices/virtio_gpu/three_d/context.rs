use super::CAPSET_ID;
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
        if name_len > 64 || !matches!(context_init, 0 | CAPSET_ID) {
            return RESP_ERR_INVALID_PARAMETER;
        }
        if self.contexts.contains_key(&header.ctx_id) {
            return RESP_ERR_INVALID_CONTEXT_ID;
        }
        if self.contexts.len() >= MAX_CONTEXTS {
            return RESP_ERR_OUT_OF_MEMORY;
        }
        self.contexts.insert(header.ctx_id, context_init);
        RESP_OK_NODATA
    }

    pub(in crate::devices::virtio_gpu) fn context_destroy(&mut self, header: CtrlHeader) -> u32 {
        if !self.contexts.contains_key(&header.ctx_id) {
            return RESP_ERR_INVALID_CONTEXT_ID;
        }
        self.contexts.remove(&header.ctx_id);
        RESP_OK_NODATA
    }

    pub(in crate::devices::virtio_gpu) fn context_resource(
        &self,
        header: CtrlHeader,
        input: &[u8],
    ) -> u32 {
        if input.len() != 32 {
            return RESP_ERR_INVALID_PARAMETER;
        }
        if !self.contexts.contains_key(&header.ctx_id) {
            return RESP_ERR_INVALID_CONTEXT_ID;
        }
        match read_u32(input, 24) {
            Some(resource_id) if self.resources.contains_key(&resource_id) => RESP_OK_NODATA,
            Some(_) => RESP_ERR_INVALID_RESOURCE_ID,
            None => RESP_ERR_INVALID_PARAMETER,
        }
    }
}
