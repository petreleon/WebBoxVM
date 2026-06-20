use super::*;

pub(in crate::arch::arm64::execute) fn is_simd_sha1_sha256_opcode(op: Opcode) -> bool {
    matches!(
        op,
        Opcode::SimdSha1h
            | Opcode::SimdSha256Su0
            | Opcode::SimdSha1C
            | Opcode::SimdSha1M
            | Opcode::SimdSha1P
            | Opcode::SimdSha1Su0
            | Opcode::SimdSha1Su1
            | Opcode::SimdSha256H
            | Opcode::SimdSha256H2
            | Opcode::SimdSha256Su1
    )
}

pub(in crate::arch::arm64::execute) fn exec_simd_sha1_sha256(cpu: &mut Armv8Cpu, instr: Instr) {
    let rd = instr.rd as usize;
    let rn = instr.rn as usize;
    let rm = instr.rm as usize;
    let d = cpu.simd[rd];
    let n = cpu.simd[rn];
    let m = cpu.simd[rm];
    cpu.simd[rd] = match instr.op {
        Opcode::SimdSha1h => (lane(n, 0).rotate_left(30)) as u128,
        Opcode::SimdSha1C => sha1_hash(d, n, m, sha_choose),
        Opcode::SimdSha1M => sha1_hash(d, n, m, sha_majority),
        Opcode::SimdSha1P => sha1_hash(d, n, m, sha_parity),
        Opcode::SimdSha1Su0 => sha1su0(d, n, m),
        Opcode::SimdSha1Su1 => sha1su1(d, n),
        Opcode::SimdSha256Su0 => sha256su0(d, n),
        Opcode::SimdSha256H => sha256_hash(d, n, m, true),
        Opcode::SimdSha256H2 => sha256_hash(n, d, m, false),
        Opcode::SimdSha256Su1 => sha256su1(d, n, m),
        _ => unreachable!(),
    };
}

fn sha1_hash(d: u128, n: u128, m: u128, f: fn(u32, u32, u32) -> u32) -> u128 {
    let mut x = lanes(d);
    let mut y = lane(n, 0);
    let w = lanes(m);
    for word in w {
        y = y
            .wrapping_add(x[0].rotate_left(5))
            .wrapping_add(f(x[1], x[2], x[3]))
            .wrapping_add(word);
        x[1] = x[1].rotate_left(30);
        let next_y = x[3];
        x = [y, x[0], x[1], x[2]];
        y = next_y;
    }
    pack_u32_lanes(x)
}

fn sha1su0(d: u128, n: u128, m: u128) -> u128 {
    (((n as u64 as u128) << 64) | (d >> 64)) ^ d ^ m
}

fn sha1su1(d: u128, n: u128) -> u128 {
    let d = lanes(d);
    let n = lanes(n);
    let t = [d[0] ^ n[1], d[1] ^ n[2], d[2] ^ n[3], d[3]];
    pack_u32_lanes([
        t[0].rotate_left(1),
        t[1].rotate_left(1),
        t[2].rotate_left(1),
        t[3].rotate_left(1) ^ t[0].rotate_left(2),
    ])
}

fn sha256su0(d: u128, n: u128) -> u128 {
    let schedule = [lane(d, 1), lane(d, 2), lane(d, 3), lane(n, 0)];
    let d = lanes(d);
    let mut out = [0u32; 4];
    for index in 0..4 {
        let word = schedule[index];
        let sigma = word.rotate_right(7) ^ word.rotate_right(18) ^ (word >> 3);
        out[index] = d[index].wrapping_add(sigma);
    }
    pack_u32_lanes(out)
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
    pack_u32_lanes(out)
}

fn sha256_hash(x_in: u128, y_in: u128, w: u128, part1: bool) -> u128 {
    let mut x = lanes(x_in);
    let mut y = lanes(y_in);
    for word in lanes(w) {
        let t = y[3]
            .wrapping_add(big1(y[0]))
            .wrapping_add(sha_choose(y[0], y[1], y[2]))
            .wrapping_add(word);
        let next_x3 = x[3].wrapping_add(t);
        let next_y3 = t
            .wrapping_add(big0(x[0]))
            .wrapping_add(sha_majority(x[0], x[1], x[2]));
        x = [next_y3, x[0], x[1], x[2]];
        y = [next_x3, y[0], y[1], y[2]];
    }
    pack_u32_lanes(if part1 { x } else { y })
}

fn lanes(value: u128) -> [u32; 4] {
    [
        lane(value, 0),
        lane(value, 1),
        lane(value, 2),
        lane(value, 3),
    ]
}

fn lane(value: u128, index: usize) -> u32 {
    simd_element(value, index, 4) as u32
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
fn sha_choose(x: u32, y: u32, z: u32) -> u32 {
    ((y ^ z) & x) ^ z
}
fn sha_majority(x: u32, y: u32, z: u32) -> u32 {
    (x & y) | ((x | y) & z)
}
fn sha_parity(x: u32, y: u32, z: u32) -> u32 {
    x ^ y ^ z
}
