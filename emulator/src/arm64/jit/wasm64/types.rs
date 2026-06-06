use crate::arm64::Opcode;
use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmBlockModule {
    pub start_pc: u64,
    pub start_pa: u64,
    pub exit_pc: u64,
    pub alternate_exit_pc: u64,
    pub dynamic_exit: bool,
    pub guest_instr_count: usize,
    pub raw_hash: u64,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WasmJitError {
    BlockDiscovery(&'static str),
    EmptyBlock,
    UnsupportedFirstOpcode(Opcode),
}

impl fmt::Display for WasmJitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BlockDiscovery(err) => write!(f, "block discovery failed: {err}"),
            Self::EmptyBlock => write!(f, "block has no instructions"),
            Self::UnsupportedFirstOpcode(op) => {
                write!(
                    f,
                    "first opcode is not wasm-jittable: {} ({})",
                    op.name(),
                    op.id()
                )
            }
        }
    }
}

impl std::error::Error for WasmJitError {}
