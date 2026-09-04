mod create;
mod map;
mod transfer;

use super::BackingEntry;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BlobMemory {
    Guest,
    Host3d,
    Host3dGuest,
}

#[derive(Debug, Clone)]
pub(super) struct BlobResource {
    pub size: usize,
    pub backing: Vec<BackingEntry>,
    pub(super) memory: BlobMemory,
    pub(super) host: Option<HostBlob>,
}

#[derive(Debug, Clone)]
pub(super) struct HostBlob {
    pub(super) bytes: Vec<u8>,
    pub(super) mapped_offset: Option<u64>,
    pub(super) owner_context: u32,
}

impl BlobResource {
    pub(super) fn guest(size: usize, backing: Vec<BackingEntry>) -> Self {
        Self {
            size,
            backing,
            memory: BlobMemory::Guest,
            host: None,
        }
    }

    pub(super) fn host_only(size: usize, owner_context: u32) -> Option<Self> {
        Self::host(size, owner_context, BlobMemory::Host3d, Vec::new())
    }

    pub(super) fn host_shadowed(
        size: usize,
        owner_context: u32,
        backing: Vec<BackingEntry>,
    ) -> Option<Self> {
        Self::host(size, owner_context, BlobMemory::Host3dGuest, backing)
    }

    fn host(
        size: usize,
        owner_context: u32,
        memory: BlobMemory,
        backing: Vec<BackingEntry>,
    ) -> Option<Self> {
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(size).ok()?;
        bytes.resize(size, 0);
        Some(Self {
            size,
            backing,
            memory,
            host: Some(HostBlob {
                bytes,
                mapped_offset: None,
                owner_context,
            }),
        })
    }

    pub(super) fn mapped_range(&self) -> Option<(u64, usize)> {
        self.host
            .as_ref()?
            .mapped_offset
            .map(|offset| (offset, self.size))
    }
}
