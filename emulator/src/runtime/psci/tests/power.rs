use super::*;

#[test]
fn version_reports_psci_0_2() {
    let mut machine = Machine::new(2);

    psci_call(&mut machine, PSCI_VERSION, 0, 0, 0);

    assert_eq!(machine.cpus[0].regs.x(0), 2);
}

#[test]
fn cpu_suspend_returns_from_the_supported_retention_state() {
    let mut machine = Machine::new(2);
    let pc = RAM_BASE + 0x6000;
    machine.cpus[0].regs.pc = pc;

    psci_call(&mut machine, PSCI_CPU_SUSPEND64, 0, 0, 0);

    assert_eq!(machine.cpus[0].regs.x(0), PSCI_SUCCESS as u64);
    assert_eq!(machine.cpus[0].regs.pc, pc + INSTRUCTION_SIZE);
    assert_eq!(machine.cpus[0].lifecycle, CpuLifecycle::Runnable);
}

#[test]
fn cpu_suspend_can_downgrade_core_powerdown_to_standby() {
    let mut machine = Machine::new(2);
    let entry = RAM_BASE + 0x7000;
    machine.bus.mem.write(entry, 4, NOP);

    psci_call(
        &mut machine,
        PSCI_CPU_SUSPEND64,
        PSCI_SUSPEND_POWERDOWN,
        entry,
        0x1234,
    );

    assert_eq!(machine.cpus[0].regs.x(0), PSCI_SUCCESS as u64);
    assert_eq!(machine.cpus[0].lifecycle, CpuLifecycle::Runnable);
}

#[test]
fn cpu_suspend_rejects_unadvertised_states_and_invalid_entries() {
    let mut machine = Machine::new(2);

    psci_call(&mut machine, PSCI_CPU_SUSPEND64, 1, 0, 0);
    assert_eq!(
        machine.cpus[0].regs.x(0),
        psci_result(PSCI_INVALID_PARAMETERS)
    );

    psci_call(
        &mut machine,
        PSCI_CPU_SUSPEND64,
        PSCI_SUSPEND_POWERDOWN,
        RAM_BASE + 1,
        0,
    );
    assert_eq!(
        machine.cpus[0].regs.x(0),
        psci_result(PSCI_INVALID_PARAMETERS)
    );
}

#[test]
fn affinity_info_aggregates_the_flat_parent_hierarchy() {
    let mut machine = Machine::new(2);
    machine.cpus[0].lifecycle = CpuLifecycle::PoweredOff;

    for level in 1..=3 {
        psci_call(&mut machine, PSCI_AFFINITY_INFO64, 0, level, 0);
        assert_eq!(machine.cpus[0].regs.x(0), PSCI_AFFINITY_OFF as u64);
    }

    machine.cpus[1].lifecycle = CpuLifecycle::Runnable;
    for level in 1..=3 {
        psci_call(&mut machine, PSCI_AFFINITY_INFO64, 0xff, level, 0);
        assert_eq!(machine.cpus[0].regs.x(0), PSCI_AFFINITY_ON as u64);
    }

    psci_call(&mut machine, PSCI_AFFINITY_INFO64, 1 << 8, 1, 0);
    assert_eq!(
        machine.cpus[0].regs.x(0),
        psci_result(PSCI_INVALID_PARAMETERS)
    );
}

#[test]
fn system_off_stops_every_core_without_returning_to_the_caller() {
    let mut machine = Machine::new(2);
    let pc = RAM_BASE + 0x8000;
    machine.cpus[0].regs.pc = pc;
    machine.cpus[1].lifecycle = CpuLifecycle::Runnable;

    psci_call(&mut machine, PSCI_SYSTEM_OFF, 0, 0, 0);

    assert!(
        machine
            .cpus
            .iter()
            .all(|cpu| cpu.lifecycle == CpuLifecycle::PoweredOff)
    );
    assert_eq!(machine.cpus[0].regs.pc, pc);
    assert_eq!(machine.run(10), 0);
}

#[test]
fn system_reset_restores_the_configured_boot_image_and_persistent_devices() {
    let mut machine = Machine::new(2);
    let entry = RAM_BASE + 0xa000;
    let dtb = RAM_BASE + 0xb000;
    let marker = RAM_BASE + 0xc000;
    let kernel_word = 0x1400_0000u32;
    let dtb_word = 0xd00d_feedu32;
    machine.configure_system_reset(
        entry,
        dtb,
        vec![
            (entry, kernel_word.to_le_bytes().to_vec()),
            (dtb, dtb_word.to_le_bytes().to_vec()),
        ],
    );
    machine.bus.mem.write(entry, 4, 0);
    machine.bus.mem.write(dtb, 4, 0);
    machine.bus.mem.write(marker, 4, 0xfeed_face);
    machine.bus.virtio_blk.set_image(&[1, 2, 3]);
    machine.bus.uart.output.extend_from_slice(b"before reset\n");
    machine.bus.gic.ctld = 1;
    machine.cpus[0].regs.set_x(9, 0xdead);
    machine.cpus[1].lifecycle = CpuLifecycle::Runnable;
    machine.cpus[1].regs.set_x(9, 0xbeef);

    psci_call(&mut machine, PSCI_SYSTEM_RESET, 0, 0, 0);

    assert_eq!(machine.cpus[0].lifecycle, CpuLifecycle::Runnable);
    assert_eq!(machine.cpus[0].regs.pc, entry);
    assert_eq!(machine.cpus[0].regs.x(0), dtb);
    assert_eq!(machine.cpus[0].regs.x(9), 0);
    assert_eq!(machine.cpus[1].lifecycle, CpuLifecycle::PoweredOff);
    assert_eq!(machine.cpus[1].regs.x(9), 0);
    assert_eq!(machine.bus.mem.read(entry, 4), Some(kernel_word as u64));
    assert_eq!(machine.bus.mem.read(dtb, 4), Some(dtb_word as u64));
    assert_eq!(machine.bus.mem.read(marker, 4), Some(0));
    assert_eq!(machine.bus.virtio_blk.read(0x100, 8), Some(1));
    assert_eq!(machine.bus.uart.output, b"before reset\n");
    assert_eq!(machine.bus.gic.ctld, 0);
    assert_eq!(machine.virtual_time, 0);
}

#[test]
fn optional_and_unknown_functions_remain_not_supported() {
    let mut machine = Machine::new(2);

    psci_call(&mut machine, PSCI_MIGRATE_INFO_TYPE, 0, 0, 0);
    assert_eq!(machine.cpus[0].regs.x(0), psci_result(PSCI_NOT_SUPPORTED));

    psci_call(&mut machine, 0x8400_ffff, 0, 0, 0);
    assert_eq!(machine.cpus[0].regs.x(0), psci_result(PSCI_NOT_SUPPORTED));
}
