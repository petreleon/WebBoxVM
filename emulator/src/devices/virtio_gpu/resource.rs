use super::protocol::{
    BackingEntry, CTRL_HEADER_LEN, RESP_ERR_INVALID_PARAMETER, RESP_ERR_INVALID_RESOURCE_ID,
    RESP_OK_NODATA, Rect, read_u32,
};
use super::{MAX_RESOURCE_BYTES, MAX_TOTAL_RESOURCE_BYTES};
use crate::memory::PhysicalMemory;

pub(super) const FORMAT_B8G8R8A8_UNORM: u32 = 1;
pub(super) const FORMAT_B8G8R8X8_UNORM: u32 = 2;
pub(super) const FORMAT_A8R8G8B8_UNORM: u32 = 3;
pub(super) const FORMAT_X8R8G8B8_UNORM: u32 = 4;
pub(super) const FORMAT_R8_UNORM: u32 = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResourceKind {
    ColorTexture2d,
    Buffer,
}

#[derive(Debug, Clone)]
pub(super) struct GpuResource {
    pub format: u32,
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    pub backing: Vec<BackingEntry>,
    kind: ResourceKind,
}

impl GpuResource {
    pub fn byte_len(width: u32, height: u32) -> Option<usize> {
        if width == 0 || height == 0 {
            return None;
        }
        let pixels = usize::try_from(width)
            .ok()?
            .checked_mul(usize::try_from(height).ok()?)?;
        pixels
            .checked_mul(4)
            .filter(|len| *len <= MAX_RESOURCE_BYTES)
    }

    pub fn supported_format(format: u32) -> bool {
        matches!(
            format,
            FORMAT_B8G8R8A8_UNORM
                | FORMAT_B8G8R8X8_UNORM
                | FORMAT_A8R8G8B8_UNORM
                | FORMAT_X8R8G8B8_UNORM
        )
    }

    pub fn new(format: u32, width: u32, height: u32) -> Option<Self> {
        let len = Self::byte_len(width, height)?;
        Self::supported_format(format).then(|| Self {
            format,
            width,
            height,
            pixels: vec![0; len],
            backing: Vec::new(),
            kind: ResourceKind::ColorTexture2d,
        })
    }

    pub fn new_buffer(width: u32) -> Option<Self> {
        let len = Self::buffer_byte_len(width)?;
        Some(Self {
            format: FORMAT_R8_UNORM,
            width,
            height: 1,
            pixels: vec![0; len],
            backing: Vec::new(),
            kind: ResourceKind::Buffer,
        })
    }

    pub fn buffer_byte_len(width: u32) -> Option<usize> {
        usize::try_from(width)
            .ok()
            .filter(|len| *len != 0 && *len <= MAX_RESOURCE_BYTES)
    }

    pub fn is_texture_2d(&self) -> bool {
        self.kind == ResourceKind::ColorTexture2d
    }

    pub fn is_buffer(&self) -> bool {
        self.kind == ResourceKind::Buffer
    }

    pub fn attach(&mut self, entries: Vec<BackingEntry>) {
        self.backing = entries;
    }

    pub fn clear_bgra(&mut self, rect: Rect, color: [u8; 4]) -> Option<()> {
        if !self.is_texture_2d() || !rect.valid_within(self.width, self.height) {
            return None;
        }
        for y in rect.y..rect.y.checked_add(rect.height)? {
            let start = usize::try_from(y.checked_mul(self.width)?.checked_add(rect.x)?).ok()? * 4;
            let end = start.checked_add(rect.width as usize * 4)?;
            for pixel in self.pixels.get_mut(start..end)?.chunks_exact_mut(4) {
                pixel.copy_from_slice(&color);
            }
        }
        Some(())
    }

    pub(super) fn transfer_buffer_to_host(
        &mut self,
        mem: &PhysicalMemory,
        offset: u64,
        start: usize,
        end: usize,
    ) -> Option<()> {
        if !self.is_buffer() || self.backing.is_empty() || end <= start || end > self.pixels.len() {
            return None;
        }
        let mut data = vec![0; end.checked_sub(start)?];
        self.read_backing(mem, offset, &mut data)?;
        self.pixels.get_mut(start..end)?.copy_from_slice(&data);
        Some(())
    }

    pub(super) fn read_backing(
        &self,
        mem: &PhysicalMemory,
        offset: u64,
        dst: &mut [u8],
    ) -> Option<()> {
        let mut skip = offset;
        let mut done = 0usize;
        for entry in &self.backing {
            if skip >= u64::from(entry.len) {
                skip -= u64::from(entry.len);
                continue;
            }
            let available = usize::try_from(u64::from(entry.len) - skip).ok()?;
            let count = available.min(dst.len() - done);
            mem.read_bytes(entry.addr.checked_add(skip)?, &mut dst[done..done + count])?;
            done += count;
            skip = 0;
            if done == dst.len() {
                return Some(());
            }
        }
        None
    }
}

impl super::VirtioGpu {
    pub(super) fn unref_resource(&mut self, input: &[u8]) -> u32 {
        if input.len() < 32 {
            return RESP_ERR_INVALID_PARAMETER;
        }
        let Some(resource_id) = read_u32(input, CTRL_HEADER_LEN) else {
            return RESP_ERR_INVALID_PARAMETER;
        };
        let Some(resource) = self.resources.remove(&resource_id) else {
            return RESP_ERR_INVALID_RESOURCE_ID;
        };
        self.allocated_resource_bytes = self
            .allocated_resource_bytes
            .saturating_sub(resource.pixels.len());
        self.remove_virgl_resource(resource_id);
        self.detach_scanout_resource(resource_id);
        RESP_OK_NODATA
    }
}

pub(super) fn total_resource_limit(current: usize, added: usize) -> bool {
    current
        .checked_add(added)
        .is_some_and(|total| total <= MAX_TOTAL_RESOURCE_BYTES)
}
