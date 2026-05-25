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
PState {
    el: u8,     // Exception level: 3, 2, 1, 0
    n: bool,    // Negative
    z: bool,    // Zero
    c: bool,    // Carry
    v: bool,    // Overflow
    d: bool,    // Debug mask
    a: bool,    // SError mask
    i: bool,    // IRQ mask
    f: bool,    // FIQ mask
}
```

Boot state: `EL3`, all interrupts masked, `PC = 0x0`.

## Physical Memory

`1 GiB` flat array. Layout (ARM64 virt platform):

| Start       | End         | Size  | Usage          |
|-------------|-------------|-------|----------------|
| 0x4000_0000 | 0x7FFF_FFFF | 1 GB  | RAM            |
| 0x0900_0000 | 0x0900_FFFF | 64 KB | PL011 UART     |
| 0x0800_0000 | 0x0800_FFFF | 64 KB | GIC distributor|
| 0x0A00_0000 | 0x0A00_0FFF | 4 KB  | VirtIO MMIO    |

## CPU State

```
Armv8Cpu {
    regs: RegisterFile,
    pstate: PState,
    sctlr_el1: u64,
    tcr_el1: u64,
    ttbr0_el1: u64,
    ttbr1_el1: u64,
    mair_el1: u64,
    vbar_el1: u64,
    esr_el1: u64,
    far_el1: u64,
    spsr_el1: u64,
    elr_el1: u64,
}
```

## Decoded Instruction

```
DecodedInstr {
    opcode: OpCode,
    rd: u8,
    rn: u8,
    rm: u8,
    imm: u64,
    shift: ShiftType,
}
```

## Memory Access

Read/write takes `(address, width)` where width is 1, 2, 4, or 8 bytes.
Little-endian only (ARM64 default).

## MMIO Dispatch

Address range → handler function. Any address not in RAM and not in device range is a bus fault.
