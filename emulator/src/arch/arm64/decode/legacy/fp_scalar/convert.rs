use super::*;

#[allow(unused_variables)]
pub(super) fn decode(raw: u32, fields: FpFields) -> DecodeStep {
    let ftype = fields.ftype;
    let rd = fields.rd;
    let rn = fields.rn;
    let rm = fields.rm;
    let size = fields.size;

    if (raw & 0x7FBF_FC00) == 0x1E22_0000 {
        let mut instr = fp_instr(Opcode::Scvtf, rd, rn, 0, 0, size);
        instr.sf = (raw >> 31) != 0;
        return DecodeStep::Hit(instr);
    }
    if (raw & 0x7FBF_0000) == 0x1E02_0000 {
        let scale = ((raw >> 10) & 0x3F) as u8;
        let Some(fbits) = 64u8.checked_sub(scale) else {
            return DecodeStep::Reject;
        };
        let mut instr = fp_instr(Opcode::Scvtf, rd, rn, 0, fbits as u64, size);
        instr.sf = (raw >> 31) != 0;
        instr.cond = 1;
        return DecodeStep::Hit(instr);
    }
    if (raw & 0x7FBF_FC00) == 0x1E23_0000 {
        let mut instr = fp_instr(Opcode::Ucvtf, rd, rn, 0, 0, size);
        instr.sf = (raw >> 31) != 0;
        return DecodeStep::Hit(instr);
    }
    if (raw & 0x7FBF_0000) == 0x1E03_0000 {
        let scale = ((raw >> 10) & 0x3F) as u8;
        let Some(fbits) = 64u8.checked_sub(scale) else {
            return DecodeStep::Reject;
        };
        let mut instr = fp_instr(Opcode::Ucvtf, rd, rn, 0, fbits as u64, size);
        instr.sf = (raw >> 31) != 0;
        instr.cond = 1;
        return DecodeStep::Hit(instr);
    }
    if (raw & 0x7FBF_0000) == 0x1E18_0000 {
        let scale = ((raw >> 10) & 0x3F) as u8;
        if (raw >> 31) == 0 && (scale & 0x20) == 0 {
            return DecodeStep::Reject;
        }
        let Some(fbits) = 64u8.checked_sub(scale) else {
            return DecodeStep::Reject;
        };
        let mut instr = fp_instr(Opcode::Fcvtzs, rd, rn, 0, fbits as u64, size);
        instr.sf = (raw >> 31) != 0;
        instr.cond = 1;
        return DecodeStep::Hit(instr);
    }
    if (raw & 0x7FBF_0000) == 0x1E19_0000 {
        let scale = ((raw >> 10) & 0x3F) as u8;
        if (raw >> 31) == 0 && (scale & 0x20) == 0 {
            return DecodeStep::Reject;
        }
        let Some(fbits) = 64u8.checked_sub(scale) else {
            return DecodeStep::Reject;
        };
        let mut instr = fp_instr(Opcode::Fcvtzu, rd, rn, 0, fbits as u64, size);
        instr.sf = (raw >> 31) != 0;
        instr.cond = 1;
        return DecodeStep::Hit(instr);
    }
    if (raw & 0x7FBF_FC00) == 0x1E20_0000 {
        let mut instr = fp_instr(Opcode::Fcvtns, rd, rn, 0, 0, size);
        instr.sf = (raw >> 31) != 0;
        return DecodeStep::Hit(instr);
    }
    if (raw & 0x7FBF_FC00) == 0x1E30_0000 {
        let mut instr = fp_instr(Opcode::Fcvtms, rd, rn, 0, 0, size);
        instr.sf = (raw >> 31) != 0;
        return DecodeStep::Hit(instr);
    }
    if (raw & 0x7FBF_FC00) == 0x1E38_0000 {
        let mut instr = fp_instr(Opcode::Fcvtzs, rd, rn, 0, 0, size);
        instr.sf = (raw >> 31) != 0;
        return DecodeStep::Hit(instr);
    }
    if (raw & 0x7FBF_FC00) == 0x1E39_0000 {
        let mut instr = fp_instr(Opcode::Fcvtzu, rd, rn, 0, 0, size);
        instr.sf = (raw >> 31) != 0;
        return DecodeStep::Hit(instr);
    }
    if (raw & 0x7FBF_FC00) == 0x1E24_0000 {
        let mut instr = fp_instr(Opcode::Fcvtas, rd, rn, 0, 0, size);
        instr.sf = (raw >> 31) != 0;
        return DecodeStep::Hit(instr);
    }
    DecodeStep::Miss
}
