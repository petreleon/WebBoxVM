use super::*;

#[test]
fn simd_cmeq_register_compares_word_lanes() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[31] = 0x1111_1111_3333_3333_2222_2222_1111_1111;
    cpu.simd[25] = 0x4444_4444_3333_3333_0000_0000_1111_1111;

    execute(&mut cpu, &mut bus, decode(0x6EB9_8FFF).unwrap()).unwrap(); // cmeq v31.4s, v31.4s, v25.4s

    assert_eq!(cpu.simd[31], 0x0000_0000_ffff_ffff_0000_0000_ffff_ffff);
}

#[test]
fn simd_cmhi_register_compares_unsigned_lanes() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[31] = 0x0000_0007_0000_0005_ffff_ffff_0000_0003;
    cpu.simd[29] = 0x0000_0006_0000_0005_ffff_fffe_0000_0004;
    execute(&mut cpu, &mut bus, decode(0x6EBD_37FD).unwrap()).unwrap(); // cmhi v29.4s, v31.4s, v29.4s
    assert_eq!(cpu.simd[29], 0xffff_ffff_0000_0000_ffff_ffff_0000_0000);

    cpu.simd[31] = 0x0000_0000_0000_0000_0000_0000_0000_0005;
    cpu.simd[29] = 0xffff_ffff_ffff_ffff_0000_0000_0000_0004;
    execute(&mut cpu, &mut bus, decode(0x6EFD_37FC).unwrap()).unwrap(); // cmhi v28.2d, v31.2d, v29.2d
    assert_eq!(cpu.simd[28], 0x0000_0000_0000_0000_ffff_ffff_ffff_ffff);

    cpu.simd[30] = 7;
    cpu.simd[31] = 6;
    execute(&mut cpu, &mut bus, decode(0x7EFF_37DF).unwrap()).unwrap(); // cmhi d31, d30, d31
    assert_eq!(cpu.simd[31], u64::MAX as u128);
}

#[test]
fn simd_scalar_uqsub_saturates_halfword() {
    let (mut cpu, mut bus) = setup();
    let qc = 1 << 27;

    cpu.simd[31] = 0xffff_ffff_0000_1234;
    cpu.simd[15] = 0x0100;
    execute(&mut cpu, &mut bus, decode(0x7E6F_2FFF).unwrap()).unwrap(); // uqsub h31, h31, h15
    assert_eq!(cpu.simd[31], 0x1134);
    assert_eq!(cpu.sys.fpsr & qc, 0);

    cpu.simd[31] = 0x0002;
    cpu.simd[15] = 0x0003;
    execute(&mut cpu, &mut bus, decode(0x7E6F_2FFF).unwrap()).unwrap();
    assert_eq!(cpu.simd[31], 0);
    assert_ne!(cpu.sys.fpsr & qc, 0);
}

#[test]
fn simd_strlen_prefix_matches_debian_libc_fast_path() {
    let (mut cpu, mut bus) = setup();
    let base = RAM_BASE + 0x4000;

    for (offset, bytes, expected) in [
        (0u64, b"/\0".as_slice(), 1u64),
        (3u64, b"sys\0".as_slice(), 3u64),
    ] {
        for i in 0..32u64 {
            bus.write(base + i, 1, 0xaa);
        }
        for (i, byte) in bytes.iter().enumerate() {
            bus.write(base + offset + i as u64, 1, *byte as u64);
        }

        cpu.regs.set_x(0, base + offset);
        execute(&mut cpu, &mut bus, decode(0x927C_EC01).unwrap()).unwrap(); // and x1, x0, #~0xf
        execute(&mut cpu, &mut bus, decode(0x4C40_7020).unwrap()).unwrap(); // ld1 {v0.16b}, [x1]
        execute(&mut cpu, &mut bus, decode(0x4E20_9801).unwrap()).unwrap(); // cmeq v1.16b, v0.16b, #0
        execute(&mut cpu, &mut bus, decode(0xD37E_F404).unwrap()).unwrap(); // lsl x4, x0, #2
        execute(&mut cpu, &mut bus, decode(0x0F0C_8422).unwrap()).unwrap(); // shrn v2.8b, v1.8h, #4
        execute(&mut cpu, &mut bus, decode(0x9E66_0042).unwrap()).unwrap(); // fmov x2, d2
        execute(&mut cpu, &mut bus, decode(0x9AC4_2442).unwrap()).unwrap(); // lsr x2, x2, x4
        assert_ne!(cpu.regs.x(2), 0);
        execute(&mut cpu, &mut bus, decode(0xDAC0_0042).unwrap()).unwrap(); // rbit x2, x2
        execute(&mut cpu, &mut bus, decode(0xDAC0_1040).unwrap()).unwrap(); // clz x0, x2
        execute(&mut cpu, &mut bus, decode(0xD342_FC00).unwrap()).unwrap(); // lsr x0, x0, #2

        assert_eq!(cpu.regs.x(0), expected);
    }
}

