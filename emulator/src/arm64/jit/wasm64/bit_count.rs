use super::opcodes::*;
use super::*;
use crate::arm64::{Instr, Opcode};

const LOCAL_VALUE: u32 = 1;

impl WasmExpr {
    pub(super) fn emit_bit_count(&mut self, instr: Instr) -> bool {
        match instr.op {
            Opcode::Clz => self.emit_clz(instr),
            Opcode::Rbit => self.emit_rbit(instr),
            _ => return false,
        }
        true
    }

    fn emit_clz(&mut self, instr: Instr) {
        self.emit_write_reg_with(instr.rd, instr.sf, |this| {
            this.emit_read_reg(instr.rn, instr.sf);
            if instr.sf {
                this.op(OP_I64_CLZ);
            } else {
                this.op(OP_I32_WRAP_I64);
                this.op(OP_I32_CLZ);
                this.op(OP_I64_EXTEND_I32_U);
            }
        });
    }

    fn emit_rbit(&mut self, instr: Instr) {
        let stages = if instr.sf { RBIT64 } else { RBIT32 };
        self.emit_write_reg_with(instr.rd, instr.sf, |this| {
            this.emit_read_reg(instr.rn, instr.sf);
            this.local_set(LOCAL_VALUE);
            for stage in stages {
                this.emit_rbit_stage(*stage);
            }
            this.local_get(LOCAL_VALUE);
        });
    }

    fn emit_rbit_stage(&mut self, stage: RbitStage) {
        self.local_get(LOCAL_VALUE);
        self.i64_const(stage.shift);
        self.op(OP_I64_SHR_U);
        self.i64_const(stage.mask);
        self.op(OP_I64_AND);
        self.local_get(LOCAL_VALUE);
        self.i64_const(stage.mask);
        self.op(OP_I64_AND);
        self.i64_const(stage.shift);
        self.op(OP_I64_SHL);
        self.op(OP_I64_OR);
        self.local_set(LOCAL_VALUE);
    }
}

#[derive(Clone, Copy)]
struct RbitStage {
    shift: u64,
    mask: u64,
}

const RBIT32: &[RbitStage] = &[
    RbitStage { shift: 1, mask: 0x5555_5555 },
    RbitStage { shift: 2, mask: 0x3333_3333 },
    RbitStage { shift: 4, mask: 0x0f0f_0f0f },
    RbitStage { shift: 8, mask: 0x00ff_00ff },
    RbitStage { shift: 16, mask: 0x0000_ffff },
];

const RBIT64: &[RbitStage] = &[
    RbitStage { shift: 1, mask: 0x5555_5555_5555_5555 },
    RbitStage { shift: 2, mask: 0x3333_3333_3333_3333 },
    RbitStage { shift: 4, mask: 0x0f0f_0f0f_0f0f_0f0f },
    RbitStage { shift: 8, mask: 0x00ff_00ff_00ff_00ff },
    RbitStage { shift: 16, mask: 0x0000_ffff_0000_ffff },
    RbitStage { shift: 32, mask: 0x0000_0000_ffff_ffff },
];
