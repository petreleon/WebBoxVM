#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AccessWidth {
    Byte = 1,
    Halfword = 2,
    Word = 4,
    Doubleword = 8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidAccessWidth(pub u8);

impl AccessWidth {
    pub const fn bytes(self) -> u8 {
        self as u8
    }

    pub const fn len(self) -> usize {
        self.bytes() as usize
    }

    pub const fn mask(self) -> u64 {
        match self {
            Self::Byte => 0xff,
            Self::Halfword => 0xffff,
            Self::Word => 0xffff_ffff,
            Self::Doubleword => u64::MAX,
        }
    }
}

impl TryFrom<u8> for AccessWidth {
    type Error = InvalidAccessWidth;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Byte),
            2 => Ok(Self::Halfword),
            4 => Ok(Self::Word),
            8 => Ok(Self::Doubleword),
            other => Err(InvalidAccessWidth(other)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_only_architectural_access_widths() {
        assert_eq!(AccessWidth::try_from(1), Ok(AccessWidth::Byte));
        assert_eq!(AccessWidth::try_from(2), Ok(AccessWidth::Halfword));
        assert_eq!(AccessWidth::try_from(4), Ok(AccessWidth::Word));
        assert_eq!(AccessWidth::try_from(8), Ok(AccessWidth::Doubleword));
        assert_eq!(AccessWidth::try_from(3), Err(InvalidAccessWidth(3)));
    }

    #[test]
    fn width_reports_len_and_value_mask() {
        assert_eq!(AccessWidth::Byte.len(), 1);
        assert_eq!(AccessWidth::Halfword.mask(), 0xffff);
        assert_eq!(AccessWidth::Word.mask(), 0xffff_ffff);
        assert_eq!(AccessWidth::Doubleword.bytes(), 8);
    }
}
