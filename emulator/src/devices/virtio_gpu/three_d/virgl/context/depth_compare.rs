#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::devices::virtio_gpu) enum DepthCompare {
    Never = 0,
    Less = 1,
    Equal = 2,
    LessEqual = 3,
    Greater = 4,
    NotEqual = 5,
    GreaterEqual = 6,
    Always = 7,
}

impl DepthCompare {
    pub const fn from_wire(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Never), 1 => Some(Self::Less), 2 => Some(Self::Equal),
            3 => Some(Self::LessEqual), 4 => Some(Self::Greater), 5 => Some(Self::NotEqual),
            6 => Some(Self::GreaterEqual), 7 => Some(Self::Always), _ => None,
        }
    }

    pub const fn wire(self) -> u32 { self as u32 }

    pub const fn passes(self, incoming: f32, stored: f32) -> bool {
        match self {
            Self::Never => false, Self::Less => incoming < stored, Self::Equal => incoming == stored,
            Self::LessEqual => incoming <= stored, Self::Greater => incoming > stored,
            Self::NotEqual => incoming != stored, Self::GreaterEqual => incoming >= stored, Self::Always => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DepthCompare::*;
    use super::*;

    #[test]
    fn standard_depth_functions_preserve_their_ordering() {
        assert_eq!(DepthCompare::from_wire(8), None);
        assert!(!Never.passes(0.5, 0.5)); assert!(Less.passes(0.25, 0.5));
        assert!(Equal.passes(0.5, 0.5)); assert!(LessEqual.passes(0.5, 0.5));
        assert!(Greater.passes(0.75, 0.5)); assert!(NotEqual.passes(0.25, 0.5));
        assert!(GreaterEqual.passes(0.5, 0.5)); assert!(Always.passes(0.25, 0.5));
    }
}
