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

/// The supported `pipe_depth_stencil_alpha_state` depth subset.
///
/// `wire` preserves the standard VirGL DSA low bits: bit 0 enables testing,
/// bit 1 enables writes, and bits 2 through 4 select the comparison function.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::devices::virtio_gpu) struct DepthState {
    pub compare: DepthCompare,
    pub write: bool,
}

impl DepthState {
    pub const fn from_wire(value: u32) -> Option<Self> {
        if value & 1 == 0 || value & !31 != 0 {
            return None;
        }
        match DepthCompare::from_wire(value >> 2) {
            Some(compare) => Some(Self { compare, write: value & 2 != 0 }),
            None => None,
        }
    }

    pub const fn wire(self) -> u32 {
        1 | ((self.write as u32) << 1) | (self.compare.wire() << 2)
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

    #[test]
    fn standard_depth_state_preserves_test_and_write_bits() {
        assert_eq!(DepthState::from_wire(7), Some(DepthState { compare: Less, write: true }));
        assert_eq!(DepthState::from_wire(17), Some(DepthState { compare: Greater, write: false }));
        assert_eq!(DepthState::from_wire(0), None);
        assert_eq!(DepthState::from_wire(35), None);
        assert_eq!(DepthState { compare: Equal, write: false }.wire(), 9);
    }
}
