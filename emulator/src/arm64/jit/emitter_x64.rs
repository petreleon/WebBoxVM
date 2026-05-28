//! x86_64 machine code emitter. Generates raw x86_64 instructions.
//! Target: macOS/Linux x86_64 ABI (System V).

use crate::arm64::Armv8Cpu;
use crate::bus::SystemBus;

/// Offset of `regs.x[n]` within Armv8Cpu (in bytes).
pub const fn reg_x_offset(n: u8) -> i32 {
    // Armv8Cpu layout: regs (RegisterFile), pstate, sys, tlb
    // RegisterFile: x[31] + sp + pc
    // x[0] is at offset 0 within RegisterFile
    0 + (n as i32) * 8
}

/// Compiled x86_64 function: receives (&mut Armv8Cpu, &mut SystemBus), returns nothing.
pub struct CompiledBlock {
    pub code: Vec<u8>,
    pub arm64_instr_count: usize,
}

impl CompiledBlock {
    /// Execute the compiled code. UNSAFE: jumps to raw machine code.
    pub unsafe fn execute(&self, cpu: &mut Armv8Cpu, bus: &mut SystemBus) {
        let fn_ptr: extern "C" fn(*mut Armv8Cpu, *mut SystemBus) =
            unsafe { std::mem::transmute(self.code.as_ptr()) };
        fn_ptr(cpu as *mut Armv8Cpu, bus as *mut SystemBus);
    }
}

/// x86_64 code emitter. Builds a buffer of raw x86_64 machine code.
pub struct Emitter {
    buf: Vec<u8>,
}

impl Emitter {
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }

    pub fn finish(self) -> Vec<u8> {
        self.buf
    }

    // ── Helper: emit bytes ──

    pub fn emit(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }

    pub fn emit_u8(&mut self, b: u8) { self.buf.push(b); }
    pub fn emit_u32(&mut self, v: u32) { self.buf.extend_from_slice(&v.to_le_bytes()); }
    pub fn emit_u64(&mut self, v: u64) { self.buf.extend_from_slice(&v.to_le_bytes()); }

    // ── Prologue: save frame, set up stack ──

    pub fn prologue(&mut self) {
        // push rbp
        self.emit_u8(0x55);
        // mov rbp, rsp
        self.emit(&[0x48, 0x89, 0xE5]);
        // Save callee-saved: push rbx, r12-r15
        self.emit(&[0x53]); // push rbx
        self.emit(&[0x41, 0x54]); // push r12
        self.emit(&[0x41, 0x55]); // push r13
        self.emit(&[0x41, 0x56]); // push r14
        self.emit(&[0x41, 0x57]); // push r15
    }

    // ── Epilogue: restore frame, return ──

    pub fn epilogue(&mut self) {
        // pop r15-r12, rbx
        self.emit(&[0x41, 0x5F]); // pop r15
        self.emit(&[0x41, 0x5E]); // pop r14
        self.emit(&[0x41, 0x5D]); // pop r13
        self.emit(&[0x41, 0x5C]); // pop r12
        self.emit(&[0x5B]); // pop rbx
        // pop rbp
        self.emit(&[0x5D]);
        // ret
        self.emit_u8(0xC3);
    }

    // ── Register helpers ──
    // Arguments: RDI = &mut Armv8Cpu, RSI = &mut SystemBus
    // We use RDI as base pointer to Armv8Cpu

    /// Load ARM64 register Xn into RAX.
    pub fn load_x(&mut self, n: u8) {
        let off = reg_x_offset(n);
        // mov rax, [rdi + off]
        if off < 128 {
            self.emit(&[0x48, 0x8B, 0x47, off as u8]);
        } else {
            self.emit(&[0x48, 0x8B, 0x87]);
            self.emit_u32(off as u32);
        }
    }

    /// Store RAX to ARM64 register Xn.
    pub fn store_x(&mut self, n: u8) {
        let off = reg_x_offset(n);
        // mov [rdi + off], rax
        if off < 128 {
            self.emit(&[0x48, 0x89, 0x47, off as u8]);
        } else {
            self.emit(&[0x48, 0x89, 0x87]);
            self.emit_u32(off as u32);
        }
    }

    /// Load ARM64 register Xn into RCX.
    pub fn load_x_rcx(&mut self, n: u8) {
        let off = reg_x_offset(n);
        // mov rcx, [rdi + off]
        if off < 128 {
            self.emit(&[0x48, 0x8B, 0x4F, off as u8]);
        } else {
            self.emit(&[0x48, 0x8B, 0x8F]);
            self.emit_u32(off as u32);
        }
    }

    /// Load ARM64 register Xn into RDX.
    pub fn load_x_rdx(&mut self, n: u8) {
        let off = reg_x_offset(n);
        // mov rdx, [rdi + off]
        if off < 128 {
            self.emit(&[0x48, 0x8B, 0x57, off as u8]);
        } else {
            self.emit(&[0x48, 0x8B, 0x97]);
            self.emit_u32(off as u32);
        }
    }

    // ── ALU operations ──

    /// ADD RAX, RCX  → RAX += RCX
    pub fn add_rax_rcx(&mut self) {
        self.emit(&[0x48, 0x01, 0xC8]); // add rax, rcx
    }

    /// SUB RAX, RCX  → RAX -= RCX
    pub fn sub_rax_rcx(&mut self) {
        self.emit(&[0x48, 0x29, 0xC8]); // sub rax, rcx
    }

    /// ADD RAX, imm32
    pub fn add_rax_imm(&mut self, imm: i32) {
        if imm >= -128 && imm <= 127 {
            self.emit(&[0x48, 0x83, 0xC0, imm as u8]); // add rax, imm8
        } else {
            self.emit(&[0x48, 0x05]); // add rax, imm32
            self.emit_u32(imm as u32);
        }
    }

    /// SUB RAX, imm32
    pub fn sub_rax_imm(&mut self, imm: i32) {
        if imm >= -128 && imm <= 127 {
            self.emit(&[0x48, 0x83, 0xE8, imm as u8]); // sub rax, imm8
        } else {
            self.emit(&[0x48, 0x2D]); // sub rax, imm32
            self.emit_u32(imm as u32);
        }
    }

    /// Compare RAX with RCX (sets flags for conditional jumps)
    pub fn cmp_rax_rcx(&mut self) {
        self.emit(&[0x48, 0x39, 0xC8]); // cmp rax, rcx
    }

    // ── Control flow ──

    /// Unconditional jump to a label (relative offset from end of instruction).
    /// Returns position of the jump for later patching.
    pub fn jmp_rel32(&mut self, offset: i32) {
        self.emit_u8(0xE9); // jmp rel32
        self.emit_u32(offset as u32);
    }

    /// Jump if equal (ZF=1). Returns position for patching.
    pub fn je_rel32(&mut self, offset: i32) {
        self.emit(&[0x0F, 0x84]);
        self.emit_u32(offset as u32);
    }

    /// Jump if not equal (ZF=0).
    pub fn jne_rel32(&mut self, offset: i32) {
        self.emit(&[0x0F, 0x85]);
        self.emit_u32(offset as u32);
    }

    // ── Move / Immediate ──

    /// MOV RAX, imm64
    pub fn mov_rax_imm64(&mut self, imm: u64) {
        self.emit(&[0x48, 0xB8]); // mov rax, imm64
        self.emit_u64(imm);
    }
}
