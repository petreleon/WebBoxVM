pub const PSTATE_N_BIT: u32 = 31;
pub const PSTATE_Z_BIT: u32 = 30;
pub const PSTATE_C_BIT: u32 = 29;
pub const PSTATE_V_BIT: u32 = 28;
pub const PSTATE_NZCV_MASK: u64 = 0xF000_0000;
pub const PSTATE_EL_SHIFT: u32 = 2;
pub const PSTATE_EL_MASK: u64 = 3 << PSTATE_EL_SHIFT;
pub const PSTATE_I_BIT: u32 = 7;
pub const PSTATE_DAIF_MASK: u64 = 0x3C0;
pub const SPSR_M_MASK: u64 = 0xF << 6;

pub const NUM_GENERAL_REGISTERS: u8 = 31;
pub const ZERO_REGISTER_INDEX: u8 = 31;
pub const SP_REGISTER_INDEX: u8 = 31;
pub const LINK_REGISTER_INDEX: u8 = 30;
pub const MAX_EL: u8 = 3;

pub const SIGN_BIT_64: u32 = 63;
pub const SIGN_BIT_32: u32 = 31;
pub const WORD_MASK: u64 = 0xFFFF_FFFF;
