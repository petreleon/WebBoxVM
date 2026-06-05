//! Data-processing instruction decoders: ADR, add/sub, move, logical, bitfield, DP, condsel, multiply.

mod addressing;
mod bitfield;
mod condition_mul;
mod cssc;
mod logical;
mod moves;
mod pointer_subtract;
mod register;
mod sources;

use super::{Instr, Opcode};
use crate::arm64::bitmask_imm::decode_bitmask_imm;

pub(super) use addressing::*;
pub(super) use bitfield::*;
pub(super) use condition_mul::*;
pub(super) use cssc::*;
pub(super) use logical::*;
pub(super) use moves::*;
pub(super) use register::*;
pub(super) use sources::*;
