use super::simd_helpers::*;
use super::*;

#[test]
fn simd_sha512_three_register_ops_match_arm_round_functions() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[2] = u64x2([0x0123_4567_89ab_cdef, 0xfedc_ba98_7654_3210]);
    cpu.simd[5] = u64x2([0x0f1e_2d3c_4b5a_6978, 0x8877_6655_4433_2211]);
    cpu.simd[6] = u64x2([0x1122_3344_5566_7788, 0x99aa_bbcc_ddee_ff00]);
    let d_before = cpu.simd[2];
    execute(&mut cpu, &mut bus, decode(0xCE66_80A2).unwrap()).unwrap();
    assert_eq!(cpu.simd[2], sha512h_expected(d_before, cpu.simd[5], cpu.simd[6]));

    cpu.simd[0] = u64x2([0x7766_5544_3322_1100, 0xffee_ddcc_bbaa_9988]);
    cpu.simd[2] = u64x2([0x0123_4567_89ab_cdef, 0xfedc_ba98_7654_3210]);
    cpu.simd[3] = u64x2([0x1357_9bdf_2468_ace0, 0xfdb9_7531_eca8_6420]);
    let d_before = cpu.simd[2];
    execute(&mut cpu, &mut bus, decode(0xCE63_8402).unwrap()).unwrap();
    assert_eq!(cpu.simd[2], sha512h2_expected(d_before, cpu.simd[0], cpu.simd[3]));

    cpu.simd[7] = u64x2([0x0102_0304_0506_0708, 0x1112_1314_1516_1718]);
    cpu.simd[22] = u64x2([0x2122_2324_2526_2728, 0x3132_3334_3536_3738]);
    cpu.simd[23] = u64x2([0x4142_4344_4546_4748, 0x5152_5354_5556_5758]);
    let d_before = cpu.simd[23];
    execute(&mut cpu, &mut bus, decode(0xCE67_8AD7).unwrap()).unwrap();
    assert_eq!(cpu.simd[23], sha512su1_expected(d_before, cpu.simd[22], cpu.simd[7]));
}

fn sha512h_expected(w: u128, x: u128, y: u128) -> u128 {
    let w = [u64_lane(w, 0), u64_lane(w, 1)];
    let x = [u64_lane(x, 0), u64_lane(x, 1)];
    let y = [u64_lane(y, 0), u64_lane(y, 1)];
    let high = ch(y[1], x[0], x[1]).wrapping_add(bs1(y[1])).wrapping_add(w[1]);
    let tmp = high.wrapping_add(y[0]);
    let low = ch(tmp, y[1], x[0]).wrapping_add(bs1(tmp)).wrapping_add(w[0]);
    u64x2([low, high])
}

fn sha512h2_expected(w: u128, x: u128, y: u128) -> u128 {
    let w = [u64_lane(w, 0), u64_lane(w, 1)];
    let x = [u64_lane(x, 0), u64_lane(x, 1)];
    let y = [u64_lane(y, 0), u64_lane(y, 1)];
    let high = maj(x[0], y[1], y[0]).wrapping_add(bs0(y[0])).wrapping_add(w[1]);
    let low = maj(high, y[0], y[1]).wrapping_add(bs0(high)).wrapping_add(w[0]);
    u64x2([low, high])
}

fn sha512su1_expected(w: u128, x: u128, y: u128) -> u128 {
    let low = u64_lane(w, 0).wrapping_add(ss1(u64_lane(x, 0))).wrapping_add(u64_lane(y, 0));
    let high = u64_lane(w, 1).wrapping_add(ss1(u64_lane(x, 1))).wrapping_add(u64_lane(y, 1));
    u64x2([low, high])
}

fn ch(x: u64, y: u64, z: u64) -> u64 { (x & y) ^ (!x & z) }
fn maj(x: u64, y: u64, z: u64) -> u64 { (x & y) ^ (x & z) ^ (y & z) }
fn bs0(x: u64) -> u64 { x.rotate_right(28) ^ x.rotate_right(34) ^ x.rotate_right(39) }
fn bs1(x: u64) -> u64 { x.rotate_right(14) ^ x.rotate_right(18) ^ x.rotate_right(41) }
fn ss1(x: u64) -> u64 { x.rotate_right(19) ^ x.rotate_right(61) ^ (x >> 6) }
