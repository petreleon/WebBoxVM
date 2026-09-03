pub(super) const CTRL_HEADER_LEN: usize = 24;

pub(super) const CMD_GET_DISPLAY_INFO: u32 = 0x0100;
pub(super) const CMD_RESOURCE_CREATE_2D: u32 = 0x0101;
pub(super) const CMD_RESOURCE_UNREF: u32 = 0x0102;
pub(super) const CMD_SET_SCANOUT: u32 = 0x0103;
pub(super) const CMD_RESOURCE_FLUSH: u32 = 0x0104;
pub(super) const CMD_TRANSFER_TO_HOST_2D: u32 = 0x0105;
pub(super) const CMD_RESOURCE_ATTACH_BACKING: u32 = 0x0106;
pub(super) const CMD_RESOURCE_DETACH_BACKING: u32 = 0x0107;
pub(super) const CMD_GET_CAPSET_INFO: u32 = 0x0108;
pub(super) const CMD_GET_CAPSET: u32 = 0x0109;
pub(super) const CMD_CTX_CREATE: u32 = 0x0200;
pub(super) const CMD_CTX_DESTROY: u32 = 0x0201;
pub(super) const CMD_CTX_ATTACH_RESOURCE: u32 = 0x0202;
pub(super) const CMD_CTX_DETACH_RESOURCE: u32 = 0x0203;
pub(super) const CMD_RESOURCE_CREATE_3D: u32 = 0x0204;
pub(super) const CMD_TRANSFER_TO_HOST_3D: u32 = 0x0205;
pub(super) const CMD_TRANSFER_FROM_HOST_3D: u32 = 0x0206;
pub(super) const CMD_SUBMIT_3D: u32 = 0x0207;

pub(super) const RESP_OK_NODATA: u32 = 0x1100;
pub(super) const RESP_OK_DISPLAY_INFO: u32 = 0x1101;
pub(super) const RESP_OK_CAPSET_INFO: u32 = 0x1102;
pub(super) const RESP_OK_CAPSET: u32 = 0x1103;
pub(super) const RESP_ERR_UNSPEC: u32 = 0x1200;
pub(super) const RESP_ERR_OUT_OF_MEMORY: u32 = 0x1201;
pub(super) const RESP_ERR_INVALID_SCANOUT_ID: u32 = 0x1202;
pub(super) const RESP_ERR_INVALID_RESOURCE_ID: u32 = 0x1203;
pub(super) const RESP_ERR_INVALID_CONTEXT_ID: u32 = 0x1204;
pub(super) const RESP_ERR_INVALID_PARAMETER: u32 = 0x1205;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct CtrlHeader {
    pub command_type: u32,
    pub flags: u32,
    pub fence_id: u64,
    pub ctx_id: u32,
    pub ring_idx: u32,
}

impl CtrlHeader {
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        Some(Self {
            command_type: read_u32(bytes, 0)?,
            flags: read_u32(bytes, 4)?,
            fence_id: read_u64(bytes, 8)?,
            ctx_id: read_u32(bytes, 16)?,
            ring_idx: read_u32(bytes, 20)?,
        })
    }

    pub fn encode(self, response_type: u32) -> Vec<u8> {
        let mut out = Vec::with_capacity(CTRL_HEADER_LEN);
        push_u32(&mut out, response_type);
        push_u32(&mut out, self.flags);
        push_u64(&mut out, self.fence_id);
        push_u32(&mut out, self.ctx_id);
        push_u32(&mut out, self.ring_idx);
        out
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Rect {
    pub fn decode(bytes: &[u8], offset: usize) -> Option<Self> {
        Some(Self {
            x: read_u32(bytes, offset)?,
            y: read_u32(bytes, offset + 4)?,
            width: read_u32(bytes, offset + 8)?,
            height: read_u32(bytes, offset + 12)?,
        })
    }

    pub fn valid_within(self, width: u32, height: u32) -> bool {
        self.width != 0
            && self.height != 0
            && self
                .x
                .checked_add(self.width)
                .is_some_and(|end| end <= width)
            && self
                .y
                .checked_add(self.height)
                .is_some_and(|end| end <= height)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Box3d {
    x: u32,
    y: u32,
    z: u32,
    width: u32,
    height: u32,
    depth: u32,
}

impl Box3d {
    pub fn decode(bytes: &[u8], offset: usize) -> Option<Self> {
        Some(Self {
            x: read_u32(bytes, offset)?,
            y: read_u32(bytes, offset + 4)?,
            z: read_u32(bytes, offset + 8)?,
            width: read_u32(bytes, offset + 12)?,
            height: read_u32(bytes, offset + 16)?,
            depth: read_u32(bytes, offset + 20)?,
        })
    }

    pub fn flat_rect(self) -> Option<Rect> {
        (self.z == 0 && self.depth == 1).then_some(Rect {
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct BackingEntry {
    pub addr: u64,
    pub len: u32,
}

pub(super) fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

pub(super) fn read_u64(bytes: &[u8], offset: usize) -> Option<u64> {
    Some(u64::from_le_bytes(
        bytes.get(offset..offset + 8)?.try_into().ok()?,
    ))
}

pub(super) fn push_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

pub(super) fn push_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}
