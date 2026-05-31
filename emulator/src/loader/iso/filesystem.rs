use std::collections::HashMap;

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

struct RawRecord {
    name: String,
    extent: u32,
    size: u32,
    flags: u8,
}

fn parse_record(data: &[u8]) -> Option<RawRecord> {
    let len = *data.first()? as usize;
    if len < 34 || data.len() < len {
        return None;
    }

    let extent = u32::from_le_bytes(data.get(2..6)?.try_into().ok()?);
    let size = u32::from_le_bytes(data.get(10..14)?.try_into().ok()?);
    let flags = data[25];
    let name_len = data[32] as usize;
    let name_bytes = data.get(33..33 + name_len)?;
    let name = record_name(name_bytes);

    Some(RawRecord {
        name,
        extent,
        size,
        flags,
    })
}

fn record_name(bytes: &[u8]) -> String {
    if bytes == [0] {
        return ".".to_string();
    }
    if bytes == [1] {
        return "..".to_string();
    }

    let mut name = String::from_utf8_lossy(bytes).into_owned();
    if let Some((base, _version)) = name.split_once(';') {
        name = base.to_string();
    }
    while name.ends_with('.') {
        name.pop();
    }
    name.to_ascii_lowercase()
}

fn join_path(parent: &str, name: &str) -> String {
    if parent == "/" {
        format!("/{name}")
    } else {
        format!("{parent}/{name}")
    }
}

fn normalize_path(path: &str) -> String {
    let path = path.replace('\\', "/");
    let mut normalized = String::new();
    for component in path.split('/') {
        if component.is_empty() || component == "." {
            continue;
        }
        normalized.push('/');
        normalized.push_str(&record_name(component.as_bytes()));
    }
    if normalized.is_empty() {
        "/".to_string()
    } else {
        normalized
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub fn minimal_iso_with_files(files: &[(&str, &[u8])]) -> Vec<u8> {
        let root_sector = 20usize;
        let first_file_sector = 21usize;
        let sectors = first_file_sector + files.len() + 1;
        let mut iso = vec![0u8; sectors * SECTOR_SIZE];

        let mut root_records = Vec::new();
        root_records.extend(record(".", root_sector, SECTOR_SIZE, DIR_FLAG));
        root_records.extend(record("..", root_sector, SECTOR_SIZE, DIR_FLAG));

        for (idx, (path, bytes)) in files.iter().enumerate() {
            let sector = first_file_sector + idx;
            let name = path.rsplit('/').next().unwrap();
            root_records.extend(record(name, sector, bytes.len(), 0));
            let start = sector * SECTOR_SIZE;
            iso[start..start + bytes.len()].copy_from_slice(bytes);
        }
        iso[root_sector * SECTOR_SIZE..root_sector * SECTOR_SIZE + root_records.len()]
            .copy_from_slice(&root_records);

        let pvd = PVD_SECTOR * SECTOR_SIZE;
        iso[pvd] = PVD_TYPE_PRIMARY;
        iso[pvd + 1..pvd + 6].copy_from_slice(VOLUME_DESCRIPTOR_ID);
        iso[pvd + 6] = 1;
        let root = record(".", root_sector, SECTOR_SIZE, DIR_FLAG);
        iso[pvd + ROOT_RECORD_OFFSET..pvd + ROOT_RECORD_OFFSET + root.len()].copy_from_slice(&root);

        iso
    }

    fn record(name: &str, extent: usize, size: usize, flags: u8) -> Vec<u8> {
        let name_bytes = match name {
            "." => vec![0],
            ".." => vec![1],
            _ => name.to_ascii_uppercase().into_bytes(),
        };
        let len = 33 + name_bytes.len() + usize::from(name_bytes.len() % 2 == 0);
        let mut out = vec![0u8; len];
        out[0] = len as u8;
        out[2..6].copy_from_slice(&(extent as u32).to_le_bytes());
        out[6..10].copy_from_slice(&(extent as u32).to_be_bytes());
        out[10..14].copy_from_slice(&(size as u32).to_le_bytes());
        out[14..18].copy_from_slice(&(size as u32).to_be_bytes());
        out[25] = flags;
        out[28..30].copy_from_slice(&1u16.to_le_bytes());
        out[30..32].copy_from_slice(&1u16.to_be_bytes());
        out[32] = name_bytes.len() as u8;
        out[33..33 + name_bytes.len()].copy_from_slice(&name_bytes);
        out
    }

    #[test]
    fn parses_root_files_case_insensitively() {
        let iso = minimal_iso_with_files(&[("/VMLINUZ", b"kernel")]);
        let fs = IsoFs::parse(&iso).unwrap();

        assert!(fs.exists("/vmlinuz"));
        assert_eq!(fs.read_file("/vmlinuz").unwrap(), b"kernel");
    }
}
