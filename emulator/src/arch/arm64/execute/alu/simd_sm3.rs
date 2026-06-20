use super::*;

pub(in crate::arch::arm64::execute) fn is_simd_sm3_opcode(op: Opcode) -> bool {
    matches!(
        op,
        Opcode::SimdSm3Partw1
            | Opcode::SimdSm3Partw2
            | Opcode::SimdSm3Ss1
            | Opcode::SimdSm3Tt1A
            | Opcode::SimdSm3Tt1B
            | Opcode::SimdSm3Tt2A
            | Opcode::SimdSm3Tt2B
    )
}

pub(in crate::arch::arm64::execute) fn exec_simd_sm3(cpu: &mut Armv8Cpu, instr: Instr) {
    let rd = instr.rd as usize;
    let rn = instr.rn as usize;
    let rm = instr.rm as usize;
    match instr.op {
        Opcode::SimdSm3Partw1 => {
            cpu.simd[rd] = sm3_partw1(cpu.simd[rd], cpu.simd[rn], cpu.simd[rm])
        }
        Opcode::SimdSm3Partw2 => {
            cpu.simd[rd] = sm3_partw2(cpu.simd[rd], cpu.simd[rn], cpu.simd[rm])
        }
        Opcode::SimdSm3Ss1 => {
            let va = instr.cond as usize;
            let sum = u32_lane(cpu.simd[rn], 3)
                .rotate_left(12)
                .wrapping_add(u32_lane(cpu.simd[rm], 3))
                .wrapping_add(u32_lane(cpu.simd[va], 3));
            cpu.simd[rd] = (sum.rotate_left(7) as u128) << 96;
        }
        Opcode::SimdSm3Tt1A | Opcode::SimdSm3Tt1B => {
            cpu.simd[rd] = sm3_tt1(cpu.simd[rd], cpu.simd[rn], cpu.simd[rm], instr);
        }
        Opcode::SimdSm3Tt2A | Opcode::SimdSm3Tt2B => {
            cpu.simd[rd] = sm3_tt2(cpu.simd[rd], cpu.simd[rn], cpu.simd[rm], instr);
        }
        _ => unreachable!(),
    }
}

fn sm3_partw1(d: u128, n: u128, m: u128) -> u128 {
    let mut words = [0u32; 4];
    for lane in 0..3 {
        words[lane] =
            (u32_lane(d, lane) ^ u32_lane(n, lane)) ^ u32_lane(m, lane + 1).rotate_left(15);
    }
    for lane in 0..4 {
        if lane == 3 {
            words[3] = (u32_lane(d, 3) ^ u32_lane(n, 3)) ^ words[0].rotate_left(15);
        }
        words[lane] ^= words[lane].rotate_left(15) ^ words[lane].rotate_left(23);
    }
    pack_u32_lanes(words)
}

fn sm3_partw2(d: u128, n: u128, m: u128) -> u128 {
    let tmp: [u32; 4] =
        core::array::from_fn(|lane| u32_lane(n, lane) ^ u32_lane(m, lane).rotate_left(7));
    let mut result: [u32; 4] = core::array::from_fn(|lane| u32_lane(d, lane) ^ tmp[lane]);
    let mut top = tmp[0].rotate_left(15);
    top ^= top.rotate_left(15) ^ top.rotate_left(23);
    result[3] ^= top;
    pack_u32_lanes(result)
}

fn sm3_tt1(d: u128, n: u128, m: u128, instr: Instr) -> u128 {
    let x = [
        u32_lane(d, 0),
        u32_lane(d, 1),
        u32_lane(d, 2),
        u32_lane(d, 3),
    ];
    let f = if instr.op == Opcode::SimdSm3Tt1A {
        x[1] ^ x[2] ^ x[3]
    } else {
        (x[3] & x[1]) | (x[3] & x[2]) | (x[1] & x[2])
    };
    let ss2 = u32_lane(n, 3) ^ x[3].rotate_left(12);
    let tt1 = f
        .wrapping_add(x[0])
        .wrapping_add(ss2)
        .wrapping_add(u32_lane(m, instr.imm as usize));
    pack_u32_lanes([x[1], x[2].rotate_left(9), x[3], tt1])
}

fn sm3_tt2(d: u128, n: u128, m: u128, instr: Instr) -> u128 {
    let x = [
        u32_lane(d, 0),
        u32_lane(d, 1),
        u32_lane(d, 2),
        u32_lane(d, 3),
    ];
    let f = if instr.op == Opcode::SimdSm3Tt2A {
        x[1] ^ x[2] ^ x[3]
    } else {
        (x[3] & x[2]) | (!x[3] & x[1])
    };
    let tt2 = f
        .wrapping_add(x[0])
        .wrapping_add(u32_lane(n, 3))
        .wrapping_add(u32_lane(m, instr.imm as usize));
    pack_u32_lanes([
        x[1],
        x[2].rotate_left(19),
        x[3],
        tt2 ^ tt2.rotate_left(9) ^ tt2.rotate_left(17),
    ])
}

fn u32_lane(value: u128, lane: usize) -> u32 {
    simd_element(value, lane, 4) as u32
}
