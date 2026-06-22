//! Trace sinks, counters, and guest-state inspection helpers.
//!
//! Observability helpers may inspect architecture and platform state, but they
//! must not own the run loop or depend on runtime internals.

use crate::arch::arm64::{Armv8Cpu, Instr, Opcode, cond_taken, translate_read_only};
use crate::platform::virt::SystemBus;

mod debug_dump;
mod trace_filters;
mod trace_hotspots;
mod trace_memory;
mod trace_paths;
mod trace_stack;
mod trace_state;
mod trace_syscalls;
mod trace_syscalls_exec;
mod trace_syscalls_write;

pub(crate) use debug_dump::dump_breakpoint_context;
pub(crate) use trace_filters::*;
pub(crate) use trace_hotspots::*;
pub(crate) use trace_memory::*;
pub(crate) use trace_paths::*;
pub(crate) use trace_stack::*;
pub(crate) use trace_state::{TraceOptions, TraceState, TraceSyscall};
pub(crate) use trace_syscalls::*;
pub(crate) use trace_syscalls_exec::*;
pub(crate) use trace_syscalls_write::*;
