use super::*;

const GICR_IGROUPR0: u64 = 0x1_0080;
const GICR_ISENABLER0: u64 = 0x1_0100;
const GICR_ICENABLER0: u64 = 0x1_0180;
const GICR_ISPENDR0: u64 = 0x1_0200;
const GICR_ICPENDR0: u64 = 0x1_0280;
const GICR_IPRIORITYR0: u64 = 0x1_0400;

fn frame(cpu_id: usize, register: u64) -> u64 {
    cpu_id as u64 * GICR_FRAME_SIZE + register
}

#[test]
fn zero_cpu_count_is_normalised_to_one() {
    let gic = Gicv3::with_cpu_count(0);

    assert_eq!(gic.cpu_count(), 1);
    assert_eq!(gic.gicr_read(0x0008, 8), Some(GICR_TYPER_LAST));
}

#[test]
fn redistributor_frames_have_unique_affinity_and_only_final_is_last() {
    let gic = Gicv3::with_cpu_count(3);

    let typer0 = gic.gicr_read(frame(0, 0x0008), 8).unwrap();
    let typer1 = gic.gicr_read(frame(1, 0x0008), 8).unwrap();
    let typer2 = gic.gicr_read(frame(2, 0x0008), 8).unwrap();

    assert_eq!(typer0, 0);
    assert_eq!(typer1, (1 << 32) | (1 << 8));
    assert_eq!(typer2, (2 << 32) | (2 << 8) | GICR_TYPER_LAST);
    assert_eq!(gic.gicr_read(frame(1, 0x000C), 4), Some(1));
    assert_eq!(gic.gicr_read(frame(3, 0x0008), 8), Some(0));
}

#[test]
fn redistributor_control_registers_do_not_alias_between_cpus() {
    let mut gic = Gicv3::with_cpu_count(2);

    gic.gicr_write(frame(0, 0x0000), 0x11, 4);
    gic.gicr_write(frame(1, 0x0000), 0x22, 4);
    gic.gicr_write(frame(0, 0x0014), 0x33, 4);
    gic.gicr_write(frame(1, 0x0014), 0x44, 4);

    assert_eq!(gic.gicr_read(frame(0, 0x0000), 4), Some(0x11));
    assert_eq!(gic.gicr_read(frame(1, 0x0000), 4), Some(0x22));
    assert_eq!(gic.gicr_read(frame(0, 0x0014), 4), Some(0x33));
    assert_eq!(gic.gicr_read(frame(1, 0x0014), 4), Some(0x44));
    assert_eq!(gic.rctlr, 0x11);
    assert_eq!(gic.rwaker, 0x33);
}

#[test]
fn sgi_ppi_configuration_is_banked_by_redistributor() {
    let mut gic = Gicv3::with_cpu_count(2);
    let private_irq = 5u32;
    let bit = 1u64 << private_irq;

    gic.gicr_write(frame(1, GICR_IGROUPR0), bit, 4);
    gic.gicr_write(frame(1, GICR_ISPENDR0), bit, 4);
    gic.gicr_write(frame(1, GICR_ISENABLER0), bit, 4);
    gic.gicr_write(frame(1, GICR_IPRIORITYR0 + 4), 0x4433_2211, 4);

    assert_eq!(gic.next_pending_enabled_for_cpu(0), None);
    assert_eq!(gic.next_pending_enabled_for_cpu(1), Some(private_irq));
    assert_eq!(gic.gicr_read(frame(1, GICR_IGROUPR0), 4), Some(bit));
    assert_eq!(gic.gicr_read(frame(0, GICR_IGROUPR0), 4), Some(0));
    assert_eq!(
        gic.gicr_read(frame(1, GICR_IPRIORITYR0 + 4), 4),
        Some(0x4433_2211)
    );
    assert_eq!(gic.gicr_read(frame(0, GICR_IPRIORITYR0 + 4), 4), Some(0));

    gic.gicr_write(frame(1, GICR_ICENABLER0), bit, 4);
    assert_eq!(gic.next_pending_enabled_for_cpu(1), None);
    assert_eq!(gic.gicr_read(frame(1, GICR_ICENABLER0), 4), Some(0));
    assert_eq!(gic.gicr_read(frame(0, GICR_ISPENDR0), 4), Some(0));

    gic.gicr_write(frame(1, GICR_ICPENDR0), bit, 4);
    assert!(!gic.is_pending_for_cpu(1, private_irq));
}

#[test]
fn distributor_group_register_uses_architectural_offset() {
    let mut gic = Gicv3::new();

    gic.gicd_write(0x0084, 0xa5a5_5a5a, 4);
    assert_eq!(gic.group[1], 0xa5a5_5a5a);
    assert_eq!(gic.gicd_read(0x0084, 4), Some(0xa5a5_5a5a));

    gic.gicd_write(0x0804, 0xffff_ffff, 4);
    assert_eq!(gic.group[1], 0xa5a5_5a5a);
    assert_eq!(gic.gicd_read(0x0804, 4), Some(0));
}

#[test]
fn private_interrupt_beats_a_lower_priority_class_spi_by_intid() {
    let mut gic = Gicv3::with_cpu_count(2);
    gic.enable_interrupt_for_cpu(1, 29);
    gic.set_pending_for_cpu(1, 29);
    gic.enable_interrupt(48);
    gic.set_interrupt_route(48, gic.cpu_affinity(1).unwrap());
    gic.set_pending(48);

    assert_eq!(gic.next_pending_enabled_for_cpu(1), Some(29));
    gic.clear_pending_for_cpu(1, 29);
    assert_eq!(gic.next_pending_enabled_for_cpu(1), Some(48));
}
