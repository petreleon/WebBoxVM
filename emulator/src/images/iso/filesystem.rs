use std::collections::HashMap;

mod records;

use records::{join_path, normalize_path, parse_record};

const SECTOR_SIZE: usize = 2048;
const PVD_SECTOR: usize = 16;
const PVD_TYPE_PRIMARY: u8 = 1;
const VOLUME_DESCRIPTOR_ID: &[u8; 5] = b"CD001";
const ROOT_RECORD_OFFSET: usize = 156;
const DIR_FLAG: u8 = 0x02;
const MAX_DIR_DEPTH: usize = 32;

#[derive(Debug, Clone)]
struct IsoEntry {
    path: String,
    extent: u32,
    size: u32,
    flags: u8,
}

impl IsoEntry {
    fn is_dir(&self) -> bool {
        self.flags & DIR_FLAG != 0
    }
}

pub struct IsoFs<'a> {
    data: &'a [u8],
    entries: Vec<IsoEntry>,
    index: HashMap<String, usize>,
}

impl<'a> IsoFs<'a> {
    pub fn parse(data: &'a [u8]) -> Result<Self, String> {
        let pvd_offset = PVD_SECTOR * SECTOR_SIZE;
        let pvd = data
            .get(pvd_offset..pvd_offset + SECTOR_SIZE)
            .ok_or_else(|| "ISO image is too small for a primary volume descriptor".to_string())?;
        if pvd[0] != PVD_TYPE_PRIMARY || &pvd[1..6] != VOLUME_DESCRIPTOR_ID {
            return Err("ISO primary volume descriptor not found at sector 16".to_string());
        }

        let root = parse_record(&pvd[ROOT_RECORD_OFFSET..])
            .ok_or_else(|| "ISO root directory record is invalid".to_string())?;
        let mut fs = Self {
            data,
            entries: Vec::new(),
            index: HashMap::new(),
        };
        fs.read_directory("/", root.extent, root.size, 0)?;
        Ok(fs)
    }

    pub fn exists(&self, path: &str) -> bool {
        self.index.contains_key(&normalize_path(path))
    }

    pub fn read_file(&self, path: &str) -> Result<&'a [u8], String> {
        let key = normalize_path(path);
        let entry = self
            .index
            .get(&key)
            .and_then(|idx| self.entries.get(*idx))
            .ok_or_else(|| format!("ISO file not found: {path}"))?;
        if entry.is_dir() {
            return Err(format!("ISO path is a directory: {}", entry.path));
        }
        self.entry_bytes(entry)
    }

    pub fn read_text_file(&self, path: &str) -> Result<String, String> {
        let bytes = self.read_file(path)?;
        Ok(String::from_utf8_lossy(bytes).into_owned())
    }

    fn read_directory(
        &mut self,
        parent: &str,
        extent: u32,
        size: u32,
        depth: usize,
    ) -> Result<(), String> {
        if depth > MAX_DIR_DEPTH {
            return Err("ISO directory tree is too deep".to_string());
        }

        let dir_bytes = self.extent_bytes(extent, size)?;
        let mut offset = 0usize;
        while offset < dir_bytes.len() {
            let record_len = dir_bytes[offset] as usize;
            if record_len == 0 {
                offset = ((offset / SECTOR_SIZE) + 1) * SECTOR_SIZE;
                continue;
            }

            let Some(record) = parse_record(&dir_bytes[offset..]) else {
                break;
            };
            offset += record_len;

            if record.name == "." || record.name == ".." {
                continue;
            }

            let path = join_path(parent, &record.name);
            let entry = IsoEntry {
                path: path.clone(),
                extent: record.extent,
                size: record.size,
                flags: record.flags,
            };
            let entry_index = self.entries.len();
            self.index.insert(normalize_path(&path), entry_index);
            self.entries.push(entry);

            if record.flags & DIR_FLAG != 0 {
                self.read_directory(&path, record.extent, record.size, depth + 1)?;
            }
        }

        Ok(())
    }

    fn entry_bytes(&self, entry: &IsoEntry) -> Result<&'a [u8], String> {
        self.extent_bytes(entry.extent, entry.size)
    }

    fn extent_bytes(&self, extent: u32, size: u32) -> Result<&'a [u8], String> {
        let start = extent as usize * SECTOR_SIZE;
        let end = start
            .checked_add(size as usize)
            .ok_or_else(|| "ISO extent overflows host usize".to_string())?;
        self.data
            .get(start..end)
            .ok_or_else(|| "ISO extent points outside image".to_string())
    }
}

#[cfg(test)]
pub(crate) mod tests;
