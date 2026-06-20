use super::simd_helpers::*;
use super::*;

#[test]
fn simd_ld1_and_st1_multi_load_consecutive_vectors() {
    let (mut cpu, mut bus) = setup();
    let base = RAM_BASE + 0x1000;
    cpu.regs.set_x(1, base);
    for byte in 0..32u64 {
        bus.write(base + byte, 1, byte);
    }

    execute(&mut cpu, &mut bus, decode(0x4C40_A03E).unwrap()).unwrap(); // ld1 {v30.16b, v31.16b}, [x1]

    assert_eq!(cpu.simd[30], vector_bytes(0));
    assert_eq!(cpu.simd[31], vector_bytes(16));

    let ld1_post_multi_base = RAM_BASE + 0x2c00;
    cpu.regs.set_x(1, ld1_post_multi_base);
    for byte in 0..32u64 {
        bus.write(ld1_post_multi_base + byte, 1, 0x20 + byte);
    }
    execute(&mut cpu, &mut bus, decode(0x4CDF_A03C).unwrap()).unwrap(); // ld1 {v28.16b, v29.16b}, [x1], #32
    assert_eq!(cpu.simd[28], vector_bytes(0x20));
    assert_eq!(cpu.simd[29], vector_bytes(0x30));
    assert_eq!(cpu.regs.x(1), ld1_post_multi_base + 32);

    let ld1_post_reg_base = RAM_BASE + 0x3000;
    cpu.regs.set_x(0, ld1_post_reg_base);
    cpu.regs.set_x(8, 40);
    for byte in 0..16u64 {
        bus.write(ld1_post_reg_base + byte, 1, 0xc0 + byte);
    }
    execute(&mut cpu, &mut bus, decode(0x4CC8_7000).unwrap()).unwrap(); // ld1 {v0.16b}, [x0], x8
    assert_eq!(cpu.simd[0], vector_bytes(0xc0));
    assert_eq!(cpu.regs.x(0), ld1_post_reg_base + 40);

    let ld2_base = RAM_BASE + 0x3400;
    cpu.regs.set_x(1, ld2_base);
    for byte in 0..32u64 {
        bus.write(ld2_base + byte, 1, byte);
    }
    execute(&mut cpu, &mut bus, decode(0x4C40_803C).unwrap()).unwrap(); // ld2 {v28.16b, v29.16b}, [x1]
    assert_eq!(cpu.simd[28], ld_structure_vector_bytes(0, 2, 0, 1, 16));
    assert_eq!(cpu.simd[29], ld_structure_vector_bytes(0, 2, 1, 1, 16));

    let ld3_post_imm_base = RAM_BASE + 0x3800;
    cpu.regs.set_x(2, ld3_post_imm_base);
    for byte in 0..48u64 {
        bus.write(ld3_post_imm_base + byte, 1, 0x50 + byte);
    }
    execute(&mut cpu, &mut bus, decode(0x4CDF_4C5A).unwrap()).unwrap(); // ld3 {v26.2d-v28.2d}, [x2], #48
    assert_eq!(cpu.simd[26], ld_structure_vector_bytes(0x50, 3, 0, 8, 2));
    assert_eq!(cpu.simd[28], ld_structure_vector_bytes(0x50, 3, 2, 8, 2));
    assert_eq!(cpu.regs.x(2), ld3_post_imm_base + 48);

    let ld4_base = RAM_BASE + 0x1400;
    cpu.regs.set_x(1, ld4_base);
    for byte in 0..64u64 {
        bus.write(ld4_base + byte, 1, byte);
    }
    execute(&mut cpu, &mut bus, decode(0x4C40_003C).unwrap()).unwrap(); // ld4 {v28.16b-v31.16b}, [x1]
    assert_eq!(cpu.simd[28], 0x3c38_3430_2c28_2420_1c18_1410_0c08_0400);
    assert_eq!(cpu.simd[29], 0x3d39_3531_2d29_2521_1d19_1511_0d09_0501);
    assert_eq!(cpu.simd[30], 0x3e3a_3632_2e2a_2622_1e1a_1612_0e0a_0602);
    assert_eq!(cpu.simd[31], 0x3f3b_3733_2f2b_2723_1f1b_1713_0f0b_0703);

    let ld4_post_imm_base = RAM_BASE + 0x1600;
    cpu.regs.set_x(1, ld4_post_imm_base);
    for byte in 0..64u64 {
        bus.write(ld4_post_imm_base + byte, 1, 0x40 + byte);
    }
    execute(&mut cpu, &mut bus, decode(0x4CDF_003C).unwrap()).unwrap(); // ld4 {v28.16b-v31.16b}, [x1], #64
    assert_eq!(cpu.simd[28], ld_structure_vector_bytes(0x40, 4, 0, 1, 16));
    assert_eq!(cpu.simd[31], ld_structure_vector_bytes(0x40, 4, 3, 1, 16));
    assert_eq!(cpu.regs.x(1), ld4_post_imm_base + 64);

    let ld4_post_reg_base = RAM_BASE + 0x1c00;
    cpu.regs.set_x(1, ld4_post_reg_base);
    cpu.regs.set_x(2, 96);
    for byte in 0..64u64 {
        bus.write(ld4_post_reg_base + byte, 1, 0x80 + byte);
    }
    execute(&mut cpu, &mut bus, decode(0x4CC2_003C).unwrap()).unwrap(); // ld4 {v28.16b-v31.16b}, [x1], x2
    assert_eq!(cpu.simd[28], ld_structure_vector_bytes(0x80, 4, 0, 1, 16));
    assert_eq!(cpu.simd[31], ld_structure_vector_bytes(0x80, 4, 3, 1, 16));
    assert_eq!(cpu.regs.x(1), ld4_post_reg_base + 96);

    let post_index_base = RAM_BASE + 0x1800;
    cpu.regs.set_x(16, post_index_base);
    for byte in 0..16u64 {
        bus.write(post_index_base + byte, 1, 0xa0 + byte);
    }
    execute(&mut cpu, &mut bus, decode(0x4CDF_7A04).unwrap()).unwrap(); // ld1 {v4.16b}, [x16], #16
    assert_eq!(cpu.simd[4], 0xafaeadac_abaaa9a8_a7a6a5a4_a3a2a1a0);
    assert_eq!(cpu.regs.x(16), post_index_base + 16);

    let out = RAM_BASE + 0x2000;
    cpu.regs.set_x(22, out);
    cpu.simd[30] = 0x0f0e_0d0c_0b0a_0908_0706_0504_0302_0100;
    cpu.simd[31] = 0x1f1e_1d1c_1b1a_1918_1716_1514_1312_1110;

    execute(&mut cpu, &mut bus, decode(0x4C00_A2DE).unwrap()).unwrap(); // st1 {v30.16b, v31.16b}, [x22]

    for byte in 0..32u64 {
        assert_eq!(bus.read(out + byte, 1), Some(byte));
    }

    let post_index_out = RAM_BASE + 0x2400;
    cpu.regs.set_x(17, post_index_out);
    cpu.simd[4] = 0x8f8e_8d8c_8b8a_8988_8786_8584_8382_8180;
    cpu.simd[5] = 0x9f9e_9d9c_9b9a_9998_9796_9594_9392_9190;

    execute(&mut cpu, &mut bus, decode(0x4C9F_AA24).unwrap()).unwrap(); // st1 {v4.16b, v5.16b}, [x17], #32

    for byte in 0..32u64 {
        assert_eq!(bus.read(post_index_out + byte, 1), Some(0x80 + byte));
    }
    assert_eq!(cpu.regs.x(17), post_index_out + 32);

    let st1_one_out = RAM_BASE + 0x2800;
    cpu.regs.set_x(17, st1_one_out);
    cpu.simd[4] = vector_bytes(0x40);

    execute(&mut cpu, &mut bus, decode(0x4C9F_7A24).unwrap()).unwrap(); // st1 {v4.4s}, [x17], #16

    for byte in 0..16u64 {
        assert_eq!(bus.read(st1_one_out + byte, 1), Some(0x40 + byte));
    }
    assert_eq!(cpu.regs.x(17), st1_one_out + 16);

    let st1_one_no_offset_out = RAM_BASE + 0x3c00;
    cpu.regs.set_x(1, st1_one_no_offset_out);
    cpu.simd[0] = vector_bytes(0x60);
    execute(&mut cpu, &mut bus, decode(0x4C00_7020).unwrap()).unwrap(); // st1 {v0.16b}, [x1]
    for byte in 0..16u64 {
        assert_eq!(bus.read(st1_one_no_offset_out + byte, 1), Some(0x60 + byte));
    }
    assert_eq!(cpu.regs.x(1), st1_one_no_offset_out);

    let st1_one_q0_out = RAM_BASE + 0x4000;
    cpu.regs.set_x(10, st1_one_q0_out);
    cpu.simd[29] = 0xffff_ffff_ffff_ffff_b7b6_b5b4_b3b2_b1b0;
    execute(&mut cpu, &mut bus, decode(0x0C9F_795D).unwrap()).unwrap(); // st1 {v29.2s}, [x10], #8
    for byte in 0..8u64 {
        assert_eq!(bus.read(st1_one_q0_out + byte, 1), Some(0xb0 + byte));
    }
    assert_eq!(cpu.regs.x(10), st1_one_q0_out + 8);

    let st1_one_post_reg_out = RAM_BASE + 0x4400;
    cpu.regs.set_x(0, st1_one_post_reg_out);
    cpu.regs.set_x(4, 24);
    cpu.simd[0] = vector_bytes(0xd0);
    execute(&mut cpu, &mut bus, decode(0x4C84_7800).unwrap()).unwrap(); // st1 {v0.4s}, [x0], x4
    for byte in 0..16u64 {
        assert_eq!(bus.read(st1_one_post_reg_out + byte, 1), Some(0xd0 + byte));
    }
    assert_eq!(cpu.regs.x(0), st1_one_post_reg_out + 24);
}
