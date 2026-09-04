use super::super::protocol::Rect;
use super::super::resource::GpuResource;
use crate::memory::PhysicalMemory;

const READBACK_FORMAT_BGRA8: u32 = 1;
const READBACK_FORMAT_RGBA8: u32 = 2;

impl GpuResource {
    pub(in crate::devices::virtio_gpu) fn transfer_gpu_readback_from_host(
        &self, mem: &mut PhysicalMemory, rect: Rect, offset: u64, format: u32, pixels: &[u8],
    ) -> Option<()> {
        if !matches!(format, READBACK_FORMAT_BGRA8 | READBACK_FORMAT_RGBA8)
            || !self.is_texture_2d() || !rect.valid_within(self.width, self.height) || self.backing.is_empty() {
            return None;
        }
        let stride = u64::from(self.width).checked_mul(4)?;
        let row_len = usize::try_from(u64::from(rect.width).checked_mul(4)?).ok()?;
        let rows = usize::try_from(rect.height).ok()?;
        if pixels.len() != row_len.checked_mul(rows)? { return None; }
        let mut bgra = pixels.to_vec();
        if format == READBACK_FORMAT_RGBA8 {
            for pixel in bgra.chunks_exact_mut(4) { pixel.swap(0, 2); }
        }
        let mut raw = vec![0; bgra.len()];
        let mut writes = Vec::new();
        for row in 0..rect.height {
            let start = usize::try_from(row).ok()?.checked_mul(row_len)?;
            super::encode(self.format, &bgra[start..start + row_len], &mut raw[start..start + row_len]);
            self.plan_writes(mem, offset.checked_add(u64::from(row).checked_mul(stride)?)?, start, row_len, &mut writes)?;
        }
        for write in writes {
            mem.write_bytes(write.addr, &raw[write.source..write.source + write.len])?;
        }
        Some(())
    }
}
