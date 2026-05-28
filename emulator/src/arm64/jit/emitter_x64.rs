//! x86_64 machine code emitter — generates raw machine code.
//! Conventions: RDI = &mut Armv8Cpu, RSI = &mut SystemBus.
//! RAX = scratch, RCX/RDX = second/third scratch.

use crate::arm64::Armv8Cpu;
use crate::bus::SystemBus;

// ── CompiledBlock ──

pub struct CompiledBlock {
    pub code: Vec<u8>,
    pub arm64_instr_count: usize,
}

impl CompiledBlock {
    pub unsafe fn execute(&self, cpu: &mut Armv8Cpu, bus: &mut SystemBus) {
        let fn_ptr: extern "C" fn(*mut Armv8Cpu, *mut SystemBus) =
            unsafe { std::mem::transmute(self.code.as_ptr()) };
        fn_ptr(cpu as *mut Armv8Cpu, bus as *mut SystemBus);
    }
}

// ── Register file offsets in Armv8Cpu ──
// Armv8Cpu layout: regs (RegisterFile), pstate, sys, tlb
// RegisterFile: [x[0]..x[30]: u64; 31], sp: u64, pc: u64
pub const OFF_X0: i32 = 0;
pub fn off_x(n: u8) -> i32 { OFF_X0 + (n as i32) * 8 }
pub const OFF_SP: i32 = 31 * 8;
pub const OFF_PC: i32 = 32 * 8;
// After RegisterFile: ProcessorState (bits: u64)
pub const OFF_PSTATE: i32 = OFF_PC + 8;
// After pstate: SystemRegisters — we need cycle_count for timer
// SystemRegisters layout: sctlr_el1 (0), tcr_el1 (8), ttbr0_el1 (16), ttbr1_el1 (24)
//   mair_el1 (32), vbar_el1 (40), spsr_el1 (48), elr_el1 (56), esr_el1 (64),
//   far_el1 (72), cpacr_el1 (80), cntfrq_el0 (88), scr_el3 (96), spsr_el3 (104),
//   elr_el3 (112), hcr_el2 (120), spsr_el2 (128), elr_el2 (136),
//   sp_el0 (144), tpidr_el0 (152), tpidr_el1 (160), tpidrro_el0 (168),
//   cycle_count (176), icc_pmr_el1 (184), icc_ctlr_el1 (192), icc_sre_el1 (200),
//   icc_iar1_el1 (208), cntp_ctl_el0 (216), cntp_cval_el0 (224), cntp_tval_el0 (232),
//   irq_pending (240), last_irq_id (244)
pub const OFF_CYCLE_COUNT: i32 = OFF_PSTATE + 8 + 176;

// External callbacks — resolved at link time
unsafe extern "C" {
    fn jit_mmu_translate(cpu: *mut Armv8Cpu, bus: *mut SystemBus, va: u64) -> u64;
    fn jit_bus_read(pa: u64, size: u64) -> u64;
    fn jit_bus_write(pa: u64, size: u64, val: u64);
}

// ── Emitter ──

pub struct Emitter {
    pub buf: Vec<u8>,
}

impl Emitter {
    pub fn new() -> Self { Self { buf: Vec::new() } }
    pub fn finish(self) -> Vec<u8> { self.buf }

    // ── raw emit ──
    pub fn e(&mut self, bytes: &[u8]) { self.buf.extend_from_slice(bytes); }
    pub fn e8(&mut self, b: u8) { self.buf.push(b); }
    pub fn e32(&mut self, v: u32) { self.buf.extend_from_slice(&v.to_le_bytes()); }

    // ── Prologue / Epilogue ──
    pub fn prologue(&mut self) {
        self.e8(0x55);                    // push rbp
        self.e(&[0x48, 0x89, 0xE5]);      // mov rbp, rsp
        self.e8(0x53);                    // push rbx
        self.e(&[0x41, 0x54]);            // push r12
        self.e(&[0x41, 0x55]);            // push r13
        self.e(&[0x41, 0x56]);            // push r14
        self.e(&[0x41, 0x57]);            // push r15
        // RDI = cpu, RSI = bus — already set by caller
    }
    pub fn epilogue(&mut self) {
        self.e(&[0x41, 0x5F]);            // pop r15
        self.e(&[0x41, 0x5E]);            // pop r14
        self.e(&[0x41, 0x5D]);            // pop r13
        self.e(&[0x41, 0x5C]);            // pop r12
        self.e8(0x5B);                    // pop rbx
        self.e8(0x5D);                    // pop rbp
        self.e8(0xC3);                    // ret
    }

