use super::simd_helpers::*;
use super::*;

#[test]
fn scalar_fp_compare_select_and_widening_simd_ops() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[31] = 0.0f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E60_23E8).unwrap()).unwrap(); // fcmp d31, #0.0
    assert!(cpu.pstate.z());
    assert!(cpu.pstate.c());

    cpu.simd[29] = 3.0f64.to_bits() as u128;
    cpu.simd[25] = 4.0f64.to_bits() as u128;
    execute(&mut cpu, &mut bus, decode(0x1E79_23B0).unwrap()).unwrap(); // fcmpe d29, d25
    assert!(cpu.pstate.n());
    assert!(!cpu.pstate.c());

    cpu.simd[31] = 11.0f64.to_bits() as u128;
    cpu.simd[30] = 22.0f64.to_bits() as u128;
    cpu.pstate.set_nzcv(false, true, true, false);
    execute(&mut cpu, &mut bus, decode(0x1E7E_0FFF).unwrap()).unwrap(); // fcsel d31, d31, d30, eq
    assert_eq!(f64_lane(&cpu, 31), 11.0);

    cpu.pstate.set_nzcv(false, false, true, false);
    execute(&mut cpu, &mut bus, decode(0x1E7E_0FFF).unwrap()).unwrap();
    assert_eq!(f64_lane(&cpu, 31), 22.0);

    cpu.simd[0] = 2.0f64.to_bits() as u128;
    cpu.simd[13] = 3.0f64.to_bits() as u128;
    cpu.pstate.set_nzcv(false, true, true, false);
    execute(&mut cpu, &mut bus, decode(0x1E6D_0400).unwrap()).unwrap(); // fccmp d0, d13, #0, eq
    assert!(cpu.pstate.n());
    assert!(!cpu.pstate.z());
    assert!(!cpu.pstate.c());
    assert!(!cpu.pstate.v());

    cpu.simd[15] = 1.0f64.to_bits() as u128;
    cpu.simd[13] = 0.0f64.to_bits() as u128;
    cpu.pstate.set_nzcv(false, false, true, false);
    execute(&mut cpu, &mut bus, decode(0x1E6D_05E4).unwrap()).unwrap(); // fccmp d15, d13, #4, eq
    assert!(!cpu.pstate.n());
    assert!(cpu.pstate.z());
    assert!(!cpu.pstate.c());
    assert!(!cpu.pstate.v());

    cpu.simd[0] = 4.0f32.to_bits() as u128;
    cpu.simd[31] = 4.0f32.to_bits() as u128;
    cpu.pstate.set_nzcv(false, false, true, false);
    execute(&mut cpu, &mut bus, decode(0x1E3F_8400).unwrap()).unwrap(); // fccmp s0, s31, #0, hi
    assert!(!cpu.pstate.n());
    assert!(cpu.pstate.z());
    assert!(cpu.pstate.c());
    assert!(!cpu.pstate.v());

    cpu.simd[0] = u128::MAX;
    execute(&mut cpu, &mut bus, decode(0x6F00_E400).unwrap()).unwrap(); // movi v0.2d, #0
    assert_eq!(cpu.simd[0], 0);

    cpu.simd[31] = 0xffff_ffff_0000_0002;
    execute(&mut cpu, &mut bus, decode(0x2F20_A7FF).unwrap()).unwrap(); // ushll v31.2d, v31.2s, #0
    assert_eq!(cpu.simd[31], 0x0000_0000_ffff_ffff_0000_0000_0000_0002);

    cpu.simd[31] = 0x8000_0001_0000_0002;
    execute(&mut cpu, &mut bus, decode(0x0F20_A7FF).unwrap()).unwrap(); // sshll v31.2d, v31.2s, #0
    assert_eq!(cpu.simd[31], 0xffff_ffff_8000_0001_0000_0000_0000_0002);

    cpu.simd[30] = vector_bytes(0);
    execute(&mut cpu, &mut bus, decode(0x2E21_3BDE).unwrap()).unwrap(); // shll v30.8h, v30.8b, #8
    assert_eq!(cpu.simd[30], 0x0700_0600_0500_0400_0300_0200_0100_0000);

    cpu.simd[16] = 0;
    cpu.simd[30] = vector_bytes(0);
    execute(&mut cpu, &mut bus, decode(0x6E21_3BD0).unwrap()).unwrap(); // shll2 v16.8h, v30.16b, #8
    assert_eq!(cpu.simd[16], 0x0f00_0e00_0d00_0c00_0b00_0a00_0900_0800);

    cpu.simd[22] = 0x0000_0000_8000_0001;
    cpu.simd[25] = 0xffff_ffff_0000_0002;
    execute(&mut cpu, &mut bus, decode(0x0EB9_02D0).unwrap()).unwrap(); // saddl v16.2d, v22.2s, v25.2s
    assert_eq!(cpu.simd[16], 0xffff_ffff_ffff_ffff_ffff_ffff_8000_0003);

    cpu.simd[22] = 0x0000_0004_ffff_fffe_0000_0002_0000_0001;
    cpu.simd[25] = 0xffff_fffd_0000_0005_0000_0006_0000_0007;
    execute(&mut cpu, &mut bus, decode(0x4EB9_02D6).unwrap()).unwrap(); // saddl2 v22.2d, v22.4s, v25.4s
    assert_eq!(cpu.simd[22], 0x0000_0000_0000_0001_0000_0000_0000_0003);

    cpu.simd[31] = 0x0000_0005_ffff_fffe;
    cpu.simd[29] = 0x0000_0007_0000_0001;
    execute(&mut cpu, &mut bus, decode(0x2EBD_23FE).unwrap()).unwrap(); // usubl v30.2d, v31.2s, v29.2s
    assert_eq!(cpu.simd[30], 0xffff_ffff_ffff_fffe_0000_0000_ffff_fffd);

    cpu.simd[31] = 0x0000_0000_0000_0003_0000_0002_0000_0001;
    cpu.simd[29] = 0xffff_ffff_0000_0005_0000_0000_0000_0000;
    execute(&mut cpu, &mut bus, decode(0x6EBD_23FF).unwrap()).unwrap(); // usubl2 v31.2d, v31.4s, v29.4s
    assert_eq!(cpu.simd[31], 0xffff_ffff_0000_0001_ffff_ffff_ffff_fffe);

    cpu.simd[6] = (100u128 << 64) | 10;
    cpu.simd[28] = (4u128 << 32) | 3;
    cpu.simd[0] = 5;
    execute(&mut cpu, &mut bus, decode(0x2F80_2386).unwrap()).unwrap(); // umlal v6.2d, v28.2s, v0.s[0]
    assert_eq!(cpu.simd[6], (120u128 << 64) | 25);

    cpu.simd[6] = (100u128 << 64) | 10;
    cpu.simd[28] = (4u128 << 96) | (3u128 << 64) | (2u128 << 32) | 1;
    cpu.simd[0] = 5;
    execute(&mut cpu, &mut bus, decode(0x6F80_2386).unwrap()).unwrap(); // umlal2 v6.2d, v28.4s, v0.s[0]
    assert_eq!(cpu.simd[6], (120u128 << 64) | 25);

    cpu.simd[29] = 0x0000_0190_ffff_fed4_0000_00c8_0000_0064;
    cpu.simd[30] = 0x0000_0000_0000_0000_fffc_0003_fffe_0001;
    execute(&mut cpu, &mut bus, decode(0x0E7E_33BD).unwrap()).unwrap(); // ssubw v29.4s, v29.4s, v30.4h
    assert_eq!(cpu.simd[29], 0x0000_0194_ffff_fed1_0000_00ca_0000_0063);

    cpu.simd[31] = 0x0000_000a_0000_0000_ffff_fc18_0000_03e8;
    cpu.simd[30] = 0xfff8_0007_fffa_0005_0000_0000_0000_0000;
    execute(&mut cpu, &mut bus, decode(0x4E7E_33FF).unwrap()).unwrap(); // ssubw2 v31.4s, v31.4s, v30.8h
    assert_eq!(cpu.simd[31], 0x0000_0012_ffff_fff9_ffff_fc1e_0000_03e3);

    cpu.simd[30] = 0x8000_0000_0000_0000_ffff_ffff_ffff_fffb;
    execute(&mut cpu, &mut bus, decode(0x4EE0_BBDE).unwrap()).unwrap(); // abs v30.2d, v30.2d
    assert_eq!(cpu.simd[30], 0x8000_0000_0000_0000_0000_0000_0000_0005);

    cpu.simd[24] = 0xffff_ffff_ffff_fff7;
    execute(&mut cpu, &mut bus, decode(0x5EE0_BB00).unwrap()).unwrap(); // abs d0, d24
    assert_eq!(cpu.simd[0], 9);
}
