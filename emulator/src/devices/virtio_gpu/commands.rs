mod decode;

use super::protocol::*;
use super::resource::{GpuResource, total_resource_limit};
use super::three_d::DeferredSubmit;
use super::{MAX_RESOURCES, SCANOUT_HEIGHT, SCANOUT_WIDTH, Scanout, VirtioGpu};
use crate::memory::PhysicalMemory;
use decode::{read_create_2d, read_rect_resource};
pub(super) struct CommandResult {
    pub response: Vec<u8>,
    pub deferred: Option<DeferredSubmit>,
}

impl VirtioGpu {
    pub(super) fn execute_queued_command(
        &mut self,
        mem: &mut PhysicalMemory,
        input: &[u8],
    ) -> CommandResult {
        let Some(header) = CtrlHeader::decode(input) else {
            return immediate(CtrlHeader::default().encode(RESP_ERR_UNSPEC));
        };
        if let Err(response) = self.validate_fence_header(header) {
            return immediate(header.encode(response));
        }
        let response = match header.command_type {
            CMD_GET_DISPLAY_INFO if input.len() >= CTRL_HEADER_LEN => {
                return immediate(self.display_info_response(header));
            }
            CMD_GET_CAPSET_INFO => {
                return immediate(self.capset_info_response(header, input));
            }
            CMD_GET_CAPSET => {
                return immediate(self.capset_response(header, input));
            }
            CMD_RESOURCE_CREATE_2D => self.create_2d(input),
            CMD_RESOURCE_CREATE_BLOB => self.create_blob(mem, header, input),
            CMD_RESOURCE_UNREF => self.unref_resource(mem, input),
            CMD_SET_SCANOUT => self.set_scanout(input),
            CMD_RESOURCE_FLUSH => self.flush(input),
            CMD_TRANSFER_TO_HOST_2D => self.transfer(mem, input),
            CMD_RESOURCE_ATTACH_BACKING => self.attach_backing(mem, input),
            CMD_RESOURCE_DETACH_BACKING => self.detach_backing(input),
            CMD_CTX_CREATE => self.context_create(header, input),
            CMD_CTX_DESTROY if input.len() == CTRL_HEADER_LEN => self.context_destroy(header),
            CMD_CTX_DESTROY => RESP_ERR_INVALID_PARAMETER,
            CMD_CTX_ATTACH_RESOURCE | CMD_CTX_DETACH_RESOURCE => {
                self.context_resource(header, input)
            }
            CMD_RESOURCE_CREATE_3D => self.create_virgl_resource(input),
            CMD_TRANSFER_TO_HOST_3D => self.transfer_to_host_3d(mem, header, input),
            CMD_TRANSFER_FROM_HOST_3D => match self.transfer_from_host_3d(mem, header, input) {
                Ok(Some(deferred)) => return deferred_result(header, deferred),
                Ok(None) => RESP_OK_NODATA,
                Err(response) => response,
            },
            CMD_RESOURCE_MAP_BLOB => return immediate(self.map_blob(mem, header, input)),
            CMD_RESOURCE_UNMAP_BLOB => self.unmap_blob(mem, input),
            CMD_SUBMIT_3D => match self.submit_3d(header, input) {
                Ok(Some(deferred)) => return deferred_result(header, deferred),
                Ok(None) => RESP_OK_NODATA,
                Err(response) => response,
            },
            _ => RESP_ERR_UNSPEC,
        };
        immediate(header.encode(response))
    }

