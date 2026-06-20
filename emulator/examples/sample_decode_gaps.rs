use emulator::arch::arm64::decode;
use std::collections::BTreeMap;

#[derive(Default)]
struct Gap {
    count: usize,
    raw: u32,
}

fn main() {
    let samples = std::env::args()
        .nth(1)
        .and_then(|arg| arg.parse::<usize>().ok())
        .unwrap_or(1_000_000);
    let mut rng = 0x3243_f6a8_8885_a308u64;
    let mut gaps: BTreeMap<String, Gap> = BTreeMap::new();
    let mut decoded = 0usize;

    for _ in 0..samples {
        rng = rng.wrapping_mul(6364136223846793005).wrapping_add(1);
        let raw = (rng >> 32) as u32;
        let Some(d64) = disarm64::decoder::decode(raw) else {
            continue;
        };
        decoded += 1;
        if decode(raw).is_some() {
            continue;
        }

        let mnemonic = format!("{:?}", d64.mnemonic);
        let entry = gaps.entry(mnemonic).or_default();
        entry.count += 1;
        if entry.raw == 0 {
            entry.raw = raw;
        }
    }

    let mut ranked: Vec<_> = gaps.into_iter().collect();
    ranked.sort_by_key(|(_, gap)| std::cmp::Reverse(gap.count));

    println!("samples={samples} disarm64_decoded={decoded}");
    for (mnemonic, gap) in ranked.into_iter().take(80) {
        println!("{:6} raw=0x{:08x} {}", gap.count, gap.raw, mnemonic);
    }
}
