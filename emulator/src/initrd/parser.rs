use crate::constants::*;

use super::{CpioEntry, round_up_to_4};

const ZSTD_MAGIC: &[u8] = &[0x28, 0xb5, 0x2f, 0xfd];

#[cfg(test)]
pub(crate) fn find_cpio_entry<'a>(
    data: &'a [u8],
    target: &str,
) -> Result<Option<&'a [u8]>, &'static str> {
    find_cpio_entries(data, [target]).map(|[found]| found)
}

#[cfg(test)]
pub(crate) fn find_cpio_entries<'a, const N: usize>(
    data: &'a [u8],
    targets: [&str; N],
) -> Result<[Option<&'a [u8]>; N], &'static str> {
    let (found, compressed) = find_cpio_entries_and_zstd_tail(data, targets)?;
    if compressed.is_some() {
        return Err("cpio contains zstd-compressed data");
    }
    Ok(found)
}

pub(crate) fn find_cpio_entries_and_zstd_tail<'a, const N: usize>(
    data: &'a [u8],
    targets: [&str; N],
) -> Result<([Option<&'a [u8]>; N], Option<&'a [u8]>), &'static str> {
    let mut offset = 0usize;
    let mut found = [None; N];
    while offset + CPIO_HEADER_SIZE <= data.len() {
        let (_mode, filesize, namesize) = read_header(data, offset)?;
        offset += CPIO_HEADER_SIZE;
        if namesize == 0 {
            return Err("cpio filename has zero length");
        }
        let name_end = offset
            .checked_add(namesize)
            .ok_or("cpio filename size overflow")?;
        if name_end > data.len() || data[name_end - 1] != 0 {
            return Err("cpio filename truncated");
        }
        let name = &data[offset..name_end - 1];
        offset = round_up_to_4(name_end);
        let file_end = offset
            .checked_add(filesize)
            .ok_or("cpio file size overflow")?;
        if file_end > data.len() {
            return Err("cpio file data truncated");
        }
        if name == CPIO_TRAILER_NAME.as_bytes() {
            if filesize != 0 {
                return Err("cpio trailer contains data");
            }
            let archive_end = round_up_to_4(file_end);
            if archive_end > data.len() {
                return Err("cpio trailer truncated");
            }
            let mut next = archive_end;
            while next < data.len() && data[next] == 0 {
                next += 1;
            }
            if next == data.len() {
                return Ok((found, None));
            }
            if data[next..].starts_with(ZSTD_MAGIC) {
                return Ok((found, Some(&data[next..])));
            }
            if next
                .checked_add(CPIO_HEADER_SIZE)
                .is_none_or(|end| end > data.len())
            {
                return Err("cpio concatenated header truncated");
            }
            offset = next;
            continue;
        }
        for (index, target) in targets.iter().enumerate() {
            if name == target.as_bytes() {
                found[index] = Some(&data[offset..file_end]);
            }
        }
        offset = round_up_to_4(file_end);
    }
    Err("cpio trailer missing")
}

/// Parse a cpio `newc` archive and return its entries.
pub fn parse_cpio(data: &[u8]) -> Result<Vec<CpioEntry>, &'static str> {
    let mut entries = Vec::new();
    let mut offset = 0usize;

    while offset + CPIO_HEADER_SIZE <= data.len() {
        let (mode, filesize, namesize) = read_header(data, offset)?;

        offset += CPIO_HEADER_SIZE;

        if namesize == 0 || offset + namesize > data.len() {
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

fn read_header(data: &[u8], offset: usize) -> Result<(u32, usize, usize), &'static str> {
    validate_magic(data, offset)?;
    let mut fields = [0u32; 13];
    for (index, field) in fields.iter_mut().enumerate() {
        *field = read_hex(data, offset + 6 + index * 8, 8)?;
    }
    Ok((fields[1], fields[6] as usize, fields[11] as usize))
}

fn validate_magic(data: &[u8], offset: usize) -> Result<(), &'static str> {
    let magic = std::str::from_utf8(&data[offset..offset + 6]).map_err(|_| "invalid cpio magic")?;
    if magic == CPIO_NEWC_MAGIC || magic == "070702" {
        Ok(())
    } else {
        Err("bad cpio magic (expected 070701 or 070702)")
    }
}

fn read_hex(data: &[u8], offset: usize, len: usize) -> Result<u32, &'static str> {
    let value = std::str::from_utf8(&data[offset..offset + len]).map_err(|_| "invalid hex")?;
    u32::from_str_radix(value, 16).map_err(|_| "bad hex digit")
}
