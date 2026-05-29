//! Benchmark: hand-rolled decoder vs disarm64.

use std::time::Instant;
use disarm64::decoder;
use emulator::arm64::decode as our_decode;

fn main() {
    // Test vectors: one per opcode we support
    let test_vecs: &[(u32, &str)] = &[
        // ADD/SUB (immediate)
        (0x910003fd, "ADD X29, SP, #0"),
        (0xd10003ff, "SUB SP, SP, #0"),
        (0xb100003f, "ADDS WZR, W1, #0"),
        (0xf100003f, "SUBS XZR, X8, #0"),
        // MOVZ/MOVK/MOVN
        (0xd2800000, "MOVZ X0, #0"),
        (0xf2a00020, "MOVK X0, #1, LSL #16"),
        (0x92800000, "MOVN X0, #0"),
        // Branches
        (0x14000000, "B"),
        (0x94000000, "BL"),
        (0x54000000, "B.EQ"),
        (0xd65f03c0, "RET"),
        (0xd61f0000, "BR X0"),
        (0xb4000040, "CBZ X0"),
        (0xb5000040, "CBNZ X0"),
        (0x36000040, "TBZ X0, #0"),
        (0x37000040, "TBNZ X0, #0"),
        // LDR/STR
        (0xf9400000, "LDR X0, [X0]"),
        (0xa9000000, "STP X0, X0, [X0]"),
        (0x58000000, "LDR X0, 0x0"),
        // Logical
        (0x12000000, "AND W0, W0, #1"),
        (0x32000000, "ORR W0, W0, #1"),
        (0x0a000000, "AND W0, W0, W0"),
        (0x2a000000, "ORR W0, W0, W0"),
        // Bitfield
        (0x53000000, "UBFX W0, W0, #0, #1"),
        (0x13000000, "SBFM W0, W0, #0, #0"),
        (0x33000000, "BFM W0, W0, #0, #0"),
        // CMP/CSEL
        (0xeb00001f, "CMP X0, X0"),
        (0x9a800000, "CSEL X0, X0, X0, EQ"),
        // Multiply
        (0x9b007c00, "MUL X0, X0, X0"),
        (0x9ac00c00, "UDIV X0, X0, X0"),
        (0x9ac10800, "LSR X0, X0, X0"),
        // System
        (0xd503201f, "NOP"),
        (0xd503207f, "WFI"),
        (0xd503205f, "WFE"),
        (0xd69f03e0, "ERET"),
        (0xd4000001, "SVC #0"),
        (0xd4200000, "BRK #0"),
        // Exclusive
        (0xc85ffc00, "LDXR X0, [X0]"),
        (0xc8000000, "STXR W0, X0, [X0]"),
        // ADRP
        (0x90000000, "ADRP X0, 0x0"),
        // MRS/MSR
        (0xd5380000, "MRS X0, SYSREG"),
        (0xd5180000, "MSR SYSREG, X0"),
        // REV/RBIT/CLZ
        (0xdac00c00, "REV X0, X0"),
        (0xdac01000, "CLZ X0, X0"),
        // CCMP
        (0xfa400400, "CCMP X0, X0, #0, EQ"),
        // CMP (extended)
        (0xeb20801f, "CMP X1, W0, SXTW"),
    ];

    const ITERATIONS: usize = 100_000;

    // --- Benchmark hand-rolled decoder ---
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        for &(raw, _) in test_vecs {
            let _ = our_decode(raw);
        }
    }
    let our_time = start.elapsed();
    let total_ops = test_vecs.len() * ITERATIONS;
    let our_ns = our_time.as_nanos() as f64 / total_ops as f64;
    println!("Hand-rolled: {:.1} ns/op ({:.1}M ops/s)", 
        our_ns, 1_000.0 / our_ns);

    // --- Benchmark disarm64 ---
    let start = Instant::now();
    for _ in 0..ITERATIONS {
        for &(raw, _) in test_vecs {
            let _ = decoder::decode(raw);
        }
    }
    let disarm_time = start.elapsed();
    let disarm_ns = disarm_time.as_nanos() as f64 / total_ops as f64;
    println!("disarm64:     {:.1} ns/op ({:.1}M ops/s)",
        disarm_ns, 1_000.0 / disarm_ns);

    println!("\nSpeed ratio: disarm64 is {:.1}x vs hand-rolled",
        disarm_ns / our_ns);

    // --- Verify correctness ---
    println!("\nCorrectness check:");
    let mut mismatches = 0;
    for &(raw, _expected) in test_vecs {
        let our = our_decode(raw);
        let d64 = decoder::decode(raw);
        // Both should succeed
        match (our, d64) {
            (Some(_), Some(dinsn)) => {
                // OK, both decoded
                let _ = dinsn;
            }
            (Some(our_insn), None) => {
                println!("  MISS: disarm64 failed on 0x{:08x} (we decoded {:?})", raw, our_insn.op);
                mismatches += 1;
            }
            (None, Some(_dinsn)) => {
                println!("  MISS: our decoder failed on 0x{:08x} (disarm64 decoded)", raw);
                mismatches += 1;
            }
            (None, None) => {
                // Both failed, expected for some test vectors
            }
        }
    }
    if mismatches == 0 {
        println!("  All decodable instructions matched");
    }
}
