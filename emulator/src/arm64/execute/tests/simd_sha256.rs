use super::simd_helpers::*;
use super::*;

#[test]
fn simd_sha256_hash_and_schedule_forms_match_arm_rounds() {
    let (mut cpu, mut bus) = setup();

    let d = u32x4([0x0123_4567, 0x89ab_cdef, 0xfedc_ba98, 0x7654_3210]);
    let n = u32x4([0x0f1e_2d3c, 0x4b5a_6978, 0x8796_a5b4, 0xc3d2_e1f0]);
    let m = u32x4([0x1111_1111, 0x2222_2222, 0x3333_3333, 0x4444_4444]);

    cpu.simd[2] = d;
    cpu.simd[5] = n;
    cpu.simd[6] = m;
    execute(&mut cpu, &mut bus, decode(0x5E06_40A2).unwrap()).unwrap();
    assert_eq!(cpu.simd[2], sha256_hash(d, n, m, true));

    cpu.simd[2] = d;
    cpu.simd[5] = n;
    cpu.simd[6] = m;
    execute(&mut cpu, &mut bus, decode(0x5E06_50A2).unwrap()).unwrap();
    assert_eq!(cpu.simd[2], sha256_hash(n, d, m, false));

    cpu.simd[2] = d;
    cpu.simd[5] = n;
    execute(&mut cpu, &mut bus, decode(0x5E28_28A2).unwrap()).unwrap();
    assert_eq!(cpu.simd[2], sha256su0(d, n));

    cpu.simd[2] = d;
    cpu.simd[5] = n;
    cpu.simd[6] = m;
    execute(&mut cpu, &mut bus, decode(0x5E06_60A2).unwrap()).unwrap();
    assert_eq!(cpu.simd[2], sha256su1(d, n, m));
}

fn sha256_hash(x_in: u128, y_in: u128, w: u128, part1: bool) -> u128 {
    let mut x = lanes(x_in);
    let mut y = lanes(y_in);
    for word in lanes(w) {
        let t = y[3]
            .wrapping_add(big1(y[0]))
            .wrapping_add(choose(y[0], y[1], y[2]))
            .wrapping_add(word);
        let next_x3 = x[3].wrapping_add(t);
        let next_y3 = t
            .wrapping_add(big0(x[0]))
            .wrapping_add(majority(x[0], x[1], x[2]));
        x = [next_y3, x[0], x[1], x[2]];
        y = [next_x3, y[0], y[1], y[2]];
    }
    u32x4(if part1 { x } else { y })
}

fn sha256su0(d: u128, n: u128) -> u128 {
    let d = lanes(d);
    let schedule = [d[1], d[2], d[3], u32_lane(n, 0)];
    let mut out = [0u32; 4];
    for index in 0..4 {
        out[index] = d[index].wrapping_add(sigma0(schedule[index]));
    }
    u32x4(out)
}

fn sha256su1(d: u128, n: u128, m: u128) -> u128 {
    let d = lanes(d);
    let n = lanes(n);
    let m = lanes(m);
    let mut out = [0u32; 4];
    out[0] = d[0].wrapping_add(n[1]).wrapping_add(sigma1(m[2]));
    out[1] = d[1].wrapping_add(n[2]).wrapping_add(sigma1(m[3]));
    out[2] = d[2].wrapping_add(n[3]).wrapping_add(sigma1(out[0]));
    out[3] = d[3].wrapping_add(m[0]).wrapping_add(sigma1(out[1]));
    u32x4(out)
}

fn lanes(value: u128) -> [u32; 4] {
    [
        u32_lane(value, 0),
        u32_lane(value, 1),
        u32_lane(value, 2),
        u32_lane(value, 3),
    ]
}

fn sigma0(x: u32) -> u32 {
    x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3)
}
fn sigma1(x: u32) -> u32 {
    x.rotate_right(17) ^ x.rotate_right(19) ^ (x >> 10)
}
fn big0(x: u32) -> u32 {
    x.rotate_right(2) ^ x.rotate_right(13) ^ x.rotate_right(22)
}
fn big1(x: u32) -> u32 {
    x.rotate_right(6) ^ x.rotate_right(11) ^ x.rotate_right(25)
}
fn choose(x: u32, y: u32, z: u32) -> u32 {
    ((y ^ z) & x) ^ z
}
fn majority(x: u32, y: u32, z: u32) -> u32 {
    (x & y) | ((x | y) & z)
}
