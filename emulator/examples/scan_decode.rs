use emulator::arm64::{Opcode, decode};
use std::collections::BTreeMap;
use std::fs;

#[derive(Debug)]
struct Section {
    name: String,
    offset: usize,
    size: usize,
    addr: u64,
}

fn main() {
    for path in std::env::args().skip(1) {
        let bytes = fs::read(&path).expect("read ELF");
        let sections = elf_sections(&bytes).expect("parse sections");
        let mut suspicious: BTreeMap<(String, String, u32), usize> = BTreeMap::new();
        let mut all: BTreeMap<(String, String), usize> = BTreeMap::new();

        for section in sections.iter().filter(|s| s.name == ".text") {
            for rel in (0..section.size.saturating_sub(3)).step_by(4) {
                let off = section.offset + rel;
                let raw = u32::from_le_bytes(bytes[off..off + 4].try_into().unwrap());
                let Some(d64) = disarm64::decoder::decode(raw) else {
                    continue;
                };
                let mnemonic = format!("{:?}", d64.mnemonic);
                let legacy = decode(raw)
                    .map(|i| format!("{:?}", i.op))
                    .unwrap_or_else(|| "None".to_string());
                *all.entry((mnemonic.clone(), legacy.clone())).or_default() += 1;

                let legacy_instr = decode(raw);
                let nop_like = matches!(legacy_instr.map(|i| i.op), None | Some(Opcode::Nop));
                if nop_like && !matches!(mnemonic.as_str(), "hint" | "prfm" | "nop" | "bti") {
                    let va = section.addr + rel as u64;
                    let dis = d64.to_string();
                    *suspicious
                        .entry((format!("{va:#x} {dis}"), legacy, raw))
                        .or_default() += 1;
                }
            }
        }

        println!("== {path} ==");
        println!("suspicious:");
        for ((dis, legacy, raw), count) in suspicious.iter().take(200) {
            println!("  {count:4} raw=0x{raw:08x} legacy={legacy:16} {dis}");
        }
        println!("top mnemonic/legacy pairs:");
        let mut pairs: Vec<_> = all.into_iter().collect();
        pairs.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
        for ((mnemonic, legacy), count) in pairs.into_iter().take(80) {
            println!("  {count:5} {mnemonic:18} -> {legacy}");
        }
    }
}

fn elf_sections(bytes: &[u8]) -> Option<Vec<Section>> {
    if bytes.get(0..4)? != b"\x7fELF" || *bytes.get(4)? != 2 || *bytes.get(5)? != 1 {
        return None;
    }

    let shoff = read_u64(bytes, 0x28)? as usize;
    let shentsize = read_u16(bytes, 0x3a)? as usize;
    let shnum = read_u16(bytes, 0x3c)? as usize;
    let shstrndx = read_u16(bytes, 0x3e)? as usize;
    let shstr = section_header(bytes, shoff, shentsize, shstrndx)?;
    let shstr_bytes = bytes.get(shstr.offset..shstr.offset + shstr.size)?;

    let mut sections = Vec::new();
    for index in 0..shnum {
        let header = section_header(bytes, shoff, shentsize, index)?;
        let name = cstr_at(shstr_bytes, header.name_offset)?;
        sections.push(Section {
            name,
            offset: header.offset,
            size: header.size,
            addr: header.addr,
        });
    }
    Some(sections)
}

struct SectionHeader {
    name_offset: usize,
    offset: usize,
    size: usize,
    addr: u64,
}

fn section_header(
    bytes: &[u8],
    shoff: usize,
    shentsize: usize,
    index: usize,
) -> Option<SectionHeader> {
    let base = shoff.checked_add(index.checked_mul(shentsize)?)?;
    Some(SectionHeader {
        name_offset: read_u32(bytes, base)? as usize,
        addr: read_u64(bytes, base + 0x10)?,
        offset: read_u64(bytes, base + 0x18)? as usize,
        size: read_u64(bytes, base + 0x20)? as usize,
    })
}

fn cstr_at(bytes: &[u8], offset: usize) -> Option<String> {
    let tail = bytes.get(offset..)?;
    let end = tail.iter().position(|b| *b == 0)?;
    Some(String::from_utf8_lossy(&tail[..end]).into_owned())
}

fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes(
        bytes.get(offset..offset + 2)?.try_into().ok()?,
    ))
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

fn read_u64(bytes: &[u8], offset: usize) -> Option<u64> {
    Some(u64::from_le_bytes(
        bytes.get(offset..offset + 8)?.try_into().ok()?,
    ))
}
