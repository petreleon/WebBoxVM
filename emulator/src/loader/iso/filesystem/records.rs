pub(super) struct RawRecord {
    pub name: String,
    pub extent: u32,
    pub size: u32,
    pub flags: u8,
}

pub(super) fn parse_record(data: &[u8]) -> Option<RawRecord> {
    let len = *data.first()? as usize;
    if len < 34 || data.len() < len {
        return None;
    }

    let extent = u32::from_le_bytes(data.get(2..6)?.try_into().ok()?);
    let size = u32::from_le_bytes(data.get(10..14)?.try_into().ok()?);
    let flags = data[25];
    let name_len = data[32] as usize;
    let name_bytes = data.get(33..33 + name_len)?;
    Some(RawRecord {
        name: record_name(name_bytes),
        extent,
        size,
        flags,
    })
}

pub(super) fn join_path(parent: &str, name: &str) -> String {
    if parent == "/" {
        format!("/{name}")
    } else {
        format!("{parent}/{name}")
    }
}

pub(super) fn normalize_path(path: &str) -> String {
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

pub(super) fn record_name(bytes: &[u8]) -> String {
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
