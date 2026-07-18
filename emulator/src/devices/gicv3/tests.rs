use super::*;

const IRQ_A: u32 = 48;
const IRQ_B: u32 = 73;

#[test]
fn pending_before_enable_becomes_deliverable() {
    let mut gic = Gicv3::new();

    gic.set_pending(IRQ_A);
    assert!(!gic.has_pending_enabled());
    assert_eq!(gic.next_pending_enabled(), None);

    gic.enable_interrupt(IRQ_A);
    assert!(gic.has_pending_enabled());
    assert_eq!(gic.next_pending_enabled(), Some(IRQ_A));

    gic.clear_pending(IRQ_A);
    assert!(!gic.has_pending_enabled());
    assert_eq!(gic.next_pending_enabled(), None);
}

#[test]
fn distributor_pending_and_enable_writes_refresh_cache() {
    let mut gic = Gicv3::new();
    let word = IRQ_A / 32;
    let bit = 1u64 << (IRQ_A % 32);

    gic.gicd_write(0x0200 + (word as u64) * 4, bit, 4);
    assert_eq!(gic.next_pending_enabled(), None);

    gic.gicd_write(0x0100 + (word as u64) * 4, bit, 4);
    assert_eq!(gic.next_pending_enabled(), Some(IRQ_A));

    gic.gicd_write(0x0180 + (word as u64) * 4, bit, 4);
    assert_eq!(gic.next_pending_enabled(), None);
}

#[test]
fn next_pending_enabled_uses_lowest_active_interrupt() {
    let mut gic = Gicv3::new();

    gic.enable_interrupt(IRQ_B);
    gic.enable_interrupt(IRQ_A);
    gic.set_pending(IRQ_B);
    gic.set_pending(IRQ_A);

    assert_eq!(gic.next_pending_enabled(), Some(IRQ_A));
}

#[test]
fn legacy_private_interrupt_apis_address_cpu_zero() {
    let mut gic = Gicv3::with_cpu_count(2);
    let sgi = 5;

    gic.enable_interrupt(sgi);
    gic.set_pending(sgi);

    assert_eq!(gic.next_pending_enabled(), Some(sgi));
    assert_eq!(gic.next_pending_enabled_for_cpu(1), None);
    assert_eq!(gic.enable_word(0), 1 << sgi);
    assert_eq!(gic.pending_word(0), 1 << sgi);

    gic.clear_pending(sgi);
    assert!(!gic.is_pending(sgi));
}

#[test]
fn default_trait_constructs_the_legacy_single_cpu_shape() {
    let gic = Gicv3::default();

    assert_eq!(gic.cpu_count(), 1);
    assert_eq!(gic.rtyper, GICR_TYPER_LAST);
}
