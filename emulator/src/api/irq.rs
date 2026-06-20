#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IrqId(u32);

impl IrqId {
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

impl From<IrqId> for u32 {
    fn from(value: IrqId) -> Self {
        value.get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn irq_ids_are_not_plain_integers_at_the_boundary() {
        let irq = IrqId::new(33);
        assert_eq!(irq.get(), 33);
        assert_eq!(u32::from(irq), 33);
    }
}
