use super::*;

#[test]
fn test_decode_eb21c01f() {
    let raw: u32 = 0xeb21c01f;
    let instr = decode(raw).unwrap();
    println!(
        "raw=0x{:08x} op={:?} rd={} rn={} rm={}",
        raw, instr.op, instr.rd, instr.rn, instr.rm
    );
    assert_eq!(instr.op, Opcode::Cmp, "Expected Cmp, got {:?}", instr.op);
}

#[test]
fn test_decode_eb21c01f_debug() {
    let raw: u32 = 0xeb21c01f;
    let sf = ((raw >> 31) & 1) != 0;
    let op = (raw >> 30) & 1;
    let s = ((raw >> 29) & 1) != 0;
    let n = ((raw >> 21) & 1) != 0;
    let rd = (raw & 0x1F) as u8;
    println!("sf={} op={} s={} n={} rd={}", sf, op, s, n, rd);
    if s && op == 1 && rd == 31 {
        println!("Would be Cmp");
    } else {
        println!("Would NOT be Cmp");
    }
}

#[test]
fn test_decode_121d7820() {
    let raw: u32 = 0x121d7820;
    match decode(raw) {
        Some(instr) => println!(
            "raw=0x{:08x} op={:?} rd={} rn={} imm={:#x}",
            raw, instr.op, instr.rd, instr.rn, instr.imm
        ),
        None => println!("raw=0x{:08x} = None", raw),
    }
}
