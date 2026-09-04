mod create;
mod map;

use super::BackingEntry;

#[derive(Debug, Clone)]
pub(super) struct BlobResource {
    pub size: usize,
    pub backing: Vec<BackingEntry>,
    host: Option<HostVisibleBlob>,
}

#[derive(Debug, Clone)]
struct HostVisibleBlob {
    bytes: Vec<u8>,
    mapped_offset: Option<u64>,
    owner_context: u32,
}

impl BlobResource {
    pub(super) fn guest(size: usize, backing: Vec<BackingEntry>) -> Self {
        Self {
            size,
            backing,
            host: None,
        }
    }

    pub(super) fn host_visible(size: usize, owner_context: u32) -> Option<Self> {
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(size).ok()?;
        bytes.resize(size, 0);
        Some(Self {
            size,
            backing: Vec::new(),
            host: Some(HostVisibleBlob {
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
