use super::completion::PendingCompletion;
use super::fence::FenceTimeline;
use super::protocol::*;
use super::{MAX_PENDING_3D_BYTES, MAX_PENDING_3D_SUBMITS, VirtioGpu};

mod capset;
mod context;
pub(super) mod packet;
mod transfer;
mod virgl;

pub(super) use capset::{CAPSET_COUNT, VIRGL_CAPSET_ID, VIRGL2_CAPSET_ID};
use packet::decode_submit;
use virgl::{DepthState, DrawMaterial, DrawWork};
pub(in crate::devices::virtio_gpu) use virgl::VirglContext;
#[cfg(test)]
pub(in crate::devices::virtio_gpu) use virgl::{ShaderKind, ShaderProgram};

pub(super) const CAPSET_ID: u32 = 7;
pub(super) const CAPSET_VERSION: u32 = 1;
pub(super) const CAPSET_SIZE: u32 = 32;
pub(super) const MAX_3D_DIMENSION: u32 = 8192;
pub(super) const MAX_3D_VERTICES: u32 = 4096;
pub(super) const MAX_3D_INDICES: u32 = 12288;

#[derive(Debug, Clone)]
pub(super) struct Pending3d {
    pub sequence: u32,
    pub timeline: FenceTimeline,
    pub bytes: usize,
    pub packet: Option<Vec<u8>>,
    pub completion: Option<PendingCompletion>,
    pub effect: Option<Pending3dEffect>,
    pub webgpu_readback: bool,
}

#[derive(Debug, Clone)]
pub(super) enum Pending3dEffect {
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
pub(super) struct DeferredSubmit {
    pub sequence: u32,
    pub header: CtrlHeader,
}

impl VirtioGpu {
    pub(super) fn capset_info_response(&self, header: CtrlHeader, input: &[u8]) -> Vec<u8> {
        if input.len() != 32 || read_u32(input, 28) != Some(0) {
            return header.encode(RESP_ERR_INVALID_PARAMETER);
        }
        let Some(capset) = capset::by_index(read_u32(input, 24).unwrap_or_default()) else {
            return header.encode(RESP_ERR_INVALID_PARAMETER);
        };
        let mut out = header.encode(RESP_OK_CAPSET_INFO);
        for value in [capset.id, capset.version, capset.size, 0] {
            push_u32(&mut out, value);
        }
        out
    }

    pub(super) fn capset_response(&self, header: CtrlHeader, input: &[u8]) -> Vec<u8> {
        if input.len() != 32 {
            return header.encode(RESP_ERR_INVALID_PARAMETER);
        }
        let (Some(id), Some(version)) = (read_u32(input, 24), read_u32(input, 28)) else {
            return header.encode(RESP_ERR_INVALID_PARAMETER);
        };
        let Some(data) = capset::data(id, version) else {
            return header.encode(RESP_ERR_INVALID_PARAMETER);
        };
        let mut out = header.encode(RESP_OK_CAPSET);
        out.extend_from_slice(&data);
        out
    }

    pub(super) fn submit_3d(
        &mut self,
        header: CtrlHeader,
        input: &[u8],
    ) -> Result<Option<DeferredSubmit>, u32> {
        match self.contexts.get(&header.ctx_id) {
            Some(&CAPSET_ID) => return self.submit_wbg3(header, input).map(Some),
            Some(&VIRGL_CAPSET_ID) | Some(&VIRGL2_CAPSET_ID) => {
                return self.submit_virgl(header, input)
            }
            _ => return Err(RESP_ERR_INVALID_CONTEXT_ID),
        }
    }

    fn submit_wbg3(&mut self, header: CtrlHeader, input: &[u8]) -> Result<DeferredSubmit, u32> {
        let packet = decode_submit(input).ok_or(RESP_ERR_INVALID_PARAMETER)?;
        if self.pending_3d.len() >= MAX_PENDING_3D_SUBMITS
            || self
                .pending_3d_bytes
                .checked_add(packet.len())
                .is_none_or(|total| total > MAX_PENDING_3D_BYTES)
        {
            return Err(RESP_ERR_OUT_OF_MEMORY);
        }
        let sequence = self.allocate_3d_sequence().ok_or(RESP_ERR_OUT_OF_MEMORY)?;
        let timeline = self.fence_timeline(header);
        let mut packet = packet.to_vec();
        packet[12..16].copy_from_slice(&sequence.to_le_bytes());
        self.pending_3d_bytes += packet.len();
        self.pending_3d.push(Pending3d {
            sequence,
            timeline,
            bytes: packet.len(),
            packet: Some(packet),
            completion: None,
            effect: None,
            webgpu_readback: false,
        });
        Ok(DeferredSubmit { sequence, header })
    }

    pub fn take_3d_update(&mut self) -> Vec<u8> {
        self.pending_3d
            .iter_mut()
            .find_map(|pending| pending.packet.take())
            .unwrap_or_default()
    }

    fn allocate_3d_sequence(&mut self) -> Option<u32> {
        for _ in 0..=self.pending_3d.len() {
            let sequence = self.next_3d_sequence.max(1);
            self.next_3d_sequence = sequence.wrapping_add(1).max(1);
            if !self
                .pending_3d
                .iter()
                .any(|pending| pending.sequence == sequence)
            {
                return Some(sequence);
            }
        }
        None
    }
}
