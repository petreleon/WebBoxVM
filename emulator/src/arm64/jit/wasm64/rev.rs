use super::opcodes::*;
use super::*;

impl WasmExpr {
    pub(super) fn emit_rev(&mut self, instr: crate::arm64::Instr) {
        let lanes = if instr.sf { REV64 } else { REV32 };
        self.emit_write_reg_with(instr.rd, instr.sf, |this| {
            for (index, lane) in lanes.iter().enumerate() {
                this.emit_rev_lane(instr.rn, instr.sf, *lane);
                if index != 0 {
                    this.op(OP_I64_OR);
                }
            }
        });
    }

    fn emit_rev_lane(&mut self, reg: u8, sf: bool, lane: RevLane) {
        self.emit_read_reg(reg, sf);
        self.i64_const(lane.mask);
        self.op(OP_I64_AND);
        if lane.shift == 0 {
            return;
        }
        self.i64_const(lane.shift.unsigned_abs() as u64);
        self.op(if lane.shift > 0 {
            OP_I64_SHL
        } else {
            OP_I64_SHR_U
        });
    }
}

#[derive(Clone, Copy)]
struct RevLane {
    mask: u64,
    shift: i32,
}

const REV32: &[RevLane] = &[
    RevLane {
        mask: 0x0000_00ff,
        shift: 24,
    },
    RevLane {
        mask: 0x0000_ff00,
        shift: 8,
    },
    RevLane {
        mask: 0x00ff_0000,
        shift: -8,
    },
    RevLane {
        mask: 0xff00_0000,
        shift: -24,
    },
];

const REV64: &[RevLane] = &[
    RevLane {
        mask: 0xff,
        shift: 56,
    },
    RevLane {
        mask: 0xff00,
        shift: 40,
    },
    RevLane {
        mask: 0xff0000,
        shift: 24,
    },
    RevLane {
        mask: 0xff000000,
        shift: 8,
    },
    RevLane {
        mask: 0xff00000000,
        shift: -8,
    },
    RevLane {
        mask: 0xff0000000000,
        shift: -24,
    },
    RevLane {
        mask: 0xff000000000000,
        shift: -40,
    },
    RevLane {
        mask: 0xff00000000000000,
        shift: -56,
    },
];
