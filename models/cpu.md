# CPU and Instruction State

Minimal CPU-side data structures for the ARM64 emulator, kept in sync with the
implementation.

## Register File

31 general-purpose 64-bit registers (`X0..X30`), plus SP and PC.

```
RegisterFile {
    x: [u64; 31],
    sp: u64,
    pc: u64,
}
```

`Wn` reads the low 32 bits of `Xn` (write zeros the top half). Register 31 is the
zero register (XZR/WZR), so reads return 0 and writes are discarded. In
address-forming instructions (LDR/STR/LDP/STP), register 31 means SP.

## Program Status (PSTATE)

```
ProcessorState {
    bits: u64,   // NZCV in bits [31:28], EL in [3:2], IRQ mask at bit 7
}
```

Boots at EL3, IRQ masked. Real UEFI firmware would drop to EL2 before calling
the PE entry; our emulator passes control at EL3 and the kernel's own head.S
drops to EL2 -> EL1.

## System Registers

```
SystemRegisters {
    sctlr_el1: u64,   // MMU enable (bit 0), caches, alignment
    tcr_el1: u64,     // Translation control (granule, T0SZ, T1SZ)
    ttbr0_el1: u64,   // TTBR0, user-space page table base
    ttbr1_el1: u64,   // TTBR1, kernel page table base
    mair_el1: u64,    // Memory attribute indirection
    vbar_el1: u64,    // Exception vector base address
    esr_el1: u64,     // Exception syndrome register
    far_el1: u64,     // Fault address register
    spsr_el1: u64,    // Saved program status
    elr_el1: u64,     // Exception link register
    cpacr_el1: u64,   // Architectural Feature Access Control
    sp_el0: u64,      // EL0 stack pointer
    cntfrq_el0: u64,  // Counter frequency (62.5 MHz)
    cntp_ctl_el0: u64,   // Physical Timer Control
    cntp_cval_el0: u64,  // Physical Timer Compare Value
    cntp_tval_el0: u64,  // Physical Timer Timer Value
    cycle_count: u64,    // Emulated cycle counter

    // GICv3 CPU interface (system-register access)
    icc_pmr_el1: u64,    // Priority Mask
    icc_ctlr_el1: u64,   // Control Register
    icc_sre_el1: u64,    // System Register Enable
    icc_iar1_el1: u64,   // Interrupt Acknowledge

    // EL2 / EL3 registers (used during boot stub)
    scr_el3: u64, spsr_el3: u64, elr_el3: u64,
    hcr_el2: u64, spsr_el2: u64, elr_el2: u64,

    irq_pending: bool,
    last_irq_id: u32,
}
```

## CPU State

```
Armv8Cpu {
    core_id: u32,
    regs: RegisterFile,
    pstate: ProcessorState,
    sys: SystemRegisters,
    tlb: Tlb,            // 2048-entry direct-mapped software TLB
}
```

Multi-core support via `Machine` struct: `Vec<Armv8Cpu>` sharing one `SystemBus`.
Scheduling is round-robin per instruction.

## Decoded Instruction

```
Instr {
    op: Opcode,       // Add, Sub, Movz, Ldr, Str, B, Bl, Ret,
                      // Cbz, Cbnz, BCond, Ldp, Stp, Adrp, Mrs, Msr, Madd,
                      // Msub, Tlbi, Svc, Eret, Brk, Wfi, Wfe, Ldxr, Stxr, ...
    rd: u8,           // Destination register (0-31)
    rn: u8,           // First source / base register
    rm: u8,           // Second source register
    imm: u64,         // Immediate, sysreg ID, or offset
    sf: bool,         // 64-bit (true) vs 32-bit (false)
    cond: u8,         // Condition code, shift type, or discriminator
    size: u8,         // Access size in bytes (for LDR/STR)
}
```
