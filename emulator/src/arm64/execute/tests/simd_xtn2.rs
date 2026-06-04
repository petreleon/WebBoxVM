use super::*;

#[test]
fn simd_xtn2_writes_upper_half_and_preserves_lower() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[15] = 0xaaaa_aaaa_aaaa_aaaa_1122_3344_5566_7788;
    cpu.simd[31] = 0x99aa_bbcc_ddee_ff00_1234_5678_9abc_def0;
    execute(&mut cpu, &mut bus, decode(0x4EA1_2BEF).unwrap()).unwrap();
    assert_eq!(cpu.simd[15], 0xddee_ff00_9abc_def0_1122_3344_5566_7788);

    cpu.simd[0] = 0xffff_ffff_ffff_ffff_0102_0304_0506_0708;
    cpu.simd[1] = 0x00ff_00ee_00dd_00cc_00bb_00aa_0099_0088;
    execute(&mut cpu, &mut bus, decode(0x4E21_2820).unwrap()).unwrap();
    assert_eq!(cpu.simd[0], 0xffee_ddcc_bbaa_9988_0102_0304_0506_0708);
}
