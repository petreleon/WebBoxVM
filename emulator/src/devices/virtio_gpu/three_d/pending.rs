use super::super::completion::PendingCompletion;
use super::super::fence::FenceTimeline;
use super::super::protocol::{CtrlHeader, Rect};
use super::virgl::{DepthState, DrawMaterial, DrawWork};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::devices::virtio_gpu) enum BrowserCompletion {
    Standard,
    Readback,
    Resident,
}

#[derive(Debug, Clone)]
pub(in crate::devices::virtio_gpu) struct Pending3d {
    pub sequence: u32,
    pub timeline: FenceTimeline,
    pub bytes: usize,
    pub packet: Option<Vec<u8>>,
    pub completion: Option<PendingCompletion>,
    pub effect: Option<Pending3dEffect>,
    pub browser_completion: BrowserCompletion,
}

#[derive(Debug, Clone)]
pub(in crate::devices::virtio_gpu) enum Pending3dEffect {
    VirglClear {
        context_id: u32,
        generation: u32,
        resource_id: u32,
        rect: Rect,
        resident_epoch: u64,
        resident_predecessor: Option<u32>,
        bgra: [u8; 4],
    },
    VirglDraw {
        context_id: u32,
        generation: u32,
        resource_id: u32,
        depth_resource: Option<u32>,
        depth_state: Option<DepthState>,
        rect: Rect,
        clear_bgra: [u8; 4],
        material: DrawMaterial,
        vertices: Vec<u8>,
        viewport: [f32; 6],
        scissor: Option<Rect>,
    },
    VirglBatch {
        context_id: u32,
        generation: u32,
        resource_id: u32,
        rect: Rect,
        resident_epoch: u64,
        resident_predecessor: Option<u32>,
        clear_bgra: [u8; 4],
        works: Vec<DrawWork>,
    },
    VirglDepthBatch {
        context_id: u32,
        generation: u32,
        resource_id: u32,
        depth_resource: u32,
        rect: Rect,
        clear_bgra: [u8; 4],
        works: Vec<DrawWork>,
    },
    VirglResidentReadback {
        context_id: u32,
        generation: u32,
        resource_id: u32,
        producer_sequence: u32,
        source_rect: Rect,
        transfer_rect: Rect,
        transfer_offset: u64,
    },
}

impl Pending3dEffect {
    pub(in crate::devices::virtio_gpu) fn color_target(&self) -> Option<(u32, Rect)> {
        match self {
            Self::VirglClear { resource_id, rect, .. }
            | Self::VirglDraw { resource_id, rect, .. }
            | Self::VirglBatch { resource_id, rect, .. }
            | Self::VirglDepthBatch { resource_id, rect, .. } => Some((*resource_id, *rect)),
            Self::VirglResidentReadback { .. } => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(in crate::devices::virtio_gpu) struct DeferredSubmit {
    pub sequence: u32,
    pub header: CtrlHeader,
}