    fn create_2d(&mut self, input: &[u8]) -> u32 {
        let Some((resource_id, format, width, height)) = read_create_2d(input) else {
            return RESP_ERR_INVALID_PARAMETER;
        };
        if resource_id == 0 || self.resource_exists(resource_id) {
            return RESP_ERR_INVALID_RESOURCE_ID;
        }
        if self.resource_count() >= MAX_RESOURCES {
            return RESP_ERR_OUT_OF_MEMORY;
        }
        if !GpuResource::supported_format(format) {
            return RESP_ERR_INVALID_PARAMETER;
        }
        let Some(resource_bytes) = GpuResource::byte_len(width, height) else {
            return RESP_ERR_INVALID_PARAMETER;
        };
        if !total_resource_limit(self.allocated_resource_bytes, resource_bytes) {
            return RESP_ERR_OUT_OF_MEMORY;
        }
        let resource = GpuResource::new(format, width, height)
            .expect("format, dimensions, and per-resource size checked above");
        self.allocated_resource_bytes += resource_bytes;
        self.resources.insert(resource_id, resource);
        RESP_OK_NODATA
    }

    fn set_scanout(&mut self, input: &[u8]) -> u32 {
        if input.len() < 48 {
            return RESP_ERR_INVALID_PARAMETER;
        }
        let Some(rect) = Rect::decode(input, CTRL_HEADER_LEN) else {
            return RESP_ERR_INVALID_PARAMETER;
        };
        let (Some(scanout_id), Some(resource_id)) = (read_u32(input, 40), read_u32(input, 44))
        else {
            return RESP_ERR_INVALID_PARAMETER;
        };
        if scanout_id != 0 {
            return RESP_ERR_INVALID_SCANOUT_ID;
        }
        if resource_id == 0 {
            self.scanout = None;
            self.pending_damage = None;
            return RESP_OK_NODATA;
        }
        let Some(resource) = self.resources.get(&resource_id) else {
            return RESP_ERR_INVALID_RESOURCE_ID;
        };
        if !rect.valid_within(resource.width, resource.height)
            || rect.width != SCANOUT_WIDTH
            || rect.height != SCANOUT_HEIGHT
        {
            return RESP_ERR_INVALID_PARAMETER;
        }
        self.scanout = Some(Scanout { resource_id, rect });
        self.pending_damage = None;
        RESP_OK_NODATA
    }

    fn flush(&mut self, input: &[u8]) -> u32 {
        let Some((rect, resource_id)) = read_rect_resource(input, 48) else {
            return RESP_ERR_INVALID_PARAMETER;
        };
        let Some(resource) = self.resources.get(&resource_id) else {
            return RESP_ERR_INVALID_RESOURCE_ID;
        };
        if !rect.valid_within(resource.width, resource.height) {
            return RESP_ERR_INVALID_PARAMETER;
        }
        if self.resident_resources.contains_key(&resource_id) { return RESP_OK_NODATA; }
        self.add_damage(resource_id, rect);
        RESP_OK_NODATA
    }

    fn transfer(&mut self, mem: &PhysicalMemory, input: &[u8]) -> u32 {
        if input.len() < 56 {
            return RESP_ERR_INVALID_PARAMETER;
        }
        let (Some(rect), Some(offset), Some(resource_id)) = (
            Rect::decode(input, CTRL_HEADER_LEN),
            read_u64(input, 40),
            read_u32(input, 48),
        ) else {
            return RESP_ERR_INVALID_PARAMETER;
        };
        if !self.resident_overwrite_allowed(resource_id, rect) { return RESP_ERR_INVALID_PARAMETER; }
        let Some(resource) = self.resources.get_mut(&resource_id) else {
            return RESP_ERR_INVALID_RESOURCE_ID;
        };
        if resource.transfer(mem, rect, offset).is_none() {
            return RESP_ERR_INVALID_PARAMETER;
        }
        self.forget_resident(resource_id);
        RESP_OK_NODATA
    }
}

fn immediate(response: Vec<u8>) -> CommandResult {
    CommandResult {
        response,
        deferred: None,
    }
}

fn deferred_result(header: CtrlHeader, deferred: DeferredSubmit) -> CommandResult {
    CommandResult {
        response: header.encode(RESP_OK_NODATA),
        deferred: Some(deferred),
    }
}
