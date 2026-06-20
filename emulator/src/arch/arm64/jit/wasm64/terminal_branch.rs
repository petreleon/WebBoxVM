use super::opcodes::*;
use super::*;

const LOCAL_EXIT_PC: u32 = 4;
pub(super) const ANY_DYNAMIC_EXIT_PC: u64 = u64::MAX;

pub(super) struct TerminalBranchExits {
    pub exit_pc: u64,
    pub alternate_exit_pc: u64,
    pub dynamic: bool,
}

impl WasmExpr {
    pub(super) fn emit_terminal_branch(
        &mut self,
        instr: crate::arch::arm64::Instr,
        pc: u64,
    ) -> Option<TerminalBranchExits> {
        let fallthrough = pc.wrapping_add(4);
        let target = (pc as i64 + instr.imm as i64) as u64;
        match instr.op {
            Opcode::Br | Opcode::Ret => return Some(self.emit_reg_branch(instr.rn)),
            Opcode::Blr => {
                self.emit_read_reg(instr.rn, true);
                self.local_set(LOCAL_EXIT_PC);
                self.emit_store_const(reg_offset(30), fallthrough);
                return Some(self.emit_reg_branch_from_local());
            }
            Opcode::B => return Some(self.emit_static_branch(target)),
            Opcode::Bl => {
                self.emit_store_const(reg_offset(30), fallthrough);
                return Some(self.emit_static_branch(target));
            }
            Opcode::BCond | Opcode::Cbz | Opcode::Cbnz | Opcode::Tbz | Opcode::Tbnz => {}
            _ => return None,
        }

        self.i64_const(target);
        self.i64_const(fallthrough);
        self.emit_branch_condition(instr);
        self.op(OP_SELECT);
        self.local_set(LOCAL_EXIT_PC);
        self.emit_write_pc_with(|this| this.local_get(LOCAL_EXIT_PC));
        self.local_get(LOCAL_EXIT_PC);

        Some(TerminalBranchExits {
            exit_pc: fallthrough,
            alternate_exit_pc: target,
            dynamic: true,
        })
    }

    fn emit_reg_branch(&mut self, rn: u8) -> TerminalBranchExits {
        self.emit_read_reg(rn, true);
        self.local_set(LOCAL_EXIT_PC);
        self.emit_reg_branch_from_local()
    }

    fn emit_reg_branch_from_local(&mut self) -> TerminalBranchExits {
        self.emit_write_pc_with(|this| this.local_get(LOCAL_EXIT_PC));
        self.local_get(LOCAL_EXIT_PC);

        TerminalBranchExits {
            exit_pc: 0,
            alternate_exit_pc: ANY_DYNAMIC_EXIT_PC,
            dynamic: true,
        }
    }

    fn emit_static_branch(&mut self, target: u64) -> TerminalBranchExits {
        self.emit_store_const(JIT_STATE_PC_OFFSET, target);
        self.i64_const(target);
        TerminalBranchExits {
            exit_pc: target,
            alternate_exit_pc: target,
            dynamic: false,
        }
    }

    fn emit_branch_condition(&mut self, instr: crate::arch::arm64::Instr) {
        match instr.op {
            Opcode::BCond => self.emit_condition(instr.cond),
            Opcode::Cbz => self.emit_cbz_condition(instr, false),
            Opcode::Cbnz => self.emit_cbz_condition(instr, true),
            Opcode::Tbz => self.emit_tbz_condition(instr, false),
            Opcode::Tbnz => self.emit_tbz_condition(instr, true),
            _ => unreachable!(),
        }
    }

    fn emit_cbz_condition(&mut self, instr: crate::arch::arm64::Instr, nonzero: bool) {
        self.emit_read_reg(instr.rd, instr.sf);
        if nonzero {
            self.i64_const(0);
            self.op(OP_I64_NE);
        } else {
            self.op(OP_I64_EQZ);
        }
    }

    fn emit_tbz_condition(&mut self, instr: crate::arch::arm64::Instr, nonzero: bool) {
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
