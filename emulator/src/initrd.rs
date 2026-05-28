//! Cpio `newc` initrd parser and loader.

use crate::bus::SystemBus;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpioEntry {
    pub name: String,
    pub data: Vec<u8>,
    pub mode: u32,
}

/// Parse a `newc` cpio archive and return its entries.
pub fn parse_cpio(data: &[u8]) -> Result<Vec<CpioEntry>, &'static str> {
    let mut entries = Vec::new();
    let mut offset = 0usize;

    while offset + 110 <= data.len() {
        let magic = std::str::from_utf8(&data[offset..offset + 6])
            .map_err(|_| "invalid cpio magic")?;
        if magic != "070701" && magic != "070702" {
            return Err("bad cpio magic");
        }

        let read_hex = |o: usize, n: usize| -> Result<u32, &'static str> {
            let s = std::str::from_utf8(&data[o..o + n]).map_err(|_| "invalid hex")?;
            u32::from_str_radix(s, 16).map_err(|_| "bad hex")
        };

        let _ino = read_hex(offset + 6, 8)?;
        let mode = read_hex(offset + 14, 8)?;
        let _uid = read_hex(offset + 22, 8)?;
        let _gid = read_hex(offset + 30, 8)?;
        let _nlink = read_hex(offset + 38, 8)?;
        let _mtime = read_hex(offset + 46, 8)?;
        let filesize = read_hex(offset + 54, 8)? as usize;
        let _devmajor = read_hex(offset + 62, 8)?;
        let _devminor = read_hex(offset + 70, 8)?;
        let _rdevmajor = read_hex(offset + 78, 8)?;
        let _rdevminor = read_hex(offset + 86, 8)?;
        let namesize = read_hex(offset + 94, 8)? as usize;
        let _check = read_hex(offset + 102, 8)?;

        offset += 110;

        if offset + namesize > data.len() {
            return Err("name truncated");
        }
        let name_bytes = &data[offset..offset + namesize - 1]; // drop null terminator
        let name = String::from_utf8_lossy(name_bytes).into_owned();
        offset = align4(offset + namesize);

        if name == "TRAILER!!!" {
            break;
        }

        if offset + filesize > data.len() {
            return Err("file truncated");
        }
        let file_data = data[offset..offset + filesize].to_vec();
        offset = align4(offset + filesize);

        entries.push(CpioEntry { name, data: file_data, mode });
    }

    Ok(entries)
}

/// Build a `newc` cpio archive from entries.
pub fn build_cpio(entries: &[(String, Vec<u8>, u32)]) -> Vec<u8> {
    let mut out = Vec::new();
    for (name, data, mode) in entries {
        push_header(&mut out, name, data.len() as u32, *mode);
        out.extend_from_slice(name.as_bytes());
        out.push(0); // null terminator
        pad_to_4(&mut out);
        out.extend_from_slice(data);
        pad_to_4(&mut out);
    }
    // Trailer
    push_header(&mut out, "TRAILER!!!", 0, 0);
    out.extend_from_slice(b"TRAILER!!!\0");
    pad_to_4(&mut out);
    out
}

fn push_header(out: &mut Vec<u8>, name: &str, filesize: u32, mode: u32) {
    let namesize = name.len() + 1;
    out.extend_from_slice(b"070701");
    out.extend_from_slice(format!("{:08x}", 0).as_bytes()); // ino
    out.extend_from_slice(format!("{:08x}", mode).as_bytes());
    out.extend_from_slice(format!("{:08x}", 0).as_bytes()); // uid
    out.extend_from_slice(format!("{:08x}", 0).as_bytes()); // gid
    out.extend_from_slice(format!("{:08x}", 1).as_bytes()); // nlink
    out.extend_from_slice(format!("{:08x}", 0).as_bytes()); // mtime
    out.extend_from_slice(format!("{:08x}", filesize).as_bytes());
    out.extend_from_slice(format!("{:08x}", 0).as_bytes()); // devmajor
    out.extend_from_slice(format!("{:08x}", 0).as_bytes()); // devminor
    out.extend_from_slice(format!("{:08x}", 0).as_bytes()); // rdevmajor
    out.extend_from_slice(format!("{:08x}", 0).as_bytes()); // rdevminor
    out.extend_from_slice(format!("{:08x}", namesize).as_bytes());
    out.extend_from_slice(format!("{:08x}", 0).as_bytes()); // check
}

fn pad_to_4(v: &mut Vec<u8>) {
    while v.len() % 4 != 0 {
        v.push(0);
    }
}

fn align4(n: usize) -> usize {
    (n + 3) & !3
}

/// Load a cpio archive into memory at `addr`.
pub fn load_initrd(bus: &mut SystemBus, addr: u64, data: &[u8]) {
    for (i, &byte) in data.iter().enumerate() {
        bus.write(addr + i as u64, 1, byte as u64);
    }
}

#[cfg(test)]
mod tests;
