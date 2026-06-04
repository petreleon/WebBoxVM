use super::*;

pub(in crate::arm64::execute) fn is_simd_crypto_opcode(op: Opcode) -> bool {
    is_simd_sm3_opcode(op)
        || matches!(
        op,
        Opcode::SimdAese
            | Opcode::SimdAesd
            | Opcode::SimdAesmc
            | Opcode::SimdAesimc
            | Opcode::SimdPmull
            | Opcode::SimdSha1h
            | Opcode::SimdSha256Su0
            | Opcode::SimdSha512Su0
            | Opcode::SimdSha512H
            | Opcode::SimdSha512H2
            | Opcode::SimdSha512Su1
            | Opcode::SimdSha1C
            | Opcode::SimdSha1M
            | Opcode::SimdSha1P
            | Opcode::SimdSha1Su0
            | Opcode::SimdSha1Su1
            | Opcode::SimdSha256H
            | Opcode::SimdSha256H2
            | Opcode::SimdSha256Su1
            | Opcode::SimdSm4e
            | Opcode::SimdSm4EKey
            | Opcode::SimdEor3
            | Opcode::SimdBcax
            | Opcode::SimdRax1
            | Opcode::SimdXar
    )
}

pub(in crate::arm64::execute) fn exec_simd_crypto(cpu: &mut Armv8Cpu, instr: Instr) {
    let rd = instr.rd as usize;
    let rn = instr.rn as usize;
    let rm = instr.rm as usize;

    match instr.op {
        Opcode::SimdAese => {
            cpu.simd[rd] = aes_sub_shift_round(cpu.simd[rd] ^ cpu.simd[rn], false);
        }
        Opcode::SimdAesd => {
            cpu.simd[rd] = aes_sub_shift_round(cpu.simd[rd] ^ cpu.simd[rn], true);
        }
        Opcode::SimdAesmc => {
            cpu.simd[rd] = aes_mix_columns(cpu.simd[rn], false);
        }
        Opcode::SimdAesimc => {
            cpu.simd[rd] = aes_mix_columns(cpu.simd[rn], true);
        }
        Opcode::SimdPmull => {
            let element_size = instr.imm.max(1) as usize;
            let bits = element_size * 8;
            let part_shift = if instr.sf { 64 } else { 0 };
            let lhs = (cpu.simd[rn] >> part_shift) & u64::MAX as u128;
            let rhs = (cpu.simd[rm] >> part_shift) & u64::MAX as u128;
            let lanes = 8 / element_size;
            let mut out = 0u128;
            for lane in 0..lanes {
                let shift = lane * bits;
                let a = (lhs >> shift) & simd_element_mask(element_size);
                let b = (rhs >> shift) & simd_element_mask(element_size);
                out |= simd_polynomial_mult(a, b, bits) << (lane * bits * 2);
            }
            cpu.simd[rd] = out;
        }
        op if is_simd_sha1_sha256_opcode(op) => exec_simd_sha1_sha256(cpu, instr),
        Opcode::SimdSha512Su0 => {
            let w = cpu.simd[rd];
            let x = cpu.simd[rn];
            let w0 = simd_element(w, 0, 8) as u64;
            let w1 = simd_element(w, 1, 8) as u64;
            let x0 = simd_element(x, 0, 8) as u64;
            let sig_w1 = w1.rotate_right(1) ^ w1.rotate_right(8) ^ (w1 >> 7);
            let sig_x0 = x0.rotate_right(1) ^ x0.rotate_right(8) ^ (x0 >> 7);
            cpu.simd[rd] =
                w0.wrapping_add(sig_w1) as u128 | ((w1.wrapping_add(sig_x0) as u128) << 64);
        }
        Opcode::SimdSha512H | Opcode::SimdSha512H2 | Opcode::SimdSha512Su1 => {
            exec_simd_sha512(cpu, instr);
        }
        Opcode::SimdSm4e => {
            let keys = cpu.simd[rn];
            let mut words = [
                simd_element(cpu.simd[rd], 0, 4) as u32,
                simd_element(cpu.simd[rd], 1, 4) as u32,
                simd_element(cpu.simd[rd], 2, 4) as u32,
                simd_element(cpu.simd[rd], 3, 4) as u32,
            ];
            for index in 0..4 {
                let round_key = simd_element(keys, index, 4) as u32;
                let mixed = words[3] ^ words[2] ^ words[1] ^ round_key;
                let round = sm4_linear_transform(sm4_sub_word(mixed)) ^ words[0];
                words = [words[1], words[2], words[3], round];
            }
            cpu.simd[rd] = pack_u32_lanes(words);
        }
        Opcode::SimdSm4EKey => {
            let consts = cpu.simd[rm];
            let mut words = [
                simd_element(cpu.simd[rn], 0, 4) as u32,
                simd_element(cpu.simd[rn], 1, 4) as u32,
                simd_element(cpu.simd[rn], 2, 4) as u32,
                simd_element(cpu.simd[rn], 3, 4) as u32,
            ];
            for index in 0..4 {
                let constant = simd_element(consts, index, 4) as u32;
                let mixed = words[3] ^ words[2] ^ words[1] ^ constant;
                let round = sm4_key_transform(sm4_sub_word(mixed)) ^ words[0];
                words = [words[1], words[2], words[3], round];
            }
            cpu.simd[rd] = pack_u32_lanes(words);
        }
        op if is_simd_sm3_opcode(op) => exec_simd_sm3(cpu, instr),
        Opcode::SimdEor3 => {
            let ra = instr.cond as usize;
            cpu.simd[rd] = cpu.simd[rn] ^ cpu.simd[rm] ^ cpu.simd[ra];
        }
        Opcode::SimdBcax => {
            let ra = instr.cond as usize;
            cpu.simd[rd] = cpu.simd[rn] ^ (cpu.simd[rm] & !cpu.simd[ra]);
        }
        Opcode::SimdRax1 => {
            cpu.simd[rd] = cpu.simd[rn] ^ simd_rotate_left_64_lanes(cpu.simd[rm], 1);
        }
        Opcode::SimdXar => {
            cpu.simd[rd] =
                simd_rotate_right_64_lanes(cpu.simd[rn] ^ cpu.simd[rm], instr.imm as u32);
        }
        _ => unreachable!(),
    }
}
