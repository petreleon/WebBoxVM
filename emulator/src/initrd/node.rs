use crate::constants::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpioEntry {
    pub name: String,
    pub data: Vec<u8>,
    pub mode: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CpioNode {
    pub name: String,
    pub data: Vec<u8>,
    pub mode: u32,
    pub nlink: u32,
    pub devmajor: u32,
    pub devminor: u32,
    pub rdevmajor: u32,
    pub rdevminor: u32,
}

impl CpioNode {
    pub fn file(name: impl Into<String>, data: impl Into<Vec<u8>>, mode: u32) -> Self {
        Self {
            name: name.into(),
            data: data.into(),
            mode: CPIO_MODE_FILE | (mode & 0o7777),
            nlink: 1,
            devmajor: 0,
            devminor: 0,
            rdevmajor: 0,
            rdevminor: 0,
        }
    }

    pub fn dir(name: impl Into<String>, mode: u32) -> Self {
        Self {
            name: name.into(),
            data: Vec::new(),
            mode: CPIO_MODE_DIR | (mode & 0o7777),
            nlink: 2,
            devmajor: 0,
            devminor: 0,
            rdevmajor: 0,
            rdevminor: 0,
        }
    }

    pub fn symlink(name: impl Into<String>, target: impl Into<Vec<u8>>) -> Self {
        Self {
            name: name.into(),
            data: target.into(),
            mode: CPIO_MODE_SYMLINK | 0o777,
            nlink: 1,
            devmajor: 0,
            devminor: 0,
            rdevmajor: 0,
            rdevminor: 0,
        }
    }

    pub fn char_device(name: impl Into<String>, mode: u32, rdevmajor: u32, rdevminor: u32) -> Self {
        Self {
            name: name.into(),
            data: Vec::new(),
            mode: CPIO_MODE_CHAR | (mode & 0o7777),
            nlink: 1,
            devmajor: 0,
            devminor: 0,
            rdevmajor,
            rdevminor,
        }
    }
}
