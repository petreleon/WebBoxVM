use super::simd_helpers::*;
use super::*;

#[test]
fn simd_sha1_hash_and_schedule_forms_match_arm_rounds() {
    let (mut cpu, mut bus) = setup();

    let d = u32x4([0x0123_4567, 0x89ab_cdef, 0xfedc_ba98, 0x7654_3210]);
    let n = u32x4([0x0f1e_2d3c, 0, 0, 0]);
    let m = u32x4([0x1111_1111, 0x2222_2222, 0x3333_3333, 0x4444_4444]);
    for (raw, func) in [
        (0x5E06_00A2, choose as fn(u32, u32, u32) -> u32),
        (0x5E06_10A2, parity),
        (0x5E06_20A2, majority),
    ] {
        cpu.simd[2] = d;
        cpu.simd[5] = n;
        cpu.simd[6] = m;
        execute(&mut cpu, &mut bus, decode(raw).unwrap()).unwrap();
        assert_eq!(cpu.simd[2], sha1_hash(d, n, m, func));
    }

    cpu.simd[2] = d;
    cpu.simd[5] = n;
    cpu.simd[6] = m;
    execute(&mut cpu, &mut bus, decode(0x5E06_30A2).unwrap()).unwrap();
    assert_eq!(cpu.simd[2], sha1su0(d, n, m));

    let d = u32x4([1, 2, 3, 4]);
    let n = u32x4([5, 6, 7, 8]);
    cpu.simd[2] = d;
    cpu.simd[5] = n;
    execute(&mut cpu, &mut bus, decode(0x5E28_18A2).unwrap()).unwrap();
    assert_eq!(cpu.simd[2], sha1su1(d, n));
}

fn sha1_hash(d: u128, n: u128, m: u128, f: fn(u32, u32, u32) -> u32) -> u128 {
    let mut x = lanes(d);
    let mut y = u32_lane(n, 0);
    for word in lanes(m) {
        y = y
            .wrapping_add(x[0].rotate_left(5))
            .wrapping_add(f(x[1], x[2], x[3]))
            .wrapping_add(word);
        x[1] = x[1].rotate_left(30);
        let next_y = x[3];
        x = [y, x[0], x[1], x[2]];
        y = next_y;
    }
    u32x4(x)
}

fn sha1su0(d: u128, n: u128, m: u128) -> u128 {
    (((n as u64 as u128) << 64) | (d >> 64)) ^ d ^ m
}

fn sha1su1(d: u128, n: u128) -> u128 {
    let d = lanes(d);
    let n = lanes(n);
    let t = [d[0] ^ n[1], d[1] ^ n[2], d[2] ^ n[3], d[3]];
    u32x4([
        t[0].rotate_left(1),
        t[1].rotate_left(1),
        t[2].rotate_left(1),
        t[3].rotate_left(1) ^ t[0].rotate_left(2),
    ])
}

fn lanes(value: u128) -> [u32; 4] {
    [
        u32_lane(value, 0),
        u32_lane(value, 1),
        u32_lane(value, 2),
        u32_lane(value, 3),
    ]
}

fn choose(x: u32, y: u32, z: u32) -> u32 {
    ((y ^ z) & x) ^ z
}
fn majority(x: u32, y: u32, z: u32) -> u32 {
    (x & y) | ((x | y) & z)
}
fn parity(x: u32, y: u32, z: u32) -> u32 {
    x ^ y ^ z
}
