use super::protocol::{CtrlHeader, RESP_ERR_INVALID_PARAMETER};
use super::{VIRTIO_GPU_F_CONTEXT_INIT, VirtioGpu};

pub(super) const FLAG_FENCE: u32 = 1;
pub(super) const FLAG_INFO_RING_IDX: u32 = 1 << 1;
const KNOWN_FLAGS: u32 = FLAG_FENCE | FLAG_INFO_RING_IDX;

/// A guest fence timeline: context-local unless the context ID is zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FenceTimeline {
    context_id: u32,
    context_generation: u32,
    ring_index: u8,
}

impl VirtioGpu {
    pub(super) fn fence_timeline(&self, header: CtrlHeader) -> FenceTimeline {
        FenceTimeline {
            context_id: header.ctx_id,
            context_generation: self
                .context_generations
                .get(&header.ctx_id)
                .copied()
                .unwrap_or_default(),
            ring_index: (header.ring_idx & u32::from(u8::MAX)) as u8,
        }
    }

    /// Reject undefined fence bits and malformed `ring_idx` padding early.
    pub(super) fn validate_fence_header(&self, header: CtrlHeader) -> Result<(), u32> {
        if header.flags & !KNOWN_FLAGS != 0 || header.ring_idx & !u32::from(u8::MAX) != 0 {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        let includes_ring = header.flags & FLAG_INFO_RING_IDX != 0;
        if includes_ring
            && (header.flags & FLAG_FENCE == 0
                || !self.feature_enabled(VIRTIO_GPU_F_CONTEXT_INIT))
        {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        if !includes_ring && header.ring_idx != 0 {
            return Err(RESP_ERR_INVALID_PARAMETER);
        }
        Ok(())
    }
}
