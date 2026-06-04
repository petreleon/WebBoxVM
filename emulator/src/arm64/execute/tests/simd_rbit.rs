use super::*;

#[test]
fn simd_rbit_reverses_bits_in_each_byte() {
    let (mut cpu, mut bus) = setup();
    cpu.simd[8] = 0x0123_4567_89ab_cdef_fedc_ba98_7654_3210;

    execute(&mut cpu, &mut bus, decode(0x6E60_5908).unwrap()).unwrap();

    assert_eq!(cpu.simd[8], 0x80c4_a2e6_91d5_b3f7_7f3b_5d19_6e2a_4c08);
}

#[test]
fn simd_rbit_8b_clears_upper_half() {
    let (mut cpu, mut bus) = setup();
    cpu.simd[0] = 0x0123_4567_89ab_cdef_fedc_ba98_7654_3210;

    execute(&mut cpu, &mut bus, decode(0x2E60_5800).unwrap()).unwrap();

    assert_eq!(cpu.simd[0], 0x7f3b_5d19_6e2a_4c08);
}
