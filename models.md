# WebBoxVM — Models

Minimal data structures for the ARM64 interpreter.

## Register File

31 general-purpose 64-bit registers (`X0..X30`), plus SP and PC.

```
RegisterFile {
    x: [u64; 31],
    sp: u64,
    pc: u64,
}
```

Where `Wn` is the low 32 bits of `Xn` (write zeros the top half).

## Program Status

```
ProcessorState {
    bits: u64,   // NZCV in top 4 bits, EL in bits [3:2]
}
```

Boot state: `EL3`, all interrupts masked, `PC = 0x0`.

## System Registers

```
SystemRegisters {
    sctlr_el1: u64,   // MMU enable (bit 0), caches, alignment
    tcr_el1: u64,     // Translation control (granule, T0SZ, T1SZ)
    ttbr0_el1: u64,   // User-space page table base
    ttbr1_el1: u64,   // Kernel page table base
    mair_el1: u64,    // Memory attribute indirection
    vbar_el1: u64,    // Exception vector base
    esr_el1: u64,     // Exception syndrome
    far_el1: u64,     // Faulting address
    spsr_el1: u64,    // Saved program status
    elr_el1: u64,     // Exception link
    sp_el0: u64,      // EL0 stack pointer
    // ... plus EL2/EL3 registers
}
```

## CPU State

```
Armv8Cpu {
    regs: RegisterFile,
    pstate: ProcessorState,
    sys: SystemRegisters,
    tlb: Tlb,            // 2048-entry software TLB
}
```

## Decoded Instruction

```
Instr {
    op: Opcode,       // e.g., Add, Ldr, Mrs, Tlbi
    rd: u8,           // Destination register
    rn: u8,           // First source / base register
    rm: u8,           // Second source register
    imm: u64,         // Immediate or sysreg ID
    sf: bool,         // 64-bit (true) vs 32-bit (false)
    cond: u8,         // Condition code or shift type
    size: u8,         // Access size in bytes (for LDR/STR)
}
```

## Physical Memory

Three disjoint `Vec<u8>` regions:

| Start       | End         | Size   | Usage          |
|-------------|-------------|--------|----------------|
| 0x0000_0000 | 0x3FFF_FFFF | 1 GB   | Low (boot, vectors) |
| 0x4000_0000 | 0x7FFF_FFFF | 1 GB   | RAM (kernel)   |
| 0x8000_0000 | 0x8FFF_FFFF | 256 MB | EFI runtime    |

Device MMIO:

| Start       | End         | Size   | Device         |
|-------------|-------------|--------|----------------|
| 0x0900_0000 | 0x0900_FFFF | 64 KB  | PL011 UART     |

## Memory Access

Read/write takes `(address, width)` where width is 1, 2, 4, or 8 bytes.
Little-endian only (ARM64 default).

## MMU / TLB

```
Tlb {
    entries: [TlbEntry; 2048],  // direct-mapped
}

TlbEntry {
    valid: bool,
    va_page: u64,   // 4 KB page number
    pa_page: u64,   // Physical page number
}
```

`translate(va)` → if `SCTLR_EL1.M == 0`, passthrough. Otherwise walk page tables or TLB.

Page table walk: 3 levels, 39-bit VA, 4 KB granule, 512 entries per level.

## MMIO Dispatch

Bus checks UART address first, then falls back to RAM.
Any address in no region returns `None` (bus fault).
