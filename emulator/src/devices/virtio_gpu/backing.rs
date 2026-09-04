use super::protocol::*;
use super::{BackingEntry, MAX_BACKING_ENTRIES, SCANOUT_HEIGHT, SCANOUT_WIDTH, VirtioGpu};
use crate::memory::PhysicalMemory;

impl VirtioGpu {
    pub(super) fn display_info_response(&self, header: CtrlHeader) -> Vec<u8> {
        let mut out = header.encode(RESP_OK_DISPLAY_INFO);
        for index in 0..16 {
            for value in if index == 0 {
                [0, 0, SCANOUT_WIDTH, SCANOUT_HEIGHT, 1, 0]
            } else {
                [0; 6]
            } {
                push_u32(&mut out, value);
            }
        }
        out
    }

    pub(super) fn attach_backing(&mut self, mem: &PhysicalMemory, input: &[u8]) -> u32 {
        if input.len() < 32 {
            return RESP_ERR_INVALID_PARAMETER;
        }
        let (Some(resource_id), Some(entry_count)) = (read_u32(input, 24), read_u32(input, 28))
        else {
            return RESP_ERR_INVALID_PARAMETER;
        };
        let Ok(entry_count) = usize::try_from(entry_count) else {
            return RESP_ERR_INVALID_PARAMETER;
        };
        let Some(expected) = entry_count.checked_mul(16).and_then(|n| n.checked_add(32)) else {
            return RESP_ERR_INVALID_PARAMETER;
        };
        if entry_count == 0 || entry_count > MAX_BACKING_ENTRIES || input.len() < expected {
            return RESP_ERR_INVALID_PARAMETER;
        }
        let Some((resource_len, has_backing)) = self
            .resources
            .get(&resource_id)
            .map(|resource| (resource.pixels.len(), !resource.backing.is_empty()))
            .or_else(|| {
                self.blobs
                    .get(&resource_id)
                    .map(|blob| (blob.size, !blob.backing.is_empty()))
            })
        else {
            return RESP_ERR_INVALID_RESOURCE_ID;
        };
        if has_backing {
            return RESP_ERR_INVALID_PARAMETER;
        }
        let Some(entries) = decode_entries(mem, input, 32, entry_count, resource_len) else {
            return RESP_ERR_INVALID_PARAMETER;
        };
        if let Some(resource) = self.resources.get_mut(&resource_id) {
            resource.attach(entries);
        } else {
            self.blobs
                .get_mut(&resource_id)
                .expect("blob checked above")
                .backing = entries;
        }
        RESP_OK_NODATA
    }

    pub(super) fn detach_backing(&mut self, input: &[u8]) -> u32 {
        if input.len() < 32 {
            return RESP_ERR_INVALID_PARAMETER;
        }
        let Some(resource_id) = read_u32(input, 24) else {
            return RESP_ERR_INVALID_PARAMETER;
        };
        if let Some(resource) = self.resources.get_mut(&resource_id) {
            if resource.backing.is_empty() {
                return RESP_ERR_INVALID_PARAMETER;
            }
            resource.backing.clear();
        } else if let Some(blob) = self.blobs.get_mut(&resource_id) {
            if blob.backing.is_empty() {
                return RESP_ERR_INVALID_PARAMETER;
            }
            blob.backing.clear();
        } else {
            return RESP_ERR_INVALID_RESOURCE_ID;
        }
        RESP_OK_NODATA
    }
}

pub(super) fn decode_entries(
    mem: &PhysicalMemory,
    input: &[u8],
    start: usize,
    count: usize,
    resource_len: usize,
) -> Option<Vec<BackingEntry>> {
    let mut entries = Vec::with_capacity(count);
    let mut total = 0usize;
    for index in 0..count {
        let offset = start.checked_add(index.checked_mul(16)?)?;
        let addr = read_u64(input, offset)?;
        let len = read_u32(input, offset + 8)?;
        let len_usize = usize::try_from(len).ok()?;
        if len == 0 || !mem.contains_range(addr, len_usize) {
            return None;
        }
        total = total.checked_add(len_usize)?;
        let rounded_len = resource_len.checked_add(4095)? & !4095;
        if total > rounded_len {
            return None;
        }
        entries.push(BackingEntry { addr, len });
    }
    (total >= resource_len).then_some(entries)
}
