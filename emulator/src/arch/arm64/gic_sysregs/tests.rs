use super::*;
use crate::constants::{PL011_UART_IRQ_ID, UART_BASE, UART_IMSC_OFFSET};

const SGI: u32 = 5;
const SPI: u32 = 48;
const SYSREG_RD: u8 = 3;

fn sysreg(op: Opcode, id: u16) -> Instr {
    Instr {
        op,
        rd: SYSREG_RD,
        imm: id as u64,
        ..Instr::nop()
    }
}

fn latch(cpu: &mut Armv8Cpu, int_id: u32) {
    cpu.sys.irq_pending = true;
    cpu.sys.last_irq_id = int_id;
}

#[test]
fn mode_zero_eoir_preserves_an_sgi_repended_after_iar() {
    let mut cpu = Armv8Cpu::with_core(1);
    let mut bus = SystemBus::with_cpu_count(2);
    bus.gic.enable_interrupt_for_cpu(1, SGI);
    bus.gic.set_pending_for_cpu(1, SGI);
    latch(&mut cpu, SGI);

    assert!(handle_gic_sysreg_access(
        &mut cpu,
        &mut bus,
        sysreg(Opcode::Mrs, SYSREG_ICC_IAR1_EL1),
    ));
    assert_eq!(cpu.regs.x(SYSREG_RD), SGI as u64);
    assert!(!bus.gic.is_pending_for_cpu(1, SGI));
    assert!(bus.gic.is_active_for_cpu(1, SGI));

    bus.gic.set_pending_for_cpu(1, SGI);
    assert_eq!(bus.gic.next_pending_enabled_for_cpu(1), None);
    cpu.regs.set_x(SYSREG_RD, SGI as u64);
    assert!(handle_gic_sysreg_access(
        &mut cpu,
        &mut bus,
        sysreg(Opcode::Msr, SYSREG_ICC_EOIR1_EL1),
    ));

    assert!(!bus.gic.is_active_for_cpu(1, SGI));
    assert!(bus.gic.is_pending_for_cpu(1, SGI));
    assert_eq!(bus.gic.next_pending_enabled_for_cpu(1), Some(SGI));
}

#[test]
fn split_eoi_and_dir_preserve_an_spi_repended_after_iar() {
    let mut cpu = Armv8Cpu::with_core(1);
    let mut bus = SystemBus::with_cpu_count(2);
    bus.gic.enable_interrupt(SPI);
    bus.gic
        .set_interrupt_route(SPI, bus.gic.cpu_affinity(1).unwrap());
    bus.gic.set_pending(SPI);
    latch(&mut cpu, SPI);

    assert!(handle_gic_sysreg_access(
        &mut cpu,
        &mut bus,
        sysreg(Opcode::Mrs, SYSREG_ICC_IAR1_EL1),
    ));
    assert!(!bus.gic.is_pending_for_cpu(1, SPI));
    assert!(bus.gic.is_active_for_cpu(1, SPI));

    bus.gic.set_pending(SPI);
    cpu.sys.icc_ctlr_el1 = ICC_CTLR_EOIMODE;
    cpu.regs.set_x(SYSREG_RD, SPI as u64);
    assert!(handle_gic_sysreg_access(
        &mut cpu,
        &mut bus,
        sysreg(Opcode::Msr, SYSREG_ICC_EOIR1_EL1),
    ));
    assert!(bus.gic.is_active_for_cpu(1, SPI));
    assert!(bus.gic.is_pending_for_cpu(1, SPI));
    assert_eq!(bus.gic.next_pending_enabled_for_cpu(1), None);

    assert!(handle_gic_sysreg_access(
        &mut cpu,
        &mut bus,
        sysreg(Opcode::Msr, SYSREG_ICC_DIR_EL1),
    ));
    assert!(!bus.gic.is_active_for_cpu(1, SPI));
    assert!(bus.gic.is_pending_for_cpu(1, SPI));
    assert_eq!(bus.gic.next_pending_enabled_for_cpu(1), Some(SPI));
}

#[test]
fn uart_level_is_revalidated_when_the_interrupt_deactivates() {
    let mut cpu = Armv8Cpu::new();
    let mut bus = SystemBus::new();
    bus.write(UART_BASE + UART_IMSC_OFFSET, 4, 0x50);
    bus.feed_uart_input("x");
    bus.gic.enable_interrupt(PL011_UART_IRQ_ID);
    latch(&mut cpu, PL011_UART_IRQ_ID);

    handle_gic_sysreg_access(&mut cpu, &mut bus, sysreg(Opcode::Mrs, SYSREG_ICC_IAR1_EL1));
    bus.refresh_interrupts();
    assert!(bus.gic.is_pending(PL011_UART_IRQ_ID));
    assert!(bus.gic.is_active_for_cpu(0, PL011_UART_IRQ_ID));

    assert_eq!(bus.read(UART_BASE, 1), Some(b'x' as u64));
    cpu.regs.set_x(SYSREG_RD, PL011_UART_IRQ_ID as u64);
    handle_gic_sysreg_access(
        &mut cpu,
        &mut bus,
        sysreg(Opcode::Msr, SYSREG_ICC_EOIR1_EL1),
    );

    assert!(!bus.gic.is_active_for_cpu(0, PL011_UART_IRQ_ID));
    assert!(!bus.gic.is_pending(PL011_UART_IRQ_ID));
}
