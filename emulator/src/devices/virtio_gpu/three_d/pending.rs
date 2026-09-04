use super::super::completion::PendingCompletion;
use super::super::fence::FenceTimeline;
use super::super::protocol::{CtrlHeader, Rect};
use super::virgl::{DepthState, DrawMaterial, DrawWork};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::devices::virtio_gpu) enum BrowserCompletion {
    Standard,
    Readback,
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
}

#[derive(Debug, Clone, Copy)]
pub(in crate::devices::virtio_gpu) struct DeferredSubmit {
    pub sequence: u32,
    pub header: CtrlHeader,
}
