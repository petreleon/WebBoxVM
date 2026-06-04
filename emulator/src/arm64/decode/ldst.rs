//! Load/Store instruction decoders.

mod atomic;
mod exclusive;
mod literal_pair;
mod scalar;
mod simd;

use super::{Instr, Opcode};

pub(super) use atomic::*;
pub(super) use exclusive::*;
pub(super) use literal_pair::*;
pub(super) use scalar::*;
pub(super) use simd::*;
