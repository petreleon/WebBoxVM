use emulator::arm64::decode;

fn main() {
    let data = std::fs::read("/tmp/kernel_raw.bin").unwrap();
    for i in (0..data.len()).step_by(4) {
        if i + 4 > data.len() { break; }
        let word = u32::from_le_bytes([data[i], data[i+1], data[i+2], data[i+3]]);
        if let Some(instr) = decode(word) {
            println!("0x{:08x} -> {:?} imm={:x} rd={}", word, instr.op, instr.imm, instr.rd);
        } else {
            println!("0x{:08x} -> ???", word);
        }
    }
}
