use emulator::arm64::decode;
use std::io::{self, Read};

fn main() {
    let words = input_words();
    for raw in words {
        let mnemonic = disarm64::decoder::decode(raw)
            .map(|opcode| format!("{:?}", opcode.mnemonic))
            .unwrap_or_else(|| "None".to_string());
        let legacy = decode(raw)
            .map(|instr| format!("{:?}", instr.op))
            .unwrap_or_else(|| "None".to_string());
        println!("0x{raw:08x} {mnemonic} {legacy}");
    }
}

fn input_words() -> Vec<u32> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let text = if args.is_empty() {
        let mut stdin = String::new();
        io::stdin().read_to_string(&mut stdin).expect("read stdin");
        stdin
    } else {
        args.join("\n")
    };

    text.split_whitespace().map(parse_word).collect()
}

fn parse_word(word: &str) -> u32 {
    u32::from_str_radix(word.trim_start_matches("0x"), 16).expect("expected 32-bit hex word")
}
