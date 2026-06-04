use super::*;

#[test]
fn sve_predicate_forms_update_predicates_and_flags() {
    let (mut cpu, mut bus) = setup();

    execute(&mut cpu, &mut bus, decode(0x2518_E3E3).unwrap()).unwrap(); // ptrue p3.b
    assert!((0..16).all(|bit| pred_bit(&cpu, 3, bit)));
    assert!(!pred_bit(&cpu, 3, 16));

    execute(&mut cpu, &mut bus, decode(0x25D8_E3E1).unwrap()).unwrap(); // ptrue p1.d
    assert!(pred_bit(&cpu, 1, 0));
    assert!(pred_bit(&cpu, 1, 8));
    assert!(!pred_bit(&cpu, 1, 1));
    assert!(!pred_bit(&cpu, 1, 9));

    execute(&mut cpu, &mut bus, decode(0x25D8_E065).unwrap()).unwrap(); // ptrue p5.d, vl3
    assert_eq!(cpu.sve_pred[5], [0; 4]);

    cpu.sve_vl_bytes = 32;
    execute(&mut cpu, &mut bus, decode(0x2558_E064).unwrap()).unwrap(); // ptrue p4.h, vl3
    assert!(pred_bit(&cpu, 4, 0));
    assert!(pred_bit(&cpu, 4, 2));
    assert!(pred_bit(&cpu, 4, 4));
    assert!(!pred_bit(&cpu, 4, 6));

    execute(&mut cpu, &mut bus, decode(0x2599_E061).unwrap()).unwrap(); // ptrues p1.s, vl3
    assert!(pred_bit(&cpu, 1, 0));
    assert!(pred_bit(&cpu, 1, 4));
    assert!(pred_bit(&cpu, 1, 8));
    assert!(!pred_bit(&cpu, 1, 12));
    assert!(cpu.pstate.n());
    assert!(!cpu.pstate.z());
    assert!(!cpu.pstate.c());

    execute(&mut cpu, &mut bus, decode(0x2599_E125).unwrap()).unwrap(); // ptrues p5.s, vl16
    assert_eq!(cpu.sve_pred[5], [0; 4]);
    assert!(!cpu.pstate.n());
    assert!(cpu.pstate.z());
    assert!(cpu.pstate.c());

    cpu.sve_vl_bytes = 16;
    execute(&mut cpu, &mut bus, decode(0x2518_E3E0).unwrap()).unwrap(); // ptrue p0.b
    execute(&mut cpu, &mut bus, decode(0x2518_E022).unwrap()).unwrap(); // ptrue p2.b, vl1

    execute(&mut cpu, &mut bus, decode(0x2550_C060).unwrap()).unwrap(); // ptest p0, p3.b
    assert!(cpu.pstate.n());
    assert!(!cpu.pstate.z());
    assert!(!cpu.pstate.c());
    assert!(!cpu.pstate.v());

    execute(&mut cpu, &mut bus, decode(0x2550_C040).unwrap()).unwrap(); // ptest p0, p2.b
    assert!(cpu.pstate.n());
    assert!(!cpu.pstate.z());
    assert!(cpu.pstate.c());

    execute(&mut cpu, &mut bus, decode(0x2518_E3E1).unwrap()).unwrap(); // ptrue p1.b
    execute(&mut cpu, &mut bus, decode(0x2543_4447).unwrap()).unwrap(); // ands p7.b, p1/z, p2.b, p3.b
    assert!(pred_bit(&cpu, 7, 0));
    assert!(!pred_bit(&cpu, 7, 1));
    assert!(cpu.pstate.n());
    assert!(!cpu.pstate.z());
    assert!(cpu.pstate.c());

    execute(&mut cpu, &mut bus, decode(0x25C3_4448).unwrap()).unwrap(); // orrs p8.b, p1/z, p2.b, p3.b
    assert!((0..16).all(|bit| pred_bit(&cpu, 8, bit)));
    assert!(cpu.pstate.n());
    assert!(!cpu.pstate.z());
    assert!(!cpu.pstate.c());
}
