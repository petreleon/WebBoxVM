use super::*;

const EXP_COEFF_H: [u16; 32] = [
    0x000, 0x016, 0x02d, 0x045, 0x05d, 0x075, 0x08e, 0x0a8, 0x0c2, 0x0dc, 0x0f8, 0x114, 0x130,
    0x14d, 0x16b, 0x189, 0x1a8, 0x1c8, 0x1e8, 0x209, 0x22b, 0x24e, 0x271, 0x295, 0x2ba, 0x2e0,
    0x306, 0x32e, 0x356, 0x37f, 0x3a9, 0x3d4,
];

const EXP_COEFF_S: [u32; 64] = [
    0x000000, 0x0164d2, 0x02cd87, 0x043a29, 0x05aac3, 0x071f62, 0x08980f, 0x0a14d5, 0x0b95c2,
    0x0d1adf, 0x0ea43a, 0x1031dc, 0x11c3d3, 0x135a2b, 0x14f4f0, 0x16942d, 0x1837f0, 0x19e046,
    0x1b8d3a, 0x1d3eda, 0x1ef532, 0x20b051, 0x227043, 0x243516, 0x25fed7, 0x27cd94, 0x29a15b,
    0x2b7a3a, 0x2d583f, 0x2f3b79, 0x3123f6, 0x3311c4, 0x3504f3, 0x36fd92, 0x38fbaf, 0x3aff5b,
    0x3d08a4, 0x3f179a, 0x412c4d, 0x4346cd, 0x45672a, 0x478d75, 0x49b9be, 0x4bec15, 0x4e248c,
    0x506334, 0x52a81e, 0x54f35b, 0x5744fd, 0x599d16, 0x5bfbb8, 0x5e60f5, 0x60ccdf, 0x633f89,
    0x65b907, 0x68396a, 0x6ac0c7, 0x6d4f30, 0x6fe4ba, 0x728177, 0x75257d, 0x77d0df, 0x7a83b3,
    0x7d3e0c,
];

const EXP_COEFF_D: [u64; 64] = [
    0x0000000000000,
    0x02c9a3e778061,
    0x059b0d3158574,
    0x0874518759bc8,
    0x0b5586cf9890f,
    0x0e3ec32d3d1a2,
    0x11301d0125b51,
    0x1429aaea92de0,
    0x172b83c7d517b,
    0x1a35beb6fcb75,
    0x1d4873168b9aa,
    0x2063b88628cd6,
    0x2387a6e756238,
    0x26b4565e27cdd,
    0x29e9df51fdee1,
    0x2d285a6e4030b,
    0x306fe0a31b715,
    0x33c08b26416ff,
    0x371a7373aa9cb,
    0x3a7db34e59ff7,
    0x3dea64c123422,
    0x4160a21f72e2a,
    0x44e086061892d,
    0x486a2b5c13cd0,
    0x4bfdad5362a27,
    0x4f9b2769d2ca7,
    0x5342b569d4f82,
    0x56f4736b527da,
    0x5ab07dd485429,
    0x5e76f15ad2148,
    0x6247eb03a5585,
    0x6623882552225,
    0x6a09e667f3bcd,
    0x6dfb23c651a2f,
    0x71f75e8ec5f74,
    0x75feb564267c9,
    0x7a11473eb0187,
    0x7e2f336cf4e62,
    0x82589994cce13,
    0x868d99b4492ed,
    0x8ace5422aa0db,
    0x8f1ae99157736,
    0x93737b0cdc5e5,
    0x97d829fde4e50,
    0x9c49182a3f090,
    0xa0c667b5de565,
    0xa5503b23e255d,
    0xa9e6b5579fdbf,
    0xae89f995ad3ad,
    0xb33a2b84f15fb,
    0xb7f76f2fb5e47,
    0xbcc1e904bc1d2,
    0xc199bdd85529c,
    0xc67f12e57d14b,
    0xcb720dcef9069,
    0xd072d4a07897c,
    0xd5818dcfba487,
    0xda9e603db3285,
    0xdfc97337b9b5f,
    0xe502ee78b3ff6,
    0xea4afa2a490da,
    0xefa1bee615a27,
    0xf50765b6e4540,
    0xfa7c1819e90d8,
];

pub(in crate::arm64::execute) fn exec_sve_fp_fexpa(cpu: &mut Armv8Cpu, instr: Instr) {
    let element_size = instr.size as usize;
    let elements = sve_vl_bytes(cpu) / element_size;
    let source = sve_read_z(cpu, instr.rn as usize);
    let mut result = [0; 256];

    for element in 0..elements {
        let value = sve_element(&source, element, element_size);
        sve_set_element(
            &mut result,
            element,
            element_size,
            fexpa(value, element_size),
        );
    }

    sve_write_z(cpu, instr.rd as usize, result);
}

fn fexpa(value: u64, element_size: usize) -> u64 {
    match element_size {
        2 => fexpa_h(value as u16) as u64,
        4 => fexpa_s(value as u32) as u64,
        8 => fexpa_d(value),
        _ => unreachable!(),
    }
}

fn fexpa_h(value: u16) -> u16 {
    let exponent = (value >> 5) & 0x1f;
    let coeff = EXP_COEFF_H[(value & 0x1f) as usize];
    (exponent << 10) | coeff
}

fn fexpa_s(value: u32) -> u32 {
    let exponent = (value >> 6) & 0xff;
    let coeff = EXP_COEFF_S[(value & 0x3f) as usize];
    (exponent << 23) | coeff
}

fn fexpa_d(value: u64) -> u64 {
    let exponent = (value >> 6) & 0x7ff;
    let coeff = EXP_COEFF_D[(value & 0x3f) as usize];
    (exponent << 52) | coeff
}
