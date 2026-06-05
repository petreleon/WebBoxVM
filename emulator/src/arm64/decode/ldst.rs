//! Load/Store instruction decoders.

mod atomic;
mod auth;
mod exclusive;
mod literal_pair;
mod rcpc;
mod rcpc_simd;
mod scalar;
mod scalar_prfm;
mod simd;

use super::{Instr, Opcode};

pub(super) use atomic::*;
pub(super) use auth::*;
pub(super) use exclusive::*;
pub(super) use literal_pair::*;
pub(super) use rcpc::*;
pub(super) use rcpc_simd::*;
pub(super) use scalar::*;
pub(super) use simd::*;
