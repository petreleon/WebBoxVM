use super::simd_helpers::*;
use super::*;

#[test]
fn simd_userland_table_and_permute_ops() {
    let (mut cpu, mut bus) = setup();

    cpu.simd[30] = 0x0000_0004_0000_0003_0000_0002_0000_0001;
    cpu.simd[28] = 0x0000_0008_0000_0007_0000_0006_0000_0005;
    execute(&mut cpu, &mut bus, decode(0x4E9C_1BDE).unwrap()).unwrap(); // uzp1 v30.4s, v30.4s, v28.4s
    assert_eq!(cpu.simd[30], 0x0000_0007_0000_0005_0000_0003_0000_0001);

    cpu.simd[8] = vector_bytes(0);
    cpu.simd[12] = vector_bytes(0x80);
    execute(&mut cpu, &mut bus, decode(0x4E0C_290B).unwrap()).unwrap(); // trn1 v11.16b, v8.16b, v12.16b
    assert_eq!(cpu.simd[11], 0x8e0e_8c0c_8a0a_8808_8606_8404_8202_8000);

    cpu.simd[12] = 0x0000_0004_0000_0003_0000_0002_0000_0001;
    cpu.simd[14] = 0x1004_0000_1003_0000_1002_0000_1001_0000;
    execute(&mut cpu, &mut bus, decode(0x4ECE_298B).unwrap()).unwrap(); // trn1 v11.2d, v12.2d, v14.2d
    assert_eq!(cpu.simd[11], 0x1002_0000_1001_0000_0000_0002_0000_0001);

    cpu.simd[31] = vector_bytes(0);
    cpu.simd[27] = vector_bytes(0x80);
    execute(&mut cpu, &mut bus, decode(0x4E1B_3BFD).unwrap()).unwrap(); // zip1 v29.16b, v31.16b, v27.16b
    assert_eq!(cpu.simd[29], 0x8707_8606_8505_8404_8303_8202_8101_8000);
    execute(&mut cpu, &mut bus, decode(0x4E1B_7BFF).unwrap()).unwrap(); // zip2 v31.16b, v31.16b, v27.16b
    assert_eq!(cpu.simd[31], 0x8f0f_8e0e_8d0d_8c0c_8b0b_8a0a_8909_8808);

    cpu.simd[29] = 0x0008_0007_0006_0005_0004_0003_0002_0001;
    cpu.simd[27] = 0x1008_1007_1006_1005_1004_1003_1002_1001;
    execute(&mut cpu, &mut bus, decode(0x4E5B_3BBE).unwrap()).unwrap(); // zip1 v30.8h, v29.8h, v27.8h
    assert_eq!(cpu.simd[30], 0x1004_0004_1003_0003_1002_0002_1001_0001);
    execute(&mut cpu, &mut bus, decode(0x4E5B_7BBD).unwrap()).unwrap(); // zip2 v29.8h, v29.8h, v27.8h
    assert_eq!(cpu.simd[29], 0x1008_0008_1007_0007_1006_0006_1005_0005);

    cpu.simd[31] = vector_bytes(0x40);
    cpu.simd[23] = 0x100f_0e0d_0c0b_0a09_0807_0605_0403_0201;
    execute(&mut cpu, &mut bus, decode(0x4E17_03FF).unwrap()).unwrap(); // tbl v31.16b, {v31.16b}, v23.16b
    assert_eq!(cpu.simd[31], 0x004f_4e4d_4c4b_4a49_4847_4645_4443_4241);

    cpu.simd[1] = repeat_byte(0xA0);
    cpu.simd[4] = bytes16([0, 17, 33, 49, 63, 64, 2, 18, 34, 50, 62, 99, 3, 19, 35, 51]);
    cpu.simd[20] = vector_bytes(0x10);
    cpu.simd[21] = vector_bytes(0x20);
    cpu.simd[22] = vector_bytes(0x30);
    cpu.simd[23] = vector_bytes(0x40);
    execute(&mut cpu, &mut bus, decode(0x4E04_7281).unwrap()).unwrap(); // tbx v1.16b, {v20.16b-v23.16b}, v4.16b
    assert_eq!(
        cpu.simd[1],
        bytes16([
            0x10, 0x21, 0x31, 0x41, 0x4F, 0xA0, 0x12, 0x22, 0x32, 0x42, 0x4E, 0xA0, 0x13, 0x23,
            0x33, 0x43
        ])
    );
}

fn bytes16(bytes: [u8; 16]) -> u128 {
    bytes
        .into_iter()
        .enumerate()
        .fold(0u128, |out, (lane, byte)| {
            out | ((byte as u128) << (lane * 8))
        })
}

fn repeat_byte(byte: u8) -> u128 {
    bytes16([byte; 16])
}
