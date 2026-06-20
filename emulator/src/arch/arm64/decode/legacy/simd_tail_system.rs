use super::*;

pub(super) fn decode(raw: u32) -> DecodeStep {
    if (raw >> 16) == 0 {
        return DecodeStep::from_option(system::decode_udf(raw));
    }
    if (raw & 0xFFFF_FC00) == 0x4F00_0400 {
        return DecodeStep::Hit(Instr {
            op: Opcode::SimdMovi,
            rd: (raw & 0x1F) as u8,
            rn: 0,
            rm: 0,
            imm: 0,
            sf: true,
            cond: 0,
            size: 16,
        });
    }
    if let Some(instr) = decode_simd_ld_structure_multi(raw) {
        return DecodeStep::Hit(instr);
    }
    if let Some(instr) = decode_simd_st_structure_multi(raw) {
        return DecodeStep::Hit(instr);
    }
    if let Some(instr) = decode_simd_st4_single_lane(raw) {
        return DecodeStep::Hit(instr);
    }
    if let Some(instr) = decode_simd_st1_multi(raw) {
        return DecodeStep::Hit(instr);
    }

    let bits28_24 = (raw >> 24) & 0x1F;

    // MRS/MSR decoding
    let top12 = (raw >> 20) & 0xFFF;
    if top12 == 0xD53 {
        let rd = (raw & 0x1F) as u8;
        let sysreg_id = ((raw >> 5) & 0x7FFF) as u16;
        return DecodeStep::Hit(Instr {
            op: Opcode::Mrs,
            rd,
            rn: 0,
            rm: 0,
            imm: sysreg_id as u64,
            sf: true,
            cond: 0,
            size: 0,
        });
    }
    if top12 == 0xD51 {
        let rd = (raw & 0x1F) as u8;
        let sysreg_id = ((raw >> 5) & 0x7FFF) as u16;
        return DecodeStep::Hit(Instr {
            op: Opcode::Msr,
            rd,
            rn: 0,
            rm: 0,
            imm: sysreg_id as u64,
            sf: true,
            cond: 0,
            size: 0,
        });
    }
    if (raw & 0xFFE0001F) == 0xD4000001 {
        let imm16 = ((raw >> 5) & 0xFFFF) as u64;
        return DecodeStep::Hit(Instr {
            size: 0,
            op: Opcode::Svc,
            rd: 0,
            rn: 0,
            rm: 0,
            imm: imm16,
            sf: true,
            cond: 0,
        });
    }
    if (raw & 0xFFE0001F) == 0xD4200000 {
        let imm16 = ((raw >> 5) & 0xFFFF) as u64;
        return DecodeStep::Hit(Instr {
            size: 0,
            op: Opcode::Brk,
            rd: 0,
            rn: 0,
            rm: 0,
            imm: imm16,
            sf: true,
            cond: 0,
        });
    }
    if (raw >> 24) == 0xD5 {
        if let Some(op) = system_barriers::opcode(raw) {
            return DecodeStep::from_option(system::decode_barrier(op));
        }
        if (raw & 0xFFFF_FFE0) == 0xD503_1000 {
            return DecodeStep::from_option(system::decode_wait_timeout(raw, Opcode::Wfe));
        }
        if (raw & 0xFFFF_FFE0) == 0xD503_1020 {
            return DecodeStep::from_option(system::decode_wait_timeout(raw, Opcode::Wfi));
        }
        match raw {
            0xD503_20DF | 0xD503_30FF => {
                return DecodeStep::from_option(system::decode_barrier(Opcode::NopBarrier));
            }
            0xD503_203F => return DecodeStep::from_option(system::decode_yield()),
            0xD503_205F => return DecodeStep::from_option(system::decode_wfe()),
            0xD503_207F => return DecodeStep::from_option(system::decode_wfi()),
            0xD503_305F => return DecodeStep::from_option(system::decode_clrex()),
            _ => {}
        }
        if is_cache_maintenance(raw) {
            return DecodeStep::from_option(system::decode_cache_maintenance(raw));
        }
        let op0 = (raw >> 19) & 0x3;
        let l = (raw >> 21) & 1;
        let crn = (raw >> 12) & 0xF;
        if l == 0 && op0 == 1 && crn == 8 {
            return DecodeStep::from_option(system::decode_tlbi(raw));
        }
        if (raw & 0xFFFF_FFE0) == 0xD50B_7420 {
            return DecodeStep::from_option(system::decode_dc_zva(raw));
        }
        if (raw & 0xFFFF_F01F) == 0xD503_401F {
            let daif_bits = ((raw >> 8) & 0xF) as u8;
            let op2 = (raw >> 5) & 0x7;
            let cond = match op2 {
                0b110 => 1, // DAIFSet
                0b111 => 2, // DAIFClr
                _ => 0,
            };
            if cond != 0 {
                let op = if cond == 1 {
                    Opcode::DaifSet
                } else {
                    Opcode::DaifClr
                };
                return DecodeStep::Hit(Instr {
                    size: 0,
                    op,
                    rd: 0,
                    rn: 0,
                    rm: 0,
                    imm: daif_bits as u64,
                    sf: true,
                    cond,
                });
            }
        }
        return DecodeStep::from_option(system::decode_nop());
    }

    if bits28_24 == 0b10000 {
        return DecodeStep::from_option(data_proc::decode_adr(raw));
    }
    DecodeStep::Miss
}

fn is_cache_maintenance(raw: u32) -> bool {
    matches!(raw & 0xFFFF_FFE0, 0xD50B_7520 | 0xD50B_7B20)
}
