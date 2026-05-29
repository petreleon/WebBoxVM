//! cpio `newc` format initrd parser and builder.
//!
//! The initrd (initial RAM disk) is a cpio archive loaded into memory that
//! the kernel mounts as its first root filesystem.  The `newc` format uses
//! ASCII hexadecimal fields — human readable but a fixed 110-byte header.
//!
//! Header layout (offsets in bytes):
//! ```text
//!   0..5    magic      "070701"
//!   6..13   ino        inode number
//!   14..21  mode       file mode (permissions + type)
//!   22..29  uid        user ID
//!   30..37  gid        group ID
//!   38..45  nlink      number of links
//!   46..53  mtime      modification time
//!   54..61  filesize   file data length
//!   62..69  devmajor   major device number
//!   70..77  devminor   minor device number
//!   78..85  rdevmajor  special file major
//!   86..93  rdevminor  special file minor
//!   94..101 namesize   filename length (including NUL)
//!   102..109 check     CRC (0 in newc)
//! ```

use crate::bus::SystemBus;
use crate::constants::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpioEntry {
    pub name: String,
    pub data: Vec<u8>,
    pub mode: u32,
}

/// Parse a cpio `newc` archive and return its entries.
pub fn parse_cpio(data: &[u8]) -> Result<Vec<CpioEntry>, &'static str> {
    let mut entries = Vec::new();
    let mut offset = 0usize;

    while offset + CPIO_HEADER_SIZE <= data.len() {
        // Verify magic
        let magic = std::str::from_utf8(&data[offset..offset + 6])
            .map_err(|_| "invalid cpio magic")?;
        if magic != CPIO_NEWC_MAGIC && magic != "070702" {
            return Err("bad cpio magic (expected 070701 or 070702)");
        }

        let read_hex = |o: usize, n: usize| -> Result<u32, &'static str> {
            let s = std::str::from_utf8(&data[o..o + n]).map_err(|_| "invalid hex")?;
            u32::from_str_radix(s, 16).map_err(|_| "bad hex digit")
        };

        let _ino  = read_hex(offset + 6, 8)?;
        let mode  = read_hex(offset + 14, 8)?;
        let _uid  = read_hex(offset + 22, 8)?;
        let _gid  = read_hex(offset + 30, 8)?;
        let _nlink = read_hex(offset + 38, 8)?;
        let _mtime = read_hex(offset + 46, 8)?;
        let filesize = read_hex(offset + 54, 8)? as usize;
        let _devmajor  = read_hex(offset + 62, 8)?;
        let _devminor  = read_hex(offset + 70, 8)?;
        let _rdevmajor = read_hex(offset + 78, 8)?;
        let _rdevminor = read_hex(offset + 86, 8)?;
        let namesize   = read_hex(offset + 94, 8)? as usize;
        let _check     = read_hex(offset + 102, 8)?;

        offset += CPIO_HEADER_SIZE;

        if offset + namesize > data.len() {
            return Err("cpio filename truncated");
        }
        // Strip the trailing NUL terminator
        let name_bytes = &data[offset..offset + namesize - 1];
        let name = String::from_utf8_lossy(name_bytes).into_owned();
        offset = round_up_to_4(offset + namesize);

        // End of archive
        if name == CPIO_TRAILER_NAME {
            break;
        }

        if offset + filesize > data.len() {
            return Err("cpio file data truncated");
        }
        let file_data = data[offset..offset + filesize].to_vec();
        offset = round_up_to_4(offset + filesize);

        entries.push(CpioEntry { name, data: file_data, mode });
    }

    Ok(entries)
}

/// Build a cpio `newc` archive from a list of (name, data, mode) tuples.
pub fn build_cpio(entries: &[(String, Vec<u8>, u32)]) -> Vec<u8> {
    let mut out = Vec::new();
    for (name, data, mode) in entries {
        push_header(&mut out, name, data.len() as u32, *mode);
        out.extend_from_slice(name.as_bytes());
        out.push(0); // NUL terminator
        pad_to_4(&mut out);
        out.extend_from_slice(data);
        pad_to_4(&mut out);
    }
    // Trailer entry marks the end of the archive
    push_header(&mut out, CPIO_TRAILER_NAME, 0, 0);
    out.extend_from_slice(CPIO_TRAILER_NAME.as_bytes());
    out.push(0);
    pad_to_4(&mut out);
    out
}

fn push_header(out: &mut Vec<u8>, name: &str, filesize: u32, mode: u32) {
    let namesize = name.len() + 1; // +1 for NUL terminator
    out.extend_from_slice(CPIO_NEWC_MAGIC.as_bytes());
    out.extend_from_slice(format!("{:08x}", 0).as_bytes());       // ino
    out.extend_from_slice(format!("{:08x}", mode).as_bytes());    // mode
    out.extend_from_slice(format!("{:08x}", 0).as_bytes());       // uid
    out.extend_from_slice(format!("{:08x}", 0).as_bytes());       // gid
    out.extend_from_slice(format!("{:08x}", 1).as_bytes());       // nlink
    out.extend_from_slice(format!("{:08x}", 0).as_bytes());       // mtime
    out.extend_from_slice(format!("{:08x}", filesize).as_bytes()); // filesize
    out.extend_from_slice(format!("{:08x}", 0).as_bytes());       // devmajor
    out.extend_from_slice(format!("{:08x}", 0).as_bytes());       // devminor
    out.extend_from_slice(format!("{:08x}", 0).as_bytes());       // rdevmajor
    out.extend_from_slice(format!("{:08x}", 0).as_bytes());       // rdevminor
    out.extend_from_slice(format!("{:08x}", namesize).as_bytes()); // namesize
    out.extend_from_slice(format!("{:08x}", 0).as_bytes());       // check
}

fn pad_to_4(v: &mut Vec<u8>) {
    while v.len() % 4 != 0 {
        v.push(0);
    }
}

fn round_up_to_4(n: usize) -> usize {
    (n + 3) & !3
}

/// Load a cpio archive into emulator memory at `addr`.
pub fn load_initrd(bus: &mut SystemBus, addr: u64, data: &[u8]) {
    for (i, &byte) in data.iter().enumerate() {
        bus.write(addr + i as u64, 1, byte as u64);
    }
}

#[cfg(test)]
mod tests;
