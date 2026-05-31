use crate::constants::*;

use super::{CpioEntry, round_up_to_4};

/// Parse a cpio `newc` archive and return its entries.
pub fn parse_cpio(data: &[u8]) -> Result<Vec<CpioEntry>, &'static str> {
    let mut entries = Vec::new();
    let mut offset = 0usize;

    while offset + CPIO_HEADER_SIZE <= data.len() {
        let magic =
            std::str::from_utf8(&data[offset..offset + 6]).map_err(|_| "invalid cpio magic")?;
        if magic != CPIO_NEWC_MAGIC && magic != "070702" {
            return Err("bad cpio magic (expected 070701 or 070702)");
        }

        let read_hex = |o: usize, n: usize| -> Result<u32, &'static str> {
            let s = std::str::from_utf8(&data[o..o + n]).map_err(|_| "invalid hex")?;
            u32::from_str_radix(s, 16).map_err(|_| "bad hex digit")
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

        offset += CPIO_HEADER_SIZE;

        if offset + namesize > data.len() {
            return Err("cpio filename truncated");
        }

        let name_bytes = &data[offset..offset + namesize - 1];
        let name = String::from_utf8_lossy(name_bytes).into_owned();
        offset = round_up_to_4(offset + namesize);

        if name == CPIO_TRAILER_NAME {
            break;
        }

        if offset + filesize > data.len() {
            return Err("cpio file data truncated");
        }
        let file_data = data[offset..offset + filesize].to_vec();
        offset = round_up_to_4(offset + filesize);

        entries.push(CpioEntry {
            name,
            data: file_data,
            mode,
        });
    }

    Ok(entries)
}
