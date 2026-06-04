use super::*;

#[allow(unused_variables)]
pub(super) fn decode(raw: u32, fields: FpFields) -> DecodeStep {
    let ftype = fields.ftype;
    let rd = fields.rd;
    let rn = fields.rn;
    let rm = fields.rm;
    let size = fields.size;

    if (raw & 0xFF3E_7C00) == 0x1E22_4000 {
        let dst_ftype = ((raw >> 15) & 0x3) as u8;
        if ftype == dst_ftype {
            return DecodeStep::Reject;
        }
        let Some(src_size) = fp_scalar_type_size(ftype) else {
            return DecodeStep::Reject;
        };
        let Some(dst_size) = fp_scalar_type_size(dst_ftype) else {
            return DecodeStep::Reject;
        };
        let mut instr = fp_instr(Opcode::FpFcvt, rd, rn, 0, 0, dst_size);
        instr.cond = src_size;
        return DecodeStep::Hit(instr);
    }
    DecodeStep::Miss
}
