use super::simd_helpers::*;
use super::*;

#[test]
fn simd_uzp2_selects_odd_elements_from_both_sources() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[30] = 0x0000_0004_0000_0003_0000_0002_0000_0001;
    cpu.simd[28] = 0x0000_0008_0000_0007_0000_0006_0000_0005;
    execute(&mut cpu, &mut bus, decode(0x4E9C_5BDE).unwrap()).unwrap();
    assert_eq!(cpu.simd[30], 0x0000_0008_0000_0006_0000_0004_0000_0002);

    cpu.simd[29] = 0x0008_0007_0006_0005_0004_0003_0002_0001;
    cpu.simd[27] = 0x1008_1007_1006_1005_1004_1003_1002_1001;
    execute(&mut cpu, &mut bus, decode(0x4E5B_5BBD).unwrap()).unwrap();
    assert_eq!(cpu.simd[29], 0x1008_1006_1004_1002_0008_0006_0004_0002);
}

#[test]
fn simd_trn2_interleaves_odd_elements_from_both_sources() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[8] = vector_bytes(0);
    cpu.simd[12] = vector_bytes(0x80);
    execute(&mut cpu, &mut bus, decode(0x4E0C_690B).unwrap()).unwrap();
    assert_eq!(cpu.simd[11], 0x8f0f_8d0d_8b0b_8909_8707_8505_8303_8101);

    cpu.simd[12] = 0x0000_0004_0000_0003_0000_0002_0000_0001;
    cpu.simd[14] = 0x1004_0000_1003_0000_1002_0000_1001_0000;
    execute(&mut cpu, &mut bus, decode(0x4ECE_698B).unwrap()).unwrap();
    assert_eq!(cpu.simd[11], 0x1004_0000_1003_0000_0000_0004_0000_0003);
}