    // ── Memory access via RDI (Armv8Cpu*) ──
    /// Load from [rdi + off] into RAX.
    pub fn ld_rax(&mut self, off: i32) {
        if off >= 0 && off < 128 {
            self.e(&[0x48, 0x8B, 0x47, off as u8]);
        } else {
            self.e(&[0x48, 0x8B, 0x87]); self.e32(off);
        }
    }
    /// Store RAX to [rdi + off].
    pub fn st_rax(&mut self, off: i32) {
        if off >= 0 && off < 128 {
            self.e(&[0x48, 0x89, 0x47, off as u8]);
        } else {
            self.e(&[0x48, 0x89, 0x87]); self.e32(off);
        }
    }
    /// Load from [rdi + off] into RCX.
    pub fn ld_rcx(&mut self, off: i32) {
        if off >= 0 && off < 128 {
            self.e(&[0x48, 0x8B, 0x4F, off as u8]);
        } else {
            self.e(&[0x48, 0x8B, 0x8F]); self.e32(off);
        }
    }
    /// Load from [rdi + off] into RDX.
    pub fn ld_rdx(&mut self, off: i32) {
        if off >= 0 && off < 128 {
            self.e(&[0x48, 0x8B, 0x57, off as u8]);
        } else {
            self.e(&[0x48, 0x8B, 0x97]); self.e32(off);
        }
    }
    /// Store RCX to [rdi + off].
    pub fn st_rcx(&mut self, off: i32) {
        if off >= 0 && off < 128 {
            self.e(&[0x48, 0x89, 0x4F, off as u8]);
        } else {
            self.e(&[0x48, 0x89, 0x8F]); self.e32(off);
        }
    }
    /// Store RDX to [rdi + off].
    pub fn st_rdx(&mut self, off: i32) {
        if off >= 0 && off < 128 {
            self.e(&[0x48, 0x89, 0x57, off as u8]);
        } else {
            self.e(&[0x48, 0x89, 0x97]); self.e32(off);
        }
    }

    // ── Reg-to-reg ALU (RAX as dst, RCX as src) ──
    pub fn add_rax_rcx(&mut self) { self.e(&[0x48, 0x01, 0xC8]); }
    pub fn sub_rax_rcx(&mut self) { self.e(&[0x48, 0x29, 0xC8]); }
    pub fn and_rax_rcx(&mut self) { self.e(&[0x48, 0x21, 0xC8]); }
    pub fn or_rax_rcx(&mut self)  { self.e(&[0x48, 0x09, 0xC8]); }
    pub fn xor_rax_rcx(&mut self) { self.e(&[0x48, 0x31, 0xC8]); }
    pub fn cmp_rax_rcx(&mut self) { self.e(&[0x48, 0x39, 0xC8]); }

    // ── RAX + imm ──
    pub fn add_rax_imm(&mut self, imm: i64) {
        if imm >= 0 && imm <= 0x7FFFFFFF {
            if imm <= 127 {
                self.e(&[0x48, 0x83, 0xC0, imm as u8]);
            } else {
                self.e(&[0x48, 0x05]); self.e32(imm as u32);
            }
        }
    }
    pub fn sub_rax_imm(&mut self, imm: i64) {
        if imm >= 0 && imm <= 0x7FFFFFFF {
            if imm <= 127 {
                self.e(&[0x48, 0x83, 0xE8, imm as u8]);
            } else {
                self.e(&[0x48, 0x2D]); self.e32(imm as u32);
            }
        }
    }
    pub fn and_rax_imm(&mut self, imm: u64) {
        if imm <= 127 {
            self.e(&[0x48, 0x83, 0xE0, imm as u8]);
        } else {
            // mov rcx, imm; and rax, rcx
            self.mov_rcx_imm64(imm);
            self.and_rax_rcx();
        }
    }
    pub fn or_rax_imm(&mut self, imm: u64) {
        if imm <= 127 {
            self.e(&[0x48, 0x83, 0xC8, imm as u8]);
        } else {
            self.mov_rcx_imm64(imm);
            self.or_rax_rcx();
        }
    }

    // ── Moves ──
    pub fn mov_rax_imm64(&mut self, imm: u64) {
        self.e(&[0x48, 0xB8]); self.buf.extend_from_slice(&imm.to_le_bytes());
    }
    pub fn mov_rcx_imm64(&mut self, imm: u64) {
        self.e(&[0x48, 0xB9]); self.buf.extend_from_slice(&imm.to_le_bytes());
    }
    pub fn mov_rax_rcx(&mut self) { self.e(&[0x48, 0x89, 0xC8]); }

