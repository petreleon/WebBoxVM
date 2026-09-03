use super::super::MAX_VIRGL_FRAGMENT_SAMPLERS;
use crate::devices::virtio_gpu::protocol::Rect;
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy, Debug)]
pub(super) struct Rasterizer {
    pub scissor: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::devices::virtio_gpu) enum SamplerAddressMode {
    ClampToEdge,
    Repeat,
}

#[derive(Clone, Copy, Debug)]
pub(in crate::devices::virtio_gpu::three_d::virgl) struct SamplerState {
    pub address_mode: SamplerAddressMode,
}

#[derive(Clone, Copy)]
pub(in crate::devices::virtio_gpu::three_d::virgl) struct SampledResource {
    pub resource: u32,
    pub address_mode: SamplerAddressMode,
}

#[derive(Clone, Copy, Debug)]
pub(in crate::devices::virtio_gpu::three_d::virgl) struct Viewport {
    scale: [f32; 3],
    translate: [f32; 3],
}

impl Viewport {
    pub(in crate::devices::virtio_gpu::three_d::virgl) fn new(
        scale: [f32; 3],
        translate: [f32; 3],
    ) -> Option<Self> {
        let valid = scale.into_iter().chain(translate).all(f32::is_finite)
            && scale[0] > 0.0
            && scale[1] > 0.0
            && scale[2] >= 0.0;
        valid.then_some(Self { scale, translate })
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn valid_within(
        self,
        width: u32,
        height: u32,
    ) -> bool {
        let [x, y, z] = self.scale;
        let [tx, ty, tz] = self.translate;
        tx - x >= 0.0
            && tx + x <= width as f32
            && ty - y >= 0.0
            && ty + y <= height as f32
            && tz - z >= 0.0
            && tz + z <= 1.0
    }

    pub(in crate::devices::virtio_gpu::three_d::virgl) fn values(self) -> [f32; 6] {
        let Self { scale, translate } = self;
        [
            scale[0],
            scale[1],
            scale[2],
            translate[0],
            translate[1],
            translate[2],
        ]
    }
}

#[derive(Clone, Debug)]
pub(super) struct PipelineState {
    pub(super) blend_states: HashSet<u32>,
    pub(super) bound_blend_state: Option<u32>,
    pub(super) rasterizers: HashMap<u32, Rasterizer>,
    pub(super) bound_rasterizer: Option<u32>,
    pub(super) viewport: Option<Viewport>,
    pub(super) scissor: Option<Rect>,
    pub(super) sampler_views: HashMap<u32, u32>,
    pub(super) sampler_states: HashMap<u32, SamplerState>,
    pub(super) bound_sampler_views: [Option<u32>; MAX_VIRGL_FRAGMENT_SAMPLERS],
    pub(super) bound_sampler_states: [Option<u32>; MAX_VIRGL_FRAGMENT_SAMPLERS],
}

impl PipelineState {
    pub(super) fn new() -> Self {
        Self {
            blend_states: HashSet::new(),
            bound_blend_state: None,
            rasterizers: HashMap::new(),
            bound_rasterizer: None,
            viewport: None,
            scissor: None,
            sampler_views: HashMap::new(),
            sampler_states: HashMap::new(),
            bound_sampler_views: [None; MAX_VIRGL_FRAGMENT_SAMPLERS],
            bound_sampler_states: [None; MAX_VIRGL_FRAGMENT_SAMPLERS],
        }
    }
}
