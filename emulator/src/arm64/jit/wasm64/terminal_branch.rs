use super::opcodes::*;
use super::*;

const LOCAL_EXIT_PC: u32 = 4;

pub(super) struct TerminalBranchExits {
    pub fallthrough: u64,
    pub target: u64,
}

impl WasmExpr {
    pub(super) fn emit_terminal_branch(
        &mut self,
        instr: crate::arm64::Instr,
        pc: u64,
    ) -> Option<TerminalBranchExits> {
        if !matches!(
            instr.op,
            Opcode::BCond | Opcode::Cbz | Opcode::Cbnz | Opcode::Tbz | Opcode::Tbnz
        ) {
            return None;
        }
        let fallthrough = pc.wrapping_add(4);
        let target = (pc as i64 + instr.imm as i64) as u64;

        self.i64_const(target);
        self.i64_const(fallthrough);
        self.emit_branch_condition(instr);
        self.op(OP_SELECT);
        self.local_set(LOCAL_EXIT_PC);
        self.emit_write_pc_with(|this| this.local_get(LOCAL_EXIT_PC));
        self.local_get(LOCAL_EXIT_PC);

        Some(TerminalBranchExits {
            fallthrough,
            target,
        })
    }

    fn emit_branch_condition(&mut self, instr: crate::arm64::Instr) {
        match instr.op {
            Opcode::BCond => self.emit_condition(instr.cond),
            Opcode::Cbz => self.emit_cbz_condition(instr, false),
            Opcode::Cbnz => self.emit_cbz_condition(instr, true),
            Opcode::Tbz => self.emit_tbz_condition(instr, false),
            Opcode::Tbnz => self.emit_tbz_condition(instr, true),
            _ => unreachable!(),
        }
    }

    fn emit_cbz_condition(&mut self, instr: crate::arm64::Instr, nonzero: bool) {
        self.emit_read_reg(instr.rd, instr.sf);
        if nonzero {
            self.i64_const(0);
            self.op(OP_I64_NE);
        } else {
            self.op(OP_I64_EQZ);
        }
    }

    fn emit_tbz_condition(&mut self, instr: crate::arm64::Instr, nonzero: bool) {
        self.emit_read_reg(instr.rd, instr.sf);
        self.i64_const(instr.cond as u64);
        self.op(OP_I64_SHR_U);
        self.i64_const(1);
        self.op(OP_I64_AND);
        if nonzero {
            self.i64_const(0);
            self.op(OP_I64_NE);
        } else {
            self.op(OP_I64_EQZ);
        }
    }
}
