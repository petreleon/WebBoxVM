use super::*;

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
        Opcode::SimdSha1h => {
            let value = simd_element(cpu.simd[rn], 0, 4) as u32;
            cpu.simd[rd] = value.rotate_left(30) as u128;
        }
        Opcode::SimdSha256Su0 => {
            let operand1 = cpu.simd[rd];
            let operand2 = cpu.simd[rn];
            let schedule = [
                simd_element(operand1, 1, 4) as u32,
                simd_element(operand1, 2, 4) as u32,
                simd_element(operand1, 3, 4) as u32,
                simd_element(operand2, 0, 4) as u32,
            ];
            let mut out = 0u128;
            for (lane, value) in schedule.into_iter().enumerate() {
                let sigma0 = value.rotate_right(7) ^ value.rotate_right(18) ^ (value >> 3);
                let word = (simd_element(operand1, lane, 4) as u32).wrapping_add(sigma0);
                out |= (word as u128) << (lane * 32);
            }
            cpu.simd[rd] = out;
        }
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
        Opcode::SimdSm3Partw1 => {
            let vd = cpu.simd[rd];
            let vn = cpu.simd[rn];
            let vm = cpu.simd[rm];
            let mut words = [0u32; 4];
            for (lane, word) in words.iter_mut().enumerate().take(3) {
                let base = (simd_element(vd, lane, 4) ^ simd_element(vn, lane, 4)) as u32;
                let rotated = (simd_element(vm, lane + 1, 4) as u32).rotate_left(15);
                *word = base ^ rotated;
            }
            for lane in 0..4 {
                if lane == 3 {
                    let base = (simd_element(vd, 3, 4) ^ simd_element(vn, 3, 4)) as u32;
                    words[3] = base ^ words[0].rotate_left(15);
                }
                words[lane] ^= words[lane].rotate_left(15) ^ words[lane].rotate_left(23);
            }
            cpu.simd[rd] = pack_u32_lanes(words);
        }
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
