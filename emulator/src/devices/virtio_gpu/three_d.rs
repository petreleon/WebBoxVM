use super::completion::PendingCompletion;
use super::protocol::*;
use super::{MAX_PENDING_3D_BYTES, MAX_PENDING_3D_SUBMITS, VirtioGpu};

mod context;
pub(super) mod packet;

use packet::{PACKET_HEADER_BYTES, VERTEX_FLOATS, decode_submit};

pub(super) const CAPSET_ID: u32 = 7;
pub(super) const CAPSET_VERSION: u32 = 1;
pub(super) const CAPSET_SIZE: u32 = 32;
pub(super) const MAX_3D_DIMENSION: u32 = 8192;
pub(super) const MAX_3D_VERTICES: u32 = 4096;
pub(super) const MAX_3D_INDICES: u32 = 12288;

#[derive(Debug, Clone)]
pub(super) struct Pending3d {
    pub sequence: u32,
    pub bytes: usize,
    pub packet: Option<Vec<u8>>,
    pub completion: Option<PendingCompletion>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct DeferredSubmit {
    pub sequence: u32,
    pub header: CtrlHeader,
}

impl VirtioGpu {
    pub(super) fn capset_info_response(&self, header: CtrlHeader, input: &[u8]) -> Vec<u8> {
        if input.len() != 32 || read_u32(input, 24) != Some(0) || read_u32(input, 28) != Some(0) {
            return header.encode(RESP_ERR_INVALID_PARAMETER);
        }
        let mut out = header.encode(RESP_OK_CAPSET_INFO);
        for value in [CAPSET_ID, CAPSET_VERSION, CAPSET_SIZE, 0] {
            push_u32(&mut out, value);
        }
        out
    }

    pub(super) fn capset_response(&self, header: CtrlHeader, input: &[u8]) -> Vec<u8> {
        if input.len() != 32
            || read_u32(input, 24) != Some(CAPSET_ID)
            || read_u32(input, 28) != Some(CAPSET_VERSION)
        {
            return header.encode(RESP_ERR_INVALID_PARAMETER);
        }
        let mut out = header.encode(RESP_OK_CAPSET);
        out.extend_from_slice(b"WBG3");
        for value in [
            CAPSET_VERSION,
            MAX_3D_DIMENSION,
            MAX_3D_VERTICES,
            MAX_3D_INDICES,
            PACKET_HEADER_BYTES as u32,
            VERTEX_FLOATS as u32,
            2,
        ] {
            push_u32(&mut out, value);
        }
        out
    }

    pub(super) fn submit_3d(
        &mut self,
        header: CtrlHeader,
        input: &[u8],
    ) -> Result<DeferredSubmit, u32> {
        if self.contexts.get(&header.ctx_id) != Some(&CAPSET_ID) {
            return Err(RESP_ERR_INVALID_CONTEXT_ID);
        }
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
        let mut packet = packet.to_vec();
        packet[12..16].copy_from_slice(&sequence.to_le_bytes());
        self.pending_3d_bytes += packet.len();
        self.pending_3d.push(Pending3d {
            sequence,
            bytes: packet.len(),
            packet: Some(packet),
            completion: None,
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
