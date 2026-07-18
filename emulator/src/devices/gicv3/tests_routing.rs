use super::*;

const SPI: u32 = 48;

fn irouter(int_id: u32) -> u64 {
    0x6000 + int_id as u64 * 8
}

fn sgi1r(int_id: u32, target_list: u16) -> u64 {
    ((int_id as u64) << 24) | target_list as u64
}

#[test]
fn spis_default_to_cpu_zero_and_can_be_retargeted() {
    let mut gic = Gicv3::with_cpu_count(3);
    gic.enable_interrupt(SPI);
    gic.set_pending(SPI);

    assert_eq!(gic.next_pending_enabled_for_cpu(0), Some(SPI));
    assert_eq!(gic.next_pending_enabled_for_cpu(1), None);

    let cpu2 = gic.cpu_affinity(2).unwrap();
    gic.gicd_write(irouter(SPI), cpu2, 8);

    assert_eq!(gic.gicd_read(irouter(SPI), 8), Some(cpu2));
    assert_eq!(gic.interrupt_route(SPI), Some(cpu2));
    assert_eq!(gic.next_pending_enabled_for_cpu(0), None);
    assert_eq!(gic.next_pending_enabled_for_cpu(2), Some(SPI));
}

#[test]
fn irouter_supports_split_32_bit_accesses() {
    let mut gic = Gicv3::with_cpu_count(2);
    let address = irouter(SPI);

    gic.gicd_write(address, 1, 4);
    gic.gicd_write(address + 4, 0xaa, 4);

    assert_eq!(gic.gicd_read(address, 4), Some(1));
    assert_eq!(gic.gicd_read(address + 4, 4), Some(0xaa));
    assert_eq!(gic.gicd_read(address, 8), Some(0x0000_00aa_0000_0001));
}

#[test]
fn irouter_for_private_interrupts_is_read_as_zero_and_write_ignored() {
    let mut gic = Gicv3::with_cpu_count(2);
    let private_address = irouter(7);

    gic.gicd_write(private_address, 1, 8);

    assert_eq!(gic.gicd_read(private_address, 8), Some(0));
    assert_eq!(gic.interrupt_route(7), None);
}

#[test]
fn target_list_sgi_is_banked_and_acknowledged_independently() {
    let mut gic = Gicv3::with_cpu_count(4);
    let sgi = 7;
    for cpu_id in 0..4 {
        gic.enable_interrupt_for_cpu(cpu_id, sgi);
    }

    let targets = gic.route_sgi1r(0, sgi1r(sgi, (1 << 1) | (1 << 3)));

    assert_eq!(targets, 2);
    assert!(!gic.is_pending_for_cpu(0, sgi));
    assert!(gic.is_pending_for_cpu(1, sgi));
    assert!(!gic.is_pending_for_cpu(2, sgi));
    assert!(gic.is_pending_for_cpu(3, sgi));
    assert_eq!(gic.next_pending_enabled_for_cpu(1), Some(sgi));
    assert_eq!(gic.next_pending_enabled_for_cpu(3), Some(sgi));

    gic.clear_pending_for_cpu(1, sgi);
    assert!(!gic.is_pending_for_cpu(1, sgi));
    assert!(gic.is_pending_for_cpu(3, sgi));
}

#[test]
fn explicit_target_list_may_include_the_sender() {
    let mut gic = Gicv3::with_cpu_count(2);
    let sgi = 3;

    assert_eq!(gic.route_sgi1r(0, sgi1r(sgi, 1)), 1);
    assert!(gic.is_pending_for_cpu(0, sgi));
    assert!(!gic.is_pending_for_cpu(1, sgi));
}

#[test]
fn irm_broadcast_excludes_the_sender() {
    let mut gic = Gicv3::with_cpu_count(4);
    let sgi = 9;
    let irm = (sgi as u64) << 24 | 1 << 40;

    assert_eq!(gic.route_sgi1r(2, irm), 3);
    assert!(gic.is_pending_for_cpu(0, sgi));
    assert!(gic.is_pending_for_cpu(1, sgi));
    assert!(!gic.is_pending_for_cpu(2, sgi));
    assert!(gic.is_pending_for_cpu(3, sgi));
}

#[test]
fn target_list_respects_cluster_affinity_and_range_selector() {
    let mut gic = Gicv3::with_cpu_count(17);
    let sgi = 4;
    let range_one_target_zero = sgi1r(sgi, 1) | 1 << 44;

    assert_eq!(gic.route_sgi1r(0, range_one_target_zero), 1);
    assert!(gic.is_pending_for_cpu(16, sgi));

    let wrong_aff1 = sgi1r(sgi, 1) | 1 << 16;
    assert_eq!(gic.route_sgi1r(0, wrong_aff1), 0);
}

#[test]
fn clearing_a_routed_spi_removes_the_single_global_pending_state() {
    let mut gic = Gicv3::with_cpu_count(2);
    gic.enable_interrupt(SPI);
    gic.set_interrupt_route(SPI, gic.cpu_affinity(1).unwrap());
    gic.set_pending(SPI);

    assert_eq!(gic.next_pending_enabled_for_cpu(1), Some(SPI));
    gic.clear_pending_for_cpu(1, SPI);
    assert_eq!(gic.next_pending_enabled_for_cpu(1), None);
    assert!(!gic.is_pending(SPI));
}

#[test]
fn irm_spi_is_claimable_by_exactly_one_cpu() {
    let mut gic = Gicv3::with_cpu_count(4);
    gic.enable_interrupt(SPI);
    gic.set_interrupt_route(SPI, 1 << 31);
    gic.set_pending(SPI);

    let eligible: Vec<_> = (0..gic.cpu_count())
        .filter(|&cpu_id| gic.next_pending_enabled_for_cpu(cpu_id) == Some(SPI))
        .collect();
    assert_eq!(eligible, vec![0]);

    let claims = (0..gic.cpu_count())
        .filter(|&cpu_id| gic.acknowledge_interrupt_for_cpu(cpu_id, SPI))
        .count();
    assert_eq!(claims, 1);
    assert!(gic.is_active_for_cpu(0, SPI));
}
