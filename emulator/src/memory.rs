use crate::efi;

const RAM_BASE: u64 = 0x4000_0000;
const RAM_SIZE: usize = 1_073_741_824;
const RAM_TOP: u64 = RAM_BASE + RAM_SIZE as u64;

const EFI_SIZE: usize = efi::EFI_MEM_SIZE as usize;

/// Base address for the kernel PE image (where the Debian kernel expects to run).
/// The PE has ImageBase=0 and .text at 0x10000.  We carve out a 64 MiB
/// region at 0x10000 so the EFI stub’s check_image_region succeeds.
const KERNEL_REGION_BASE: u64 = 0x1_0000;
const KERNEL_REGION_SIZE: usize = 64 * 1024 * 1024; // 64 MiB
const KERNEL_REGION_TOP: u64 = KERNEL_REGION_BASE + KERNEL_REGION_SIZE as u64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhysicalMemory {
    ram: Vec<u8>,
    kernel_region: Vec<u8>,
    efi: Vec<u8>,
}

impl PhysicalMemory {
    pub fn new() -> Self {
        Self {
            ram: vec![0u8; RAM_SIZE],
            kernel_region: vec![0u8; KERNEL_REGION_SIZE],
            efi: vec![0u8; EFI_SIZE],
        }
    }

    pub fn read(&self, addr: u64, size: u8) -> Option<u64> {
        if let Some(offset) = self.ram_offset(addr) {
            Some(match size {
                1 => self.ram[offset] as u64,
                2 => u16::from_le_bytes([self.ram[offset], self.ram[offset + 1]]) as u64,
                4 => u32::from_le_bytes([
                    self.ram[offset],
                    self.ram[offset + 1],
                    self.ram[offset + 2],
                    self.ram[offset + 3],
                ]) as u64,
                8 => u64::from_le_bytes([
                    self.ram[offset],
                    self.ram[offset + 1],
                    self.ram[offset + 2],
                    self.ram[offset + 3],
                    self.ram[offset + 4],
                    self.ram[offset + 5],
                    self.ram[offset + 6],
                    self.ram[offset + 7],
                ]),
                _ => return None,
            })
        } else if let Some(offset) = self.kernel_region_offset(addr) {
            Some(match size {
                1 => self.kernel_region[offset] as u64,
                2 => u16::from_le_bytes([self.kernel_region[offset], self.kernel_region[offset + 1]]) as u64,
                4 => u32::from_le_bytes([
                    self.kernel_region[offset],
                    self.kernel_region[offset + 1],
                    self.kernel_region[offset + 2],
                    self.kernel_region[offset + 3],
                ]) as u64,
                8 => u64::from_le_bytes([
                    self.kernel_region[offset],
                    self.kernel_region[offset + 1],
                    self.kernel_region[offset + 2],
                    self.kernel_region[offset + 3],
                    self.kernel_region[offset + 4],
                    self.kernel_region[offset + 5],
                    self.kernel_region[offset + 6],
                    self.kernel_region[offset + 7],
                ]),
                _ => return None,
            })
        } else if let Some(offset) = self.efi_offset(addr) {
            Some(match size {
                1 => self.efi[offset] as u64,
                2 => u16::from_le_bytes([self.efi[offset], self.efi[offset + 1]]) as u64,
                4 => u32::from_le_bytes([
                    self.efi[offset],
                    self.efi[offset + 1],
                    self.efi[offset + 2],
                    self.efi[offset + 3],
                ]) as u64,
                8 => u64::from_le_bytes([
                    self.efi[offset],
                    self.efi[offset + 1],
                    self.efi[offset + 2],
                    self.efi[offset + 3],
                    self.efi[offset + 4],
                    self.efi[offset + 5],
                    self.efi[offset + 6],
                    self.efi[offset + 7],
                ]),
                _ => return None,
            })
        } else {
            None
        }
    }

