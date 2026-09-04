use super::protocol::Rect;
use super::resource::{
    FORMAT_A8R8G8B8_UNORM, FORMAT_B8G8R8A8_UNORM, FORMAT_B8G8R8X8_UNORM, FORMAT_R8G8B8A8_UNORM,
    FORMAT_X8R8G8B8_UNORM, GpuResource,
};
use crate::memory::PhysicalMemory;
mod readback;
struct BackingWrite {
    addr: u64,
    source: usize,
    len: usize,
}
impl GpuResource {
    pub(super) fn transfer(&mut self, mem: &PhysicalMemory, rect: Rect, offset: u64) -> Option<()> {
        if !self.is_texture_2d()
            || !rect.valid_within(self.width, self.height)
            || self.backing.is_empty()
        {
            return None;
        }
        let stride = u64::from(self.width).checked_mul(4)?;
        let row_len = usize::try_from(u64::from(rect.width).checked_mul(4)?).ok()?;
        let mut raw = vec![0; row_len];
        for row in 0..rect.height {
            let y = u64::from(rect.y.checked_add(row)?);
            self.read_backing(
                mem,
                offset.checked_add(u64::from(row).checked_mul(stride)?)?,
                &mut raw,
            )?;
            let pixel = usize::try_from(y.checked_mul(u64::from(self.width))?)
                .ok()?
                .checked_add(rect.x as usize)?;
            let start = pixel.checked_mul(4)?;
            normalize(
                self.format,
                &raw,
                self.pixels.get_mut(start..start + row_len)?,
            );
        }
        Some(())
    }
    pub(super) fn transfer_from_host(
        &self,
        mem: &mut PhysicalMemory,
        rect: Rect,
        offset: u64,
    ) -> Option<()> {
        if !self.is_texture_2d()
            || !rect.valid_within(self.width, self.height)
            || self.backing.is_empty()
        {
            return None;
        }
        let stride = u64::from(self.width).checked_mul(4)?;
        let row_len = usize::try_from(u64::from(rect.width).checked_mul(4)?).ok()?;
        let rows = usize::try_from(rect.height).ok()?;
        let mut raw = vec![0; row_len.checked_mul(rows)?];
        let mut writes = Vec::new();
        for row in 0..rect.height {
            let y = u64::from(rect.y.checked_add(row)?);
            let pixel = usize::try_from(y.checked_mul(u64::from(self.width))?)
                .ok()?
                .checked_add(usize::try_from(rect.x).ok()?)?;
            let start = pixel.checked_mul(4)?;
            let raw_start = usize::try_from(row).ok()?.checked_mul(row_len)?;
            encode(
                self.format,
                self.pixels.get(start..start.checked_add(row_len)?)?,
                &mut raw[raw_start..raw_start + row_len],
            );
            self.plan_writes(
                mem,
                offset.checked_add(u64::from(row).checked_mul(stride)?)?,
                raw_start,
                row_len,
                &mut writes,
            )?;
        }
        for write in writes {
            mem.write_bytes(write.addr, &raw[write.source..write.source + write.len])?;
        }
        Some(())
    }
    pub(super) fn transfer_buffer_from_host(
        &self,
        mem: &mut PhysicalMemory,
        offset: u64,
        start: usize,
        end: usize,
    ) -> Option<()> {
        if !self.is_buffer() || self.backing.is_empty() || end <= start || end > self.pixels.len() {
            return None;
        }
        let mut writes = Vec::new();
        self.plan_writes(mem, offset, start, end.checked_sub(start)?, &mut writes)?;
        for write in writes {
            mem.write_bytes(
                write.addr,
                &self.pixels[write.source..write.source + write.len],
            )?;
        }
        Some(())
    }
    fn plan_writes(
        &self,
        mem: &PhysicalMemory,
        offset: u64,
        source: usize,
        len: usize,
        writes: &mut Vec<BackingWrite>,
    ) -> Option<()> {
        let mut skip = offset;
        let mut done = 0usize;
        for entry in &self.backing {
            if skip >= u64::from(entry.len) {
                skip -= u64::from(entry.len);
                continue;
            }
            let available = usize::try_from(u64::from(entry.len) - skip).ok()?;
            let count = available.min(len - done);
            let addr = entry.addr.checked_add(skip)?;
            if !mem.contains_range(addr, count) {
                return None;
            }
            writes.push(BackingWrite {
                addr,
                source: source.checked_add(done)?,
                len: count,
            });
            done += count;
            skip = 0;
            if done == len {
                return Some(());
            }
        }
        None
    }
}
fn encode(format: u32, src: &[u8], dst: &mut [u8]) {
    for (source, target) in src.chunks_exact(4).zip(dst.chunks_exact_mut(4)) {
        match format {
            FORMAT_B8G8R8A8_UNORM => target.copy_from_slice(source),
            FORMAT_B8G8R8X8_UNORM => {
                target.copy_from_slice(&[source[0], source[1], source[2], 255])
            }
            FORMAT_A8R8G8B8_UNORM => {
                target.copy_from_slice(&[source[3], source[2], source[1], source[0]])
            }
            FORMAT_X8R8G8B8_UNORM => {
                target.copy_from_slice(&[255, source[2], source[1], source[0]])
            }
            FORMAT_R8G8B8A8_UNORM => {
                target.copy_from_slice(&[source[2], source[1], source[0], source[3]])
            }
            _ => unreachable!("format validated at resource creation"),
        }
    }
}

fn normalize(format: u32, src: &[u8], dst: &mut [u8]) {
    for (source, target) in src.chunks_exact(4).zip(dst.chunks_exact_mut(4)) {
        match format {
            FORMAT_B8G8R8A8_UNORM => target.copy_from_slice(source),
            FORMAT_B8G8R8X8_UNORM => {
                target.copy_from_slice(&[source[0], source[1], source[2], 255])
            }
            FORMAT_A8R8G8B8_UNORM => {
                target.copy_from_slice(&[source[3], source[2], source[1], source[0]])
            }
            FORMAT_X8R8G8B8_UNORM => {
                target.copy_from_slice(&[source[3], source[2], source[1], 255])
            }
            FORMAT_R8G8B8A8_UNORM => {
                target.copy_from_slice(&[source[2], source[1], source[0], source[3]])
            }
            _ => unreachable!("format validated at resource creation"),
        }
    }
}
