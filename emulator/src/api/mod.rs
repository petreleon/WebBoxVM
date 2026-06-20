//! Public API facade and boundary primitives.

mod access;
mod address;
mod irq;
mod vm;

pub use access::{AccessWidth, InvalidAccessWidth};
pub use address::{PhysAddr, PhysRange, VirtAddr};
pub use irq::IrqId;
pub use vm::{VmConfig, VmConfigError, VmEvent, VmHandle, VmMetrics};
