use super::{BlobMemory, BlobResource};
use crate::memory::PhysicalMemory;

impl BlobResource {
    pub(in crate::devices::virtio_gpu) fn transfer_shadow_to_host(
        &mut self,
        mem: &PhysicalMemory,
        backing_offset: u64,
        start: usize,
        end: usize,
    ) -> Option<()> {
        self.shadow_range(start, end)?;
        let host = self.host.as_mut()?;
        read_backing(
            mem,
            &self.backing,
            backing_offset,
            host.bytes.get_mut(start..end)?,
        )
    }

    pub(in crate::devices::virtio_gpu) fn transfer_shadow_from_host(
        &self,
        mem: &mut PhysicalMemory,
        backing_offset: u64,
        start: usize,
        end: usize,
    ) -> Option<()> {
        self.shadow_range(start, end)?;
        let host = self.host.as_ref()?;
        write_backing(
            mem,
            &self.backing,
            backing_offset,
            host.bytes.get(start..end)?,
        )
    }

    fn shadow_range(&self, start: usize, end: usize) -> Option<()> {
        (self.memory == BlobMemory::Host3dGuest
            && !self.backing.is_empty()
            && start < end
            && end <= self.size)
            .then_some(())
    }
}

fn read_backing(
    mem: &PhysicalMemory,
    backing: &[crate::devices::virtio_gpu::BackingEntry],
    offset: u64,
    dst: &mut [u8],
) -> Option<()> {
    for_each_backing(backing, offset, dst.len(), |addr, start, len| {
        mem.read_bytes(addr, &mut dst[start..start + len])
    })
}

fn write_backing(
    mem: &mut PhysicalMemory,
    backing: &[crate::devices::virtio_gpu::BackingEntry],
    offset: u64,
    src: &[u8],
) -> Option<()> {
    for_each_backing(backing, offset, src.len(), |addr, start, len| {
        mem.write_bytes(addr, &src[start..start + len])
    })
}

fn for_each_backing<F>(
    backing: &[crate::devices::virtio_gpu::BackingEntry],
    offset: u64,
    len: usize,
    mut copy: F,
) -> Option<()>
where
    F: FnMut(u64, usize, usize) -> Option<()>,
{
    let mut skip = offset;
    let mut done = 0usize;
    for entry in backing {
        if skip >= u64::from(entry.len) {
            skip -= u64::from(entry.len);
            continue;
        }
        let available = usize::try_from(u64::from(entry.len) - skip).ok()?;
        let count = available.min(len.checked_sub(done)?);
        copy(entry.addr.checked_add(skip)?, done, count)?;
        done = done.checked_add(count)?;
        skip = 0;
        if done == len {
            return Some(());
        }
    }
    None
}
