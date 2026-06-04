use super::*;

#[allow(unused_variables)]
pub(super) fn decode(raw: u32, fields: FpFields) -> DecodeStep {
    let ftype = fields.ftype;
    let rd = fields.rd;
    let rn = fields.rn;
    let rm = fields.rm;
    let size = fields.size;

    if (raw & 0xFF20_0C00) == 0x1E20_0400 {
        let mut instr = fp_instr(
            if (raw & 0x10) != 0 {
                Opcode::Fccmpe
            } else {
                Opcode::Fccmp
            },
            0,
            rn,
            rm,
            (raw & 0xF) as u64,
            size,
        );
        instr.cond = ((raw >> 12) & 0xF) as u8;
        return DecodeStep::Hit(instr);
    }
    if (raw & 0xFF20_FC00) == 0x1E20_2000 && (raw & 0x7) == 0 {
        let cmp_kind = ((raw >> 3) & 0x3) as u8;
        let mut instr = fp_instr(
            if (cmp_kind & 0b10) != 0 {
                Opcode::Fcmpe
            } else {
                Opcode::Fcmp
            },
            0,
            rn,
            if (cmp_kind & 1) != 0 { 0 } else { rm },
            0,
            size,
        );
        instr.cond = cmp_kind & 1;
        return DecodeStep::Hit(instr);
    }
    if (raw & 0xFF20_0C00) == 0x1E20_0C00 {
        let mut instr = fp_instr(Opcode::Fcsel, rd, rn, rm, 0, size);
        instr.cond = ((raw >> 12) & 0xF) as u8;
        return DecodeStep::Hit(instr);
    }
    DecodeStep::Miss
}
