use super::*;

#[test]
fn boot_el3() {
    let p = ProcessorState::new();
    assert_eq!(p.el(), MAX_EL);
}

#[test]
fn nzcv_roundtrip() {
    let mut p = ProcessorState::new();
    p.set_nzcv(true, false, true, false);
    assert!(p.n() && !p.z() && p.c() && !p.v());
}

#[test]
fn el_transition() {
    let p = ProcessorState::new().with_el(1);
    assert_eq!(p.el(), 1);
}

#[test]
fn el1h_masked_selects_sp_el1_and_masks_full_daif() {
    let p = ProcessorState::el1h_masked();

    assert_eq!(p.el(), 1);
    assert!(p.sp_select());
    assert!(p.all_exceptions_masked());
    assert_eq!(p.to_u64(), PSTATE_EL1H | PSTATE_DAIF_MASK);
}

#[test]
fn sp_select_roundtrips_without_changing_el() {
    let p = ProcessorState::new().with_el(1).with_sp_select(true);
    assert_eq!(p.el(), 1);
    assert!(p.sp_select());

    let p = p.with_sp_select(false);
    assert_eq!(p.el(), 1);
    assert!(!p.sp_select());
}
