# Memory and Translation

Memory layout, translation, and dispatch models for the ARM64 emulator.

## Physical Memory

Three disjoint `Vec<u8>` regions:

| Start       | End         | Size    | Usage               |
|-------------|-------------|---------|---------------------|
| 0x0000_0000 | 0x3FFF_FFFF | 1 GiB   | Low (MMIO, vectors) |
| 0x4000_0000 | 0x7FFF_FFFF | 1 GiB   | RAM (kernel, heap)  |
| 0x8000_0000 | 0x8FFF_FFFF | 256 MiB | EFI firmware tables |

## EFI Firmware Region (0x8000_0000-0x8FFF_FFFF)

| Address     | Structure                         |
|-------------|-----------------------------------|
| 0x8000_0000 | EFI Image Handle                  |
| 0x8000_1000 | EFI System Table                  |
| 0x8000_2000 | Runtime Services Table            |
| 0x8000_3000 | Boot Services Table               |
| 0x8000_4000 | Trampoline Code (32-byte slots)   |
| 0x8000_C000 | Large Trampolines (512-byte blocks) |
| 0x8000_8000 | Loaded Image Protocol             |

## Memory Access

Read/write takes `(address, width)` where width is 1, 2, 4, or 8 bytes.
Little-endian only (ARM64 default).

## MMU / TLB

```
Tlb {
    entries: [TlbEntry; 2048],  // direct-mapped by VA bits [23:12]
}

TlbEntry {
    valid: bool,
    va_page: u64,   // virtual page number (VA >> 12)
    pa_page: u64,   // physical page number (PA >> 12)
}
```

`translate(va)`:

1. If `SCTLR_EL1.M == 0`, pass through as an identity map.
2. Check TLB for cached entry.
3. Walk 3-level page table (39-bit VA, 4 KiB granule, 512 entries/level).
4. Cache result in TLB.

Kernel MMIO fixups map `0xffff8000_09xxxxxx` to physical UART/GIC for
`early_ioremap`.

## MMIO Dispatch

`SystemBus::read/write(addr, size)` routes:

1. UART range to the PL011 device.
2. GICD range to the GIC distributor.
3. GICR range to the GIC redistributor.
4. RAM/Low/EFI to physical memory.

Writes to unmapped regions are silently discarded.
