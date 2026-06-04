use super::*;

#[test]
fn sve_logical_vectors_and_predicated_forms_update_expected_lanes() {
    let (mut cpu, mut bus) = setup();
    set_z_elem(&mut cpu, 29, 0, 0xF0F0);
    set_z_elem(&mut cpu, 3, 0, 0x0FF0);
    execute(&mut cpu, &mut bus, decode(0x0423_33BD).unwrap()).unwrap();
    assert_eq!(z_elem(&cpu, 29, 0), 0x00F0);

    execute(&mut cpu, &mut bus, decode(0x25D8_E023).unwrap()).unwrap(); // ptrue p3.d, vl1
    set_z_elem(&mut cpu, 1, 0, 0xF0);
    set_z_elem(&mut cpu, 1, 1, 0xF0);
    set_z_elem(&mut cpu, 31, 0, 0x0F);
    set_z_elem(&mut cpu, 31, 1, 0x0F);
    execute(&mut cpu, &mut bus, decode(0x04D8_0FE1).unwrap()).unwrap();
    assert_eq!(z_elem(&cpu, 1, 0), 0xFF);
    assert_eq!(z_elem(&cpu, 1, 1), 0xF0);

    set_z_elem(&mut cpu, 1, 0, 0xFF);
    execute(&mut cpu, &mut bus, decode(0x04D9_0FE1).unwrap()).unwrap();
    assert_eq!(z_elem(&cpu, 1, 0), 0xF0);
    assert_eq!(z_elem(&cpu, 1, 1), 0xF0);

    set_z_elem(&mut cpu, 31, 0, 0xF0F0);
    set_z_elem(&mut cpu, 31, 1, 0xF0F0);
    set_z_elem(&mut cpu, 30, 0, 0x0FF0);
    execute(&mut cpu, &mut bus, decode(0x04DA_0FDF).unwrap()).unwrap();
    assert_eq!(z_elem(&cpu, 31, 0), 0x00F0);
    assert_eq!(z_elem(&cpu, 31, 1), 0xF0F0);
}

#[test]
fn sve_predicate_eor_zeroes_inactive_and_sets_flags_for_eors() {
    let (mut cpu, mut bus) = setup();
    execute(&mut cpu, &mut bus, decode(0x2518_E3E0).unwrap()).unwrap(); // ptrue p0.b
    execute(&mut cpu, &mut bus, decode(0x2518_E022).unwrap()).unwrap(); // ptrue p2.b, vl1
    execute(&mut cpu, &mut bus, decode(0x2500_4242).unwrap()).unwrap();
    assert!(!pred_bit(&cpu, 2, 0));
    assert!((1..16).all(|bit| pred_bit(&cpu, 2, bit)));
    assert!(!pred_bit(&cpu, 2, 16));

    execute(&mut cpu, &mut bus, decode(0x2518_E022).unwrap()).unwrap(); // ptrue p2.b, vl1
    execute(&mut cpu, &mut bus, decode(0x2540_4242).unwrap()).unwrap();
    assert!(!cpu.pstate.n());
    assert!(!cpu.pstate.z());
    assert!(!cpu.pstate.c());
}

#[test]
fn sve_predicate_inverted_logicals_zero_inactive_and_set_flags() {
    let (mut cpu, mut bus) = setup();
    execute(&mut cpu, &mut bus, decode(0x2518_E3E0).unwrap()).unwrap(); // ptrue p0.b
    execute(&mut cpu, &mut bus, decode(0x2518_E022).unwrap()).unwrap(); // ptrue p2.b, vl1
    execute(&mut cpu, &mut bus, decode(0x2518_E041).unwrap()).unwrap(); // ptrue p1.b, vl2
    execute(&mut cpu, &mut bus, decode(0x2518_E063).unwrap()).unwrap(); // ptrue p3.b, vl3
    execute(&mut cpu, &mut bus, decode(0x2518_E02F).unwrap()).unwrap(); // ptrue p15.b, vl1

    execute(&mut cpu, &mut bus, decode(0x250F_4413).unwrap()).unwrap(); // bic p3.b, p1/z, p0.b, p15.b
    assert!(!pred_bit(&cpu, 3, 0));
    assert!(pred_bit(&cpu, 3, 1));
    assert!(!pred_bit(&cpu, 3, 2));

    execute(&mut cpu, &mut bus, decode(0x2518_E063).unwrap()).unwrap(); // ptrue p3.b, vl3
    execute(&mut cpu, &mut bus, decode(0x25C2_4073).unwrap()).unwrap(); // orns p3.b, p0/z, p3.b, p2.b
    assert!(pred_bit(&cpu, 3, 0));
    assert!(pred_bit(&cpu, 3, 15));
    assert!(cpu.pstate.n());
    assert!(!cpu.pstate.z());
    assert!(!cpu.pstate.c());
}
