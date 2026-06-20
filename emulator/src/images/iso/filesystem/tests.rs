use super::records::record_name;
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

#[test]
fn strips_version_and_trailing_dot_from_record_names() {
    assert_eq!(record_name(b"VMLINUZ.;1"), "vmlinuz");
}
