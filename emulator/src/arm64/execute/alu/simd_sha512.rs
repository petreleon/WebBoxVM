use super::*;

pub(in crate::arm64::execute) fn exec_simd_sha512(cpu: &mut Armv8Cpu, instr: Instr) {
    let d = u64x2(cpu.simd[instr.rd as usize]);
    let n = u64x2(cpu.simd[instr.rn as usize]);
    let m = u64x2(cpu.simd[instr.rm as usize]);
    let out = match instr.op {
        Opcode::SimdSha512H => sha512h(d, n, m),
        Opcode::SimdSha512H2 => sha512h2(d, n, m),
        Opcode::SimdSha512Su1 => sha512su1(d, n, m),
        _ => unreachable!(),
    };
    cpu.simd[instr.rd as usize] = pack(out);
}

fn sha512h(w: [u64; 2], x: [u64; 2], y: [u64; 2]) -> [u64; 2] {
    let high = sha_ch(y[1], x[0], x[1])
        .wrapping_add(big_sigma1(y[1]))
        .wrapping_add(w[1]);
    let tmp = high.wrapping_add(y[0]);
    let low = sha_ch(tmp, y[1], x[0])
        .wrapping_add(big_sigma1(tmp))
        .wrapping_add(w[0]);
    [low, high]
}

fn sha512h2(w: [u64; 2], x: [u64; 2], y: [u64; 2]) -> [u64; 2] {
    let high = sha_maj(x[0], y[1], y[0])
        .wrapping_add(big_sigma0(y[0]))
        .wrapping_add(w[1]);
    let low = sha_maj(high, y[0], y[1])
        .wrapping_add(big_sigma0(high))
        .wrapping_add(w[0]);
    [low, high]
}

fn sha512su1(w: [u64; 2], x: [u64; 2], y: [u64; 2]) -> [u64; 2] {
    [
        w[0].wrapping_add(small_sigma1(x[0])).wrapping_add(y[0]),
        w[1].wrapping_add(small_sigma1(x[1])).wrapping_add(y[1]),
    ]
}

fn u64x2(value: u128) -> [u64; 2] {
    [value as u64, (value >> 64) as u64]
}

fn pack(words: [u64; 2]) -> u128 {
    words[0] as u128 | ((words[1] as u128) << 64)
}

fn sha_ch(x: u64, y: u64, z: u64) -> u64 {
    (x & y) ^ (!x & z)
}

fn sha_maj(x: u64, y: u64, z: u64) -> u64 {
    (x & y) ^ (x & z) ^ (y & z)
}

fn big_sigma0(x: u64) -> u64 {
    x.rotate_right(28) ^ x.rotate_right(34) ^ x.rotate_right(39)
}

fn big_sigma1(x: u64) -> u64 {
    x.rotate_right(14) ^ x.rotate_right(18) ^ x.rotate_right(41)
}

fn small_sigma1(x: u64) -> u64 {
    x.rotate_right(19) ^ x.rotate_right(61) ^ (x >> 6)
}
