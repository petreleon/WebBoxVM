use emulator::arm64::decode;

fn main() {
    for arg in std::env::args().skip(1) {
        let word = u32::from_str_radix(arg.trim_start_matches("0x"), 16)
            .expect("expected 32-bit hex word");
        let dis = disarm64::decoder::decode(word)
            .map(|opcode| opcode.to_string())
            .unwrap_or_else(|| "<undecoded>".to_string());
        println!("0x{word:08x}: {dis:32} {:?}", decode(word));
    }
}
