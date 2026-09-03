use super::VIRGL_CAPSET_ID;
use crate::devices::virtio_gpu::VirtioGpu;
use crate::devices::virtio_gpu::protocol::*;
use crate::devices::virtio_gpu::resource::GpuResource;
use crate::memory::PhysicalMemory;

const TRANSFER_3D_BYTES: usize = 72;

struct Transfer3d {
    box3d: Box3d,
    offset: u64,
    resource_id: u32,
    level: u32,
    stride: u32,
    layer_stride: u32,
}

impl Transfer3d {
    fn decode(input: &[u8]) -> Option<Self> {
        if input.len() != TRANSFER_3D_BYTES {
            return None;
        }
        Some(Self {
            box3d: Box3d::decode(input, CTRL_HEADER_LEN)?,
            offset: read_u64(input, 48)?,
            resource_id: read_u32(input, 56)?,
            level: read_u32(input, 60)?,
            stride: read_u32(input, 64)?,
            layer_stride: read_u32(input, 68)?,
        })
    }

    fn texture_rect(&self) -> Option<Rect> {
        (self.level == 0 && self.stride == 0 && self.layer_stride == 0)
            .then(|| self.box3d.flat_rect())
            .flatten()
    }

    fn buffer_range(&self, resource: &GpuResource) -> Option<(usize, usize)> {
        (self.level == 0 && self.stride == 0 && self.layer_stride == 0)
            .then(|| self.box3d.buffer_range(resource.pixels.len()))
            .flatten()
    }
}

impl VirtioGpu {
    pub(in crate::devices::virtio_gpu) fn transfer_to_host_3d(
        &mut self,
        mem: &PhysicalMemory,
        header: CtrlHeader,
        input: &[u8],
    ) -> u32 {
        let transfer = match self.virgl_transfer(header, input) {
            Ok(transfer) => transfer,
            Err(response) => return response,
        };
        let resource = self
            .resources
            .get_mut(&transfer.resource_id)
            .expect("resource existence checked above");
        let transferred = if resource.is_texture_2d() {
            transfer
                .texture_rect()
                .and_then(|rect| resource.transfer(mem, rect, transfer.offset))
        } else if resource.is_buffer() {
            transfer.buffer_range(resource).and_then(|(start, end)| {
                resource.transfer_buffer_to_host(mem, transfer.offset, start, end)
            })
        } else {
            None
        };
        if transferred.is_none() {
            return RESP_ERR_INVALID_PARAMETER;
        }
        RESP_OK_NODATA
    }

    pub(in crate::devices::virtio_gpu) fn transfer_from_host_3d(
        &mut self,
        mem: &mut PhysicalMemory,
        header: CtrlHeader,
        input: &[u8],
    ) -> u32 {
        let transfer = match self.virgl_transfer(header, input) {
            Ok(transfer) => transfer,
            Err(response) => return response,
        };
        let resource = self
            .resources
            .get(&transfer.resource_id)
            .expect("resource existence checked above");
        let transferred = if resource.is_texture_2d() {
            transfer
                .texture_rect()
                .and_then(|rect| resource.transfer_from_host(mem, rect, transfer.offset))
        } else if resource.is_buffer() {
            transfer.buffer_range(resource).and_then(|(start, end)| {
                resource.transfer_buffer_from_host(mem, transfer.offset, start, end)
            })
        } else {
            None
        };
        if transferred.is_none() {
            return RESP_ERR_INVALID_PARAMETER;
        }
        RESP_OK_NODATA
    }

    fn virgl_transfer(&self, header: CtrlHeader, input: &[u8]) -> Result<Transfer3d, u32> {
        let transfer = Transfer3d::decode(input).ok_or(RESP_ERR_INVALID_PARAMETER)?;
        if !self.resources.contains_key(&transfer.resource_id) {
            return Err(RESP_ERR_INVALID_RESOURCE_ID);
        }
        if self.contexts.get(&header.ctx_id) != Some(&VIRGL_CAPSET_ID) {
            return Err(RESP_ERR_INVALID_CONTEXT_ID);
        }
        if !self.is_virgl_resource(transfer.resource_id) {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        Ok(transfer)
    }
}
