use super::*;

#[test]
fn bounds_cannot_be_zero_or_exceed_the_standard_lane() {
    for (steps, chunk, seconds) in [
        (0, 1, 1),
        (1, 0, 1),
        (1, 1, 0),
        (MAX_STEPS + 1, 1, 1),
        (1, CHUNK_STEPS + 1, 1),
        (1, 1, TIMEOUT_SECS + 1),
    ] {
        assert!(Budget::new(steps, chunk, seconds).is_err());
    }
}

#[test]
fn idle_vm_still_exhausts_a_finite_number_of_chunks() {
    let mut budget = Budget::new(7, 3, 1).unwrap();
    for _ in 0..3 {
        let requested = budget.next(Duration::ZERO).unwrap();
        budget.record(requested, 0).unwrap();
    }
    assert_eq!(budget.steps, 0);
    assert_eq!(budget.chunks, 3);
    assert!(budget.next(Duration::ZERO).is_err());
}

#[test]
fn last_chunk_uses_only_the_remaining_instruction_budget() {
    let mut budget = Budget::new(7, 3, 1).unwrap();
    for expected in [3, 3, 1] {
        assert_eq!(budget.next(Duration::ZERO).unwrap(), expected);
        budget.record(expected, expected).unwrap();
    }
    assert_eq!(budget.steps, 7);
    assert!(budget.next(Duration::ZERO).is_err());
}

#[test]
fn configured_timeout_applies_after_a_chunk_too() {
    let budget = Budget::new(10, 1, 1).unwrap();
    assert!(budget.check_time(Duration::from_millis(999)).is_ok());
    assert!(budget.check_time(Duration::from_secs(1)).is_err());
    assert!(budget.next(Duration::from_secs(1)).is_err());
}

#[test]
fn vm_cannot_overrun_the_requested_instruction_count() {
    let mut budget = Budget::new(7, 3, 1).unwrap();
    assert!(budget.record(3, 4).is_err());
    assert_eq!((budget.steps, budget.chunks), (0, 0));
}
