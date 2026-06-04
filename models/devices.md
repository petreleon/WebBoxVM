# Devices and Interrupts

Device maps and interrupt behavior for the ARM64 emulator.

## Device MMIO Map

| Start       | End         | Size    | Device                     |
|-------------|-------------|---------|----------------------------|
| 0x0800_0000 | 0x0800_FFFF | 64 KiB  | GICv3 Distributor (GICD)   |
| 0x080A_0000 | 0x08FF_FFFF | ~15 MiB | GICv3 Redistributor (GICR) |
| 0x0900_0000 | 0x0900_0FFF | 4 KiB   | PL011 UART                 |

## PL011 UART (Emulated Registers)

| Offset | Register | Description                         |
|--------|----------|-------------------------------------|
| 0x00   | UARTDR   | Data Register (R/W)                 |
| 0x04   | UARTRSR  | Receive Status / Error Clear        |
| 0x18   | UARTFR   | Flag Register (TXFE, RXFE, BUSY)    |
| 0x24   | UARTIBRD | Integer Baud Rate Divisor           |
| 0x28   | UARTFBRD | Fractional Baud Rate Divisor        |
| 0x2C   | UARTLCR_H| Line Control Register (high)        |
| 0x30   | UARTCR   | Control Register (UARTEN, TXE, RXE) |
| 0x34   | UARTIFLS | Interrupt FIFO Level Select         |
| 0x38   | UARTIMSC | Interrupt Mask Set/Clear            |
| 0x3C   | UARTRIS  | Raw Interrupt Status                |
| 0x40   | UARTMIS  | Masked Interrupt Status             |
| 0x44   | UARTICR  | Interrupt Clear                     |
| 0x48   | UARTDMACR| DMA Control Register                |

## GICv3 Interrupt Controller

Minimal distributor and redistributor MMIO emulation enough for Linux init:

- **GICD**: interrupt enable, pending, priority, group arrays (32 interrupts).
- **GICR**: per-core control (CTLR, WAKER, TYPER).
- **CPU Interface**: ICC_PMR_EL1, ICC_CTLR_EL1, ICC_SRE_EL1, ICC_IAR1_EL1,
  ICC_EOIR1_EL1 via system registers.
- Timer IRQ (ID 30) is the only interrupt delivered.

## Timer / IRQ Model

Cycle counter increments per instruction (62.5 MHz simulated). Timer IRQ (PPI
30) fires when `cycle_count >= CNTP_CVAL_EL0`. Delivery skips until `VBAR_EL1`
is configured.

WFI/WFE fast-forward the cycle counter to timer expiry. DAIFSet/DAIFClr control
`PSTATE.I` (IRQ mask). After VBAR is set and the kernel configures the timer, a
one-shot IRQ fires to break early-boot spin loops.
