mod arithmetic;
mod compare_select;
mod convert;
mod helpers;
mod type_convert;
mod unary_rounding;

use super::*;
use helpers::*;

#[derive(Clone, Copy)]
pub(super) struct FpFields {
    pub(super) ftype: u8,
    pub(super) rd: u8,
    pub(super) rn: u8,
    pub(super) rm: u8,
    pub(super) size: u8,
}

macro_rules! try_fp_stage {
    ($stage:expr) => {
        match $stage {
            DecodeStep::Hit(instr) => return Some(instr),
            DecodeStep::Reject => return None,
            DecodeStep::Miss => {}
        }
    };
}

pub(super) fn decode_fp_scalar(raw: u32) -> Option<Instr> {
    let ftype = ((raw >> 22) & 0x3) as u8;
    let fields = FpFields {
        ftype,
        rd: (raw & 0x1F) as u8,
        rn: ((raw >> 5) & 0x1F) as u8,
        rm: ((raw >> 16) & 0x1F) as u8,
        size: 0,
    };

    try_fp_stage!(type_convert::decode(raw, fields));

    let size = fp_scalar_type_size(ftype)?;
    let fields = FpFields { size, ..fields };

    try_fp_stage!(arithmetic::decode(raw, fields));
    try_fp_stage!(unary_rounding::decode(raw, fields));
    try_fp_stage!(convert::decode(raw, fields));
    try_fp_stage!(compare_select::decode(raw, fields));

    None
}
