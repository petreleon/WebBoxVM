# WebBoxVM — Models

Minimal data structures for the ARM64 emulator, kept in sync with the implementation.

## Register File

31 general-purpose 64-bit registers (`X0..X30`), plus SP and PC.

```
RegisterFile {
    x: [u64; 31],
    sp: u64,
    pc: u64,
}
```

`Wn` reads the low 32 bits of `Xn` (write zeros the top half). Register 31 is the zero register (XZR/WZR) — reads return 0, writes are discarded. In address‑forming instructions (LDR/STR/LDP/STP), register 31 means SP.

## Program Status (PSTATE)

```
ProcessorState {
    bits: u64,   // NZCV in bits [31:28], EL in [3:2], IRQ mask at bit 7
}
```

Boots at EL3, IRQ masked.  Real UEFI firmware would drop to EL2 before calling the PE entry; our emulator passes control at EL3 and the kernel's own head.S drops to EL2→EL1.

## System Registers

```
SystemRegisters {
    sctlr_el1: u64,   // MMU enable (bit 0), caches, alignment
    tcr_el1: u64,     // Translation control (granule, T0SZ, T1SZ)
    ttbr0_el1: u64,   // TTBR0 — user-space page table base
    ttbr1_el1: u64,   // TTBR1 — kernel page table base
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

    // GICv3 CPU interface (system‑register access)
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
    tlb: Tlb,            // 2048-entry direct‑mapped software TLB
}
```

Multi‑core support via `Machine` struct: Vec<Armv8Cpu> sharing one SystemBus. Round‑robin scheduling per instruction.

## Decoded Instruction

```
Instr {
    op: Opcode,       // 90 opcodes: Add, Sub, Movz, Ldr, Str, B, Bl, Ret,
                      // Cbz, Cbnz, BCond, Ldp, Stp, Adrp, Mrs, Msr, Madd,
                      // Msub, Tlbi, Svc, Eret, Brk, Wfi, Wfe, Ldxr, Stxr, ...
    rd: u8,           // Destination register (0–31)
    rn: u8,           // First source / base register
    rm: u8,           // Second source register
    imm: u64,         // Immediate, sysreg ID, or offset
    sf: bool,         // 64‑bit (true) vs 32‑bit (false)
    cond: u8,         // Condition code, shift type, or discriminator
    size: u8,         // Access size in bytes (for LDR/STR)
}
```

## Physical Memory

Three disjoint `Vec<u8>` regions:

| Start       | End         | Size   | Usage               |
|-------------|-------------|--------|---------------------|
| 0x0000_0000 | 0x3FFF_FFFF | 1 GiB  | Low (MMIO, vectors) |
| 0x4000_0000 | 0x7FFF_FFFF | 1 GiB  | RAM (kernel, heap)  |
| 0x8000_0000 | 0x8FFF_FFFF | 256 MiB| EFI firmware tables |

## Device MMIO Map

| Start       | End         | Size     | Device                      |
|-------------|-------------|----------|-----------------------------|
| 0x0800_0000 | 0x0800_FFFF | 64 KiB   | GICv3 Distributor (GICD)   |
| 0x080A_0000 | 0x08FF_FFFF | ~15 MiB  | GICv3 Redistributor (GICR) |
| 0x0900_0000 | 0x0900_0FFF | 4 KiB    | PL011 UART                  |

## PL011 UART (Emulated Registers)

| Offset | Register | Description                          |
|--------|----------|--------------------------------------|
| 0x00   | UARTDR   | Data Register (R/W)                  |
| 0x04   | UARTRSR  | Receive Status / Error Clear         |
| 0x18   | UARTFR   | Flag Register (TXFE, RXFE, BUSY)     |
| 0x24   | UARTIBRD | Integer Baud Rate Divisor            |
| 0x28   | UARTFBRD | Fractional Baud Rate Divisor         |
| 0x2C   | UARTLCR_H| Line Control Register (high)         |
| 0x30   | UARTCR   | Control Register (UARTEN, TXE, RXE)  |
| 0x34   | UARTIFLS | Interrupt FIFO Level Select          |
| 0x38   | UARTIMSC | Interrupt Mask Set/Clear             |
| 0x3C   | UARTRIS  | Raw Interrupt Status                 |
| 0x40   | UARTMIS  | Masked Interrupt Status              |
| 0x44   | UARTICR  | Interrupt Clear                      |
| 0x48   | UARTDMACR| DMA Control Register                 |

## GICv3 Interrupt Controller

Minimal distributor + redistributor MMIO emulation enough for Linux init:

- **GICD** — Interrupt enable, pending, priority, group arrays (32 interrupts)
- **GICR** — Per‑core control (CTLR, WAKER, TYPER)
- **CPU Interface** — ICC_PMR_EL1, ICC_CTLR_EL1, ICC_SRE_EL1, ICC_IAR1_EL1, ICC_EOIR1_EL1 (via system registers)
- Timer IRQ (ID 30) is the only interrupt delivered

## EFI Firmware Region (0x8000_0000–0x8FFF_FFFF)

| Address       | Structure              |
|---------------|------------------------|
| 0x8000_0000   | EFI Image Handle       |
| 0x8000_1000   | EFI System Table       |
| 0x8000_2000   | Runtime Services Table |
| 0x8000_3000   | Boot Services Table    |
| 0x8000_4000   | Trampoline Code (32‑byte slots) |
| 0x8000_C000   | Large Trampolines (512‑byte blocks) |
| 0x8000_8000   | Loaded Image Protocol  |

## Memory Access

Read/write takes `(address, width)` where width is 1, 2, 4, or 8 bytes.
Little-endian only (ARM64 default).

## MMU / TLB

```
Tlb {
    entries: [TlbEntry; 2048],  // direct‑mapped by VA bits [23:12]
}

TlbEntry {
    valid: bool,
    va_page: u64,   // virtual page number (VA >> 12)
    pa_page: u64,   // physical page number (PA >> 12)
}
```

`translate(va)`:
1. If `SCTLR_EL1.M == 0` → pass‑through (identity map)
2. Check TLB for cached entry
3. Walk 3‑level page table (39‑bit VA, 4 KiB granule, 512 entries/level)
4. Cache result in TLB

Kernel MMIO fixups: map `0xffff8000_09xxxxxx` → physical UART/GIC for `early_ioremap`.

## MMIO Dispatch

`SystemBus::read/write(addr, size)` routes:
1. UART range → PL011 device
2. GICD range → GIC distributor
3. GICR range → GIC redistributor
4. RAM/Low/EFI → PhysicalMemory

Writes to unmapped regions are silently discarded.

## Timer / IRQ Model

Cycle counter increments per instruction (62.5 MHz simulated). Timer IRQ (PPI 30) fires when `cycle_count ≥ CNTP_CVAL_EL0`. Delivery skips until `VBAR_EL1` is configured.

WFI/WFE fast‑forward the cycle counter to timer expiry. DAIFSet/DAIFClr control `PSTATE.I` (IRQ mask). After VBAR is set and the kernel configures the timer, a one‑shot IRQ fires to break early‑boot spin loops.
