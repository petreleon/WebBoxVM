use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if raw == 0xD500_401F {
        return DecodeStep::from_option(system::decode_cfinv());
    }
    if (raw & 0xFFE0_7C10) == 0xBA00_0400 {
        return DecodeStep::from_option(system::decode_rmif(raw));
    }
    if (raw & 0xFFFF_FC1F) == 0x3A00_080D {
        return DecodeStep::from_option(system::decode_setf(raw, Opcode::Setf8));
    }
    if (raw & 0xFFFF_FC1F) == 0x3A00_480D {
        return DecodeStep::from_option(system::decode_setf(raw, Opcode::Setf16));
    }
    DecodeStep::Miss
}