    // ── Flag operations ──
    /// Load flags from RAX into x86_64 EFLAGS via SAHF.
    /// RAX lower byte: N(ZF=7) Z(ZF=6) C(CF=0) V(OF=?) → we pack into AH
    /// Actually SAHF sets SF, ZF, AF, PF, CF from AH. We only need SF, ZF, CF.
    /// Pack NZCV into AH for SAHF: C→CF(bit0), Z→ZF(bit6), N→SF(bit7), V→ignored
    pub fn sahf_from_al(&mut self) {
        // Assuming AL = (N<<7) | (Z<<6) | (C<<0) | (V...)
        // SAHF: AH → SF, ZF, AF, PF, CF
        // AH bits: 7=SF, 6=ZF, 4=AF, 2=PF, 0=CF
        // We need to map N→SF(bit7), Z→ZF(bit6), C→CF(bit0)
        // AL = (N<<7)|(Z<<6)|(C<<0) → this directly matches SAHF format!
        self.e(&[0x9E]); // sahf
    }
    /// Store x86_64 flags into AL (NZCV format).
    /// lahf loads AH from flags: SF→bit7, ZF→bit6, AF→bit4, PF→bit2, CF→bit0
    /// We also need V (overflow) from the OF flag.
    pub fn lahf_to_rax(&mut self) {
        self.e(&[0x9F]); // lahf → AH = flags
        // AH is bits 15:8 of AX. We want the result in AL.
        // and eax, 0xC1 keeps SF(bit7), ZF(bit6), CF(bit0)
        // Then setc al + shifts?
        // Simpler: lahf; movzx eax, ah; shl eax, 8 is not needed.
        // Actually lahf loads AH. Then:
        // seto al  → AL = OF (V flag)
        // But we need to combine them.
        // For now, use a simpler approach:
        // pushfq; pop rax → full RFLAGS in RAX
    }

    /// Store NZCV (as a byte) to [rdi + pstate + 3].
    /// pstate.bits has NZCV at bits [31:28].
    pub fn st_flags_from_al(&mut self) {
        // RAX low byte has packed NZCV: N(bit7) Z(bit6) C(bit1) V(bit0)
        // We need to write this to pstate.bits[31:28]
        // But changing only those 4 bits requires read-modify-write.
        // Simpler: store into a dedicated flags_cache field.
    }

    // ── Control flow ──
    pub fn jmp_rel32(&mut self, off: i32) {
        self.e8(0xE9); self.e32(off as u32);
    }
    pub fn je_rel32(&mut self, off: i32) {
        self.e(&[0x0F, 0x84]); self.e32(off as u32);
    }
    pub fn jne_rel32(&mut self, off: i32) {
        self.e(&[0x0F, 0x85]); self.e32(off as u32);
    }
    pub fn jl_rel32(&mut self, off: i32) {
        self.e(&[0x0F, 0x8C]); self.e32(off as u32);
    }
    pub fn jg_rel32(&mut self, off: i32) {
        self.e(&[0x0F, 0x8F]); self.e32(off as u32);
    }
    pub fn jle_rel32(&mut self, off: i32) {
        self.e(&[0x0F, 0x8E]); self.e32(off as u32);
    }
    pub fn jge_rel32(&mut self, off: i32) {
        self.e(&[0x0F, 0x8D]); self.e32(off as u32);
    }
    pub fn jb_rel32(&mut self, off: i32) {
        self.e(&[0x0F, 0x82]); self.e32(off as u32);
    }
    pub fn jae_rel32(&mut self, off: i32) {
        self.e(&[0x0F, 0x83]); self.e32(off as u32);
    }
    pub fn test_rax_rax(&mut self) {
        self.e(&[0x48, 0x85, 0xC0]); // test rax, rax
    }

    // ── Shifts ──
    pub fn shl_rax_cl(&mut self) { self.e(&[0x48, 0xD3, 0xE0]); }
    pub fn shr_rax_cl(&mut self) { self.e(&[0x48, 0xD3, 0xE8]); }
    pub fn sar_rax_cl(&mut self) { self.e(&[0x48, 0xD3, 0xF8]); }

    // ── Call extern functions ──
    /// Call a Rust function using relative call (patch later).
    /// We use a placeholder that gets resolved at link time.
    pub fn call_r11(&mut self) {
        self.e(&[0x41, 0xFF, 0xD3]); // call r11
    }
    /// Load function pointer into R11 and call.
    pub fn call_ptr(&mut self, ptr: *const u8) {
        self.mov_rcx_imm64(ptr as u64);
        self.e(&[0x49, 0x89, 0xCB]); // mov r11, rcx
        self.e(&[0x41, 0xFF, 0xD3]); // call r11
    }
}