#[test]
fn simd_strlen_page_boundary_matches_debian_libc_path() {
    let (mut cpu, mut bus) = setup();
    let page = RAM_BASE + 0x5000;
    let ptr = page + 0xff0;

    for (bytes, expected) in [
        (b"/\0".as_slice(), 1u64),
        (b"sys\0".as_slice(), 3u64),
        (b"devices\0".as_slice(), 7u64),
    ] {
        for i in 0xfe0..0x1000u64 {
            bus.write(page + i, 1, 0xaa);
        }
        for (i, byte) in bytes.iter().enumerate() {
            bus.write(ptr + i as u64, 1, *byte as u64);
        }

        cpu.regs.set_x(0, ptr);
        execute(&mut cpu, &mut bus, decode(0x927B_E801).unwrap()).unwrap(); // and x1, x0, #~0x1f
        execute(&mut cpu, &mut bus, decode(0x5281_8062).unwrap()).unwrap(); // mov w2, #0xc03
        execute(&mut cpu, &mut bus, decode(0x72B8_0602).unwrap()).unwrap(); // movk w2, #0xc030, lsl #16
        execute(&mut cpu, &mut bus, decode(0x4C40_A021).unwrap()).unwrap(); // ld1 {v1.16b, v2.16b}, [x1]
        execute(&mut cpu, &mut bus, decode(0x4E04_0C40).unwrap()).unwrap(); // dup v0.4s, w2
        execute(&mut cpu, &mut bus, decode(0x4E20_9821).unwrap()).unwrap(); // cmeq v1.16b, v1.16b, #0
        execute(&mut cpu, &mut bus, decode(0x4E20_9842).unwrap()).unwrap(); // cmeq v2.16b, v2.16b, #0
        execute(&mut cpu, &mut bus, decode(0x4E20_1C21).unwrap()).unwrap(); // and v1.16b, v1.16b, v0.16b
        execute(&mut cpu, &mut bus, decode(0x4E20_1C42).unwrap()).unwrap(); // and v2.16b, v2.16b, v0.16b
        execute(&mut cpu, &mut bus, decode(0x4E22_BC20).unwrap()).unwrap(); // addp v0.16b, v1.16b, v2.16b
        execute(&mut cpu, &mut bus, decode(0x4E20_BC00).unwrap()).unwrap(); // addp v0.16b, v0.16b, v0.16b
        execute(&mut cpu, &mut bus, decode(0x9E66_0003).unwrap()).unwrap(); // fmov x3, d0
        execute(&mut cpu, &mut bus, decode(0xD37F_F804).unwrap()).unwrap(); // lsl x4, x0, #1
        execute(&mut cpu, &mut bus, decode(0x9AC4_2463).unwrap()).unwrap(); // lsr x3, x3, x4
        assert_ne!(cpu.regs.x(3), 0);
        execute(&mut cpu, &mut bus, decode(0xDAC0_0063).unwrap()).unwrap(); // rbit x3, x3
        execute(&mut cpu, &mut bus, decode(0xDAC0_1060).unwrap()).unwrap(); // clz x0, x3
        execute(&mut cpu, &mut bus, decode(0xD341_FC00).unwrap()).unwrap(); // lsr x0, x0, #1

        assert_eq!(cpu.regs.x(0), expected);
    }
}
