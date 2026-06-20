//! Load/Store instruction execution.

use super::{Instr, Opcode, branch_target};
use crate::arch::arm64::Armv8Cpu;
use crate::arch::arm64::helpers::{read_base, read_reg, write_reg, write_reg_sp};
use crate::arch::arm64::mmu::{Fault, translate, translate_write};
use crate::constants::*;
use crate::platform::virt::SystemBus;
use std::env;

const SIMD_MULTI_POST_INDEX: u8 = 0xFE;

mod address;
mod atomic;
mod exclusive;
mod guest;
mod guest_trace;
mod guest_translate;
mod mops;
mod mte;
mod pair;
mod scalar;
mod simd_structure;
mod simd_structure_lane;

pub(super) use atomic::exec_atomic;
pub(super) use exclusive::exec_exclusive;
pub(super) use mops::exec_mops;
pub(super) use mte::{exec_mte_gpr, exec_mte_mem};
pub(super) use pair::exec_ldp_stp;
pub(super) use scalar::{exec_ldr_lit, exec_ldr_str};

use address::*;
use guest::*;
use guest_trace::*;
use guest_translate::*;
use simd_structure::*;
use simd_structure_lane::*;
