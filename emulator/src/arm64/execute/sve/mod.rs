use super::{Instr, Opcode};
use crate::arm64::Armv8Cpu;
use crate::arm64::helpers::read_base;
use crate::arm64::mmu::{Fault, translate, translate_write};
use crate::bus::SystemBus;

mod count;
mod helpers;
mod memory_b;
mod memory_contiguous;
mod memory_d;
mod memory_register;
mod predicate;
mod predicate_bytes;
mod vector;

pub(super) use count::sve_pred_count;
use helpers::*;
pub(super) use memory_b::exec_sve_st1b;
pub(super) use memory_contiguous::exec_sve_contiguous_load;
pub(super) use memory_d::exec_sve_ld1_st1_d;
pub(super) use memory_register::exec_sve_ldr_str;
pub(super) use predicate::{exec_sve_pred_logical, exec_sve_ptest, exec_sve_ptrue};
use predicate_bytes::*;
pub(super) use vector::{
    exec_sve_dup_gpr, exec_sve_int_binary, exec_sve_logical_binary, exec_sve_movprfx, exec_sve_sel,
};
