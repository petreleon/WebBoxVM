use super::*;

#[allow(unused_variables)]
pub(super) fn decode(raw: u32, fields: FpFields) -> DecodeStep {
    let ftype = fields.ftype;
    let rd = fields.rd;
    let rn = fields.rn;
    let rm = fields.rm;
    let size = fields.size;

    if (raw & 0xFFFF_FC00) == 0x1EE1_4000 || (raw & 0xFFBF_FC00) == 0x1E21_4000 {
        return DecodeStep::Hit(fp_instr(Opcode::FpNeg, rd, rn, 0, 0, size));
    }
    if (raw & 0xFFFF_FC00) == 0x1EE0_C000 || (raw & 0xFFBF_FC00) == 0x1E20_C000 {
        return DecodeStep::Hit(fp_instr(Opcode::FpAbs, rd, rn, 0, 0, size));
    }
    if (raw & 0xFFFF_FC00) == 0x1EE1_C000 || (raw & 0xFFBF_FC00) == 0x1E21_C000 {
        return DecodeStep::Hit(fp_instr(Opcode::FpSqrt, rd, rn, 0, 0, size));
    }
    if (raw & 0xFF3F_FC00) == 0x1E25_4000 {
        return DecodeStep::Hit(fp_instr(Opcode::FpFrintm, rd, rn, 0, 0, size));
    }
    if (raw & 0xFF3F_FC00) == 0x1E24_4000 {
        return DecodeStep::Hit(fp_instr(Opcode::FpFrintn, rd, rn, 0, 0, size));
    }
    if (raw & 0xFF3F_FC00) == 0x1E26_4000 {
        return DecodeStep::Hit(fp_instr(Opcode::FpFrinta, rd, rn, 0, 0, size));
    }
    if (raw & 0xFF3F_FC00) == 0x1E27_4000 {
        return DecodeStep::Hit(fp_instr(Opcode::FpFrintx, rd, rn, 0, 0, size));
    }
    if (raw & 0xFF3F_FC00) == 0x1E24_C000 {
        return DecodeStep::Hit(fp_instr(Opcode::FpFrintp, rd, rn, 0, 0, size));
    }
    if (raw & 0xFF3F_FC00) == 0x1E25_C000 {
        return DecodeStep::Hit(fp_instr(Opcode::FpFrintz, rd, rn, 0, 0, size));
    }
    if (raw & 0xFF3F_FC00) == 0x1E27_C000 {
        return DecodeStep::Hit(fp_instr(Opcode::FpFrinti, rd, rn, 0, 0, size));
    }
    if (raw & 0xFF20_1C00) == 0x1E20_1000 {
        return DecodeStep::Hit(fp_instr(
            Opcode::FpMovImm,
            rd,
            0,
            0,
            ((raw >> 13) & 0xFF) as u64,
            size,
        ));
    }
    DecodeStep::Miss
}