    pub fn write(&mut self, addr: u64, size: u8, value: u64) -> Option<()> {
        if let Some(offset) = self.ram_offset(addr) {
            match size {
                1 => self.ram[offset] = value as u8,
                2 => self.ram[offset..][..2].copy_from_slice(&(value as u16).to_le_bytes()),
                4 => self.ram[offset..][..4].copy_from_slice(&(value as u32).to_le_bytes()),
                8 => self.ram[offset..][..8].copy_from_slice(&value.to_le_bytes()),
                _ => return None,
            }
            Some(())
        } else if let Some(offset) = self.kernel_region_offset(addr) {
            match size {
                1 => self.kernel_region[offset] = value as u8,
                2 => self.kernel_region[offset..][..2].copy_from_slice(&(value as u16).to_le_bytes()),
                4 => self.kernel_region[offset..][..4].copy_from_slice(&(value as u32).to_le_bytes()),
                8 => self.kernel_region[offset..][..8].copy_from_slice(&value.to_le_bytes()),
                _ => return None,
            }
            Some(())
        } else if let Some(offset) = self.efi_offset(addr) {
            match size {
                1 => self.efi[offset] = value as u8,
                2 => self.efi[offset..][..2].copy_from_slice(&(value as u16).to_le_bytes()),
                4 => self.efi[offset..][..4].copy_from_slice(&(value as u32).to_le_bytes()),
                8 => self.efi[offset..][..8].copy_from_slice(&value.to_le_bytes()),
                _ => return None,
            }
            Some(())
        } else {
            None
        }
    }

    pub fn read_bytes(&self, addr: u64, dst: &mut [u8]) -> Option<()> {
        if let Some(offset) = self.ram_offset(addr) {
            dst.copy_from_slice(&self.ram[offset..offset + dst.len()]);
            Some(())
        } else if let Some(offset) = self.kernel_region_offset(addr) {
            dst.copy_from_slice(&self.kernel_region[offset..offset + dst.len()]);
            Some(())
        } else if let Some(offset) = self.efi_offset(addr) {
            dst.copy_from_slice(&self.efi[offset..offset + dst.len()]);
            Some(())
        } else {
            None
        }
    }

    fn ram_offset(&self, addr: u64) -> Option<usize> {
        if addr >= RAM_BASE && addr < RAM_TOP {
            Some((addr - RAM_BASE) as usize)
        } else {
            None
        }
    }

    fn kernel_region_offset(&self, addr: u64) -> Option<usize> {
        if addr >= KERNEL_REGION_BASE && addr < KERNEL_REGION_TOP {
            Some((addr - KERNEL_REGION_BASE) as usize)
        } else {
            None
        }
    }

    fn efi_offset(&self, addr: u64) -> Option<usize> {
        if addr >= efi::EFI_MEM_BASE && addr < efi::EFI_MEM_BASE + efi::EFI_MEM_SIZE {
            Some((addr - efi::EFI_MEM_BASE) as usize)
        } else {
            None
        }
    }
}

impl Default for PhysicalMemory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u64_roundtrip() {
        let mut m = PhysicalMemory::new();
        assert!(m.write(0x4000_0000, 8, 0xCAFE0000_DEADBEEF).is_some());
        assert_eq!(m.read(0x4000_0000, 8), Some(0xCAFE0000_DEADBEEF));
    }

    #[test]
    fn kernel_region_roundtrip() {
        let mut m = PhysicalMemory::new();
        assert!(m.write(0x1_0000, 8, 0x1234_5678_9ABC_DEFF).is_some());
        assert_eq!(m.read(0x1_0000, 8), Some(0x1234_5678_9ABC_DEFF));
    }

    #[test]
    fn u8_roundtrip() {
        let mut m = PhysicalMemory::new();
        assert!(m.write(0x4000_0100, 1, 0x42).is_some());
        assert_eq!(m.read(0x4000_0100, 1), Some(0x42));
    }

    #[test]
    fn unmapped_fails() {
        let m = PhysicalMemory::new();
        assert_eq!(m.read(0x0000_0000, 4), None);
    }
}
