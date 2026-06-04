use super::*;

#[test]
fn decode_sha512_three_register_forms_cross_checked_with_disarm64() {
    assert_decode_cases(&[
        (0xCE66_80A2, Opcode::SimdSha512H, "sha512h"),
        (0xCE63_8402, Opcode::SimdSha512H2, "sha512h2"),
        (0xCE67_8AD7, Opcode::SimdSha512Su1, "sha512su1"),
    ]);

    let h = decode(0xCE66_80A2).unwrap(); // sha512h q2, q5, v6.2d
    assert_eq!((h.rd, h.rn, h.rm, h.size), (2, 5, 6, 16));

    let h2 = decode(0xCE63_8402).unwrap(); // sha512h2 q2, q0, v3.2d
    assert_eq!((h2.rd, h2.rn, h2.rm, h2.size), (2, 0, 3, 16));

    let su1 = decode(0xCE67_8AD7).unwrap(); // sha512su1 v23.2d, v22.2d, v7.2d
    assert_eq!((su1.rd, su1.rn, su1.rm, su1.size), (23, 22, 7, 16));
}
