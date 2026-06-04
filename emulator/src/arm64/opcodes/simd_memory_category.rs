use super::Opcode;

pub(super) fn is_opcode(op: Opcode) -> bool {
    matches!(
        op,
        Opcode::SimdLdp
            | Opcode::SimdStp
            | Opcode::SimdLdr
            | Opcode::SimdStr
            | Opcode::SimdMovi
            | Opcode::SimdLd1
            | Opcode::SimdLd1Multi
            | Opcode::SimdLd1Lane
            | Opcode::SimdLd1r
            | Opcode::SimdLd2
            | Opcode::SimdLd3
            | Opcode::SimdSt1Multi
            | Opcode::SimdSt1Lane
            | Opcode::SimdLd4
            | Opcode::SimdSt4Single
            | Opcode::SimdSt4
            | Opcode::SimdAese
            | Opcode::SimdAesd
            | Opcode::SimdAesmc
            | Opcode::SimdAesimc
            | Opcode::SimdPmull
            | Opcode::SimdSha1h
            | Opcode::SimdSha256Su0
            | Opcode::SimdSha512Su0
            | Opcode::SimdSm4e
            | Opcode::SimdSm3Partw1
            | Opcode::SimdEor3
            | Opcode::SimdBcax
            | Opcode::SimdRax1
            | Opcode::SimdXar
    )
}
