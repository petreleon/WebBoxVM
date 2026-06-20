use super::*;

#[allow(unused_variables)]
pub(super) fn decode(raw: u32, fields: FpFields) -> DecodeStep {
    let ftype = fields.ftype;
    let rd = fields.rd;
    let rn = fields.rn;
    let rm = fields.rm;
    let size = fields.size;

    if (raw & 0xFF20_0000) == 0x1F00_0000 {
        let mut instr = fp_instr(
            if ((raw >> 15) & 1) != 0 {
                Opcode::Fmsub
            } else {
                Opcode::Fmadd
            },
            rd,
            rn,
            rm,
            0,
            size,
        );
        instr.cond = ((raw >> 10) & 0x1F) as u8;
        return DecodeStep::Hit(instr);
    }
    if (raw & 0xFF20_8000) == 0x1F20_0000 {
        let mut instr = fp_instr(Opcode::Fnmadd, rd, rn, rm, 0, size);
        instr.cond = ((raw >> 10) & 0x1F) as u8;
        return DecodeStep::Hit(instr);
    }
    if (raw & 0xFF20_8000) == 0x1F20_8000 {
        let mut instr = fp_instr(Opcode::Fnmsub, rd, rn, rm, 0, size);
        instr.cond = ((raw >> 10) & 0x1F) as u8;
        return DecodeStep::Hit(instr);
    }
    if (raw & 0xFF20_FC00) == 0x1E20_0800 {
        return DecodeStep::Hit(fp_instr(Opcode::FpMul, rd, rn, rm, 0, size));
    }
    if (raw & 0xFF20_FC00) == 0x1E20_8800 {
        return DecodeStep::Hit(fp_instr(Opcode::FpFnmul, rd, rn, rm, 0, size));
    }
    if (raw & 0xFF20_FC00) == 0x1E20_2800 {
        return DecodeStep::Hit(fp_instr(Opcode::FpAdd, rd, rn, rm, 0, size));
    }
    if (raw & 0xFF20_FC00) == 0x1E20_3800 {
        return DecodeStep::Hit(fp_instr(Opcode::FpSub, rd, rn, rm, 0, size));
    }
    if (raw & 0xFF20_FC00) == 0x1E20_1800 {
        return DecodeStep::Hit(fp_instr(Opcode::FpDiv, rd, rn, rm, 0, size));
    }
    if (raw & 0xFF20_FC00) == 0x1E20_4800 {
        return DecodeStep::Hit(fp_instr(Opcode::FpMax, rd, rn, rm, 0, size));
    }
    if (raw & 0xFF20_FC00) == 0x1E20_5800 {
        return DecodeStep::Hit(fp_instr(Opcode::FpMin, rd, rn, rm, 0, size));
    }
    if (raw & 0xFF20_FC00) == 0x1E20_6800 {
        return DecodeStep::Hit(fp_instr(Opcode::FpMaxnm, rd, rn, rm, 0, size));
    }
    if (raw & 0xFF20_FC00) == 0x1E20_7800 {
        return DecodeStep::Hit(fp_instr(Opcode::FpMinnm, rd, rn, rm, 0, size));
    }
    DecodeStep::Miss
}
