use super::simd_helpers::*;
use super::*;

#[test]
fn simd_sm3_schedule_forms_match_arm_pseudocode() {
    let (mut cpu, mut bus) = setup();
    let d = u32x4([0x0011_2233, 0x4455_6677, 0x8899_aabb, 0xccdd_eeff]);
    let n = u32x4([0x1020_3040, 0x5060_7080, 0x90a0_b0c0, 0xd0e0_f000]);
    let m = u32x4([0x89ab_cdef, 0x0123_4567, 0x7654_3210, 0xfedc_ba98]);

    cpu.simd[4] = d;
    cpu.simd[23] = n;
    cpu.simd[22] = m;
    execute(&mut cpu, &mut bus, decode(0xCE76_C6E4).unwrap()).unwrap();
    assert_eq!(cpu.simd[4], sm3partw2_expected(d, n, m));
}

#[test]
fn simd_sm3_ss1_and_round_forms_match_arm_pseudocode() {
    let (mut cpu, mut bus) = setup();
    let state = u32x4([0x0123_4567, 0x89ab_cdef, 0xfedc_ba98, 0x7654_3210]);
    let n = u32x4([0x1111_2222, 0x3333_4444, 0x5555_6666, 0x7777_8888]);
    let m = u32x4([0x1020_3040, 0x5060_7080, 0x90a0_b0c0, 0xd0e0_f000]);
    let a = u32x4([0x0bad_cafe, 0xfeed_face, 0x1357_2468, 0x89ab_cdef]);

    cpu.simd[23] = state;
    cpu.simd[5] = n;
    cpu.simd[20] = m;
    cpu.simd[6] = a;
    execute(&mut cpu, &mut bus, decode(0xCE54_18B7).unwrap()).unwrap();
    assert_eq!(cpu.simd[23], sm3ss1_expected(n, m, a));

    cpu.simd[5] = state;
    cpu.simd[23] = n;
    cpu.simd[22] = m;
    execute(&mut cpu, &mut bus, decode(0xCE56_A2E5).unwrap()).unwrap();
    assert_eq!(cpu.simd[5], sm3tt1_expected(state, n, m, 2, false));

    cpu.simd[5] = state;
    execute(&mut cpu, &mut bus, decode(0xCE56_B6E5).unwrap()).unwrap();
    assert_eq!(cpu.simd[5], sm3tt1_expected(state, n, m, 3, true));

    cpu.simd[6] = state;
    cpu.simd[0] = m;
    execute(&mut cpu, &mut bus, decode(0xCE40_9AE6).unwrap()).unwrap();
    assert_eq!(cpu.simd[6], sm3tt2_expected(state, n, m, 1, false));

    cpu.simd[6] = state;
    execute(&mut cpu, &mut bus, decode(0xCE40_BEE6).unwrap()).unwrap();
    assert_eq!(cpu.simd[6], sm3tt2_expected(state, n, m, 3, true));
}

fn sm3partw2_expected(d: u128, n: u128, m: u128) -> u128 {
    let tmp: [u32; 4] =
        core::array::from_fn(|lane| u32_lane(n, lane) ^ u32_lane(m, lane).rotate_left(7));
    let mut out: [u32; 4] = core::array::from_fn(|lane| u32_lane(d, lane) ^ tmp[lane]);
    let mut top = tmp[0].rotate_left(15);
    top ^= top.rotate_left(15) ^ top.rotate_left(23);
    out[3] ^= top;
    u32x4(out)
}

fn sm3ss1_expected(n: u128, m: u128, a: u128) -> u128 {
    let sum = u32_lane(n, 3)
        .rotate_left(12)
        .wrapping_add(u32_lane(m, 3))
        .wrapping_add(u32_lane(a, 3));
    (sum.rotate_left(7) as u128) << 96
}

fn sm3tt1_expected(d: u128, n: u128, m: u128, lane: usize, majority: bool) -> u128 {
    let x = [
        u32_lane(d, 0),
        u32_lane(d, 1),
        u32_lane(d, 2),
        u32_lane(d, 3),
    ];
    let f = if majority {
        (x[3] & x[1]) | (x[3] & x[2]) | (x[1] & x[2])
    } else {
        x[1] ^ x[2] ^ x[3]
    };
    let ss2 = u32_lane(n, 3) ^ x[3].rotate_left(12);
    let tt = f
        .wrapping_add(x[0])
        .wrapping_add(ss2)
        .wrapping_add(u32_lane(m, lane));
    u32x4([x[1], x[2].rotate_left(9), x[3], tt])
}

fn sm3tt2_expected(d: u128, n: u128, m: u128, lane: usize, choose: bool) -> u128 {
    let x = [
        u32_lane(d, 0),
        u32_lane(d, 1),
        u32_lane(d, 2),
        u32_lane(d, 3),
    ];
    let f = if choose {
        (x[3] & x[2]) | (!x[3] & x[1])
    } else {
        x[1] ^ x[2] ^ x[3]
    };
    let tt = f
        .wrapping_add(x[0])
        .wrapping_add(u32_lane(n, 3))
        .wrapping_add(u32_lane(m, lane));
    u32x4([
        x[1],
        x[2].rotate_left(19),
        x[3],
        tt ^ tt.rotate_left(9) ^ tt.rotate_left(17),
    ])
}
