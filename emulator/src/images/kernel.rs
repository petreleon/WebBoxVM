use crate::constants::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KernelImage {
    pub payload: Vec<u8>,
    pub entry: u64,
    pub image_size: u64,
    pub needs_efi_tables: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KernelHeader {
    pub text_offset: u64,
    pub image_size: u64,
    pub flags: u64,
}

pub fn parse_kernel_image(data: &[u8]) -> Result<KernelImage, &'static str> {
    let header = parse_header(data)?;
    let load_size = if header.image_size > 0 {
        header.image_size as usize
    } else {
        data.len()
    };
    let payload = data[..load_size.min(data.len())].to_vec();
    let needs_efi_tables = is_pe_image(data);
    let image_size = if needs_efi_tables {
        header.image_size.max(data.len() as u64)
    } else {
        header.image_size
    };
    let entry = if needs_efi_tables {
        parse_pe_entry(data)?
    } else {
        KERNEL_LOAD_ADDR + header.text_offset
    };

    Ok(KernelImage {
        payload,
        entry,
        image_size,
        needs_efi_tables,
    })
}

pub fn parse_header(data: &[u8]) -> Result<KernelHeader, &'static str> {
    if data.len() < 64 {
        return Err("kernel file too small (need >= 64 bytes)");
    }
    let r32 = |o: usize| u32::from_le_bytes([data[o], data[o + 1], data[o + 2], data[o + 3]]);
    let r64 = |o: usize| {
        let mut b = [0u8; 8];
        b.copy_from_slice(&data[o..o + 8]);
        u64::from_le_bytes(b)
    };
    let magic = r32(56);
    if magic != ARM64_KERNEL_MAGIC {
        return Err("bad ARM64 kernel magic (expected \"ARM\\x64\")");
    }
    Ok(KernelHeader {
        text_offset: r64(8),
        image_size: r64(16),
        flags: r64(24),
    })
}

pub fn is_pe_image(data: &[u8]) -> bool {
    data.len() > KERNEL_PE_OFFSET + 4
        && &data[KERNEL_PE_OFFSET..KERNEL_PE_OFFSET + 4] == PE_SIGNATURE.as_slice()
}

pub fn parse_pe_entry(data: &[u8]) -> Result<u64, &'static str> {
    let opt_start = KERNEL_PE_OFFSET + 24;
    if data.len() < opt_start + 20 {
        return Err("PE optional header truncated");
    }
    let entry_rva = u32::from_le_bytes([
        data[opt_start + 16],
        data[opt_start + 17],
        data[opt_start + 18],
        data[opt_start + 19],
    ]);
    Ok(KERNEL_LOAD_ADDR + entry_rva as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic_image(text_offset: u64, image_size: u64) -> Vec<u8> {
        let mut image = vec![0u8; 64];
        image[8..16].copy_from_slice(&text_offset.to_le_bytes());
        image[16..24].copy_from_slice(&image_size.to_le_bytes());
        image[56..60].copy_from_slice(&ARM64_KERNEL_MAGIC.to_le_bytes());
        image
    }

    #[test]
    fn raw_kernel_image_reports_entry_and_payload() {
        let image = synthetic_image(0x80000, 64);
        let parsed = parse_kernel_image(&image).unwrap();

        assert_eq!(parsed.entry, KERNEL_LOAD_ADDR + 0x80000);
        assert_eq!(parsed.payload, image);
        assert_eq!(parsed.image_size, 64);
        assert!(!parsed.needs_efi_tables);
    }

    #[test]
    fn pe_kernel_image_reports_efi_entry() {
        let mut image = synthetic_image(0, 0);
        image.resize(KERNEL_PE_OFFSET + 48, 0);
        image[KERNEL_PE_OFFSET..KERNEL_PE_OFFSET + 4].copy_from_slice(PE_SIGNATURE);
        image[KERNEL_PE_OFFSET + 40..KERNEL_PE_OFFSET + 44]
            .copy_from_slice(&0x1234u32.to_le_bytes());

        let parsed = parse_kernel_image(&image).unwrap();

        assert_eq!(parsed.entry, KERNEL_LOAD_ADDR + 0x1234);
        assert_eq!(parsed.image_size, image.len() as u64);
        assert!(parsed.needs_efi_tables);
    }
}
