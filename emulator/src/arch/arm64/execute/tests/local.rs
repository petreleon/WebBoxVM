use super::*;

#[test]
fn local_alu_executes_without_a_bus() {
    let mut cpu = Armv8Cpu::new();
    cpu.regs.pc = 0x1000;
    cpu.regs.set_x(1, 7);
    cpu.regs.set_x(2, 11);

    let result = try_execute_local(&mut cpu, decode(0x8B02_0020).unwrap());

    assert_eq!(result, Some(Ok(())));
    assert_eq!(cpu.regs.x(0), 18);
    assert_eq!(cpu.regs.pc, 0x1004);
    assert_eq!(cpu.sys.cycle_count, 1);
}

#[test]
fn local_branch_executes_without_a_bus() {
    let mut cpu = Armv8Cpu::new();
    cpu.regs.pc = 0x1000;

    let result = try_execute_local(&mut cpu, decode(0x1400_0002).unwrap());

    assert_eq!(result, Some(Ok(())));
    assert_eq!(cpu.regs.pc, 0x1008);
}

#[test]
fn load_store_falls_back_without_mutating_cpu() {
    let mut cpu = Armv8Cpu::new();
    cpu.regs.pc = 0x1000;
    cpu.regs.set_x(0, 0x1234);
    cpu.regs.set_x(1, 0x8000);
    let before = cpu.clone();

    let result = try_execute_local(&mut cpu, decode(0xF900_0020).unwrap());

    assert_eq!(result, None);
    assert_eq!(cpu, before);
}

#[test]
fn system_instruction_falls_back_without_mutating_cpu() {
    let mut cpu = Armv8Cpu::new();
    cpu.regs.pc = 0x1000;
    let before = cpu.clone();

    let result = try_execute_local(&mut cpu, decode(0xD53B_4220).unwrap());

    assert_eq!(result, None);
    assert_eq!(cpu, before);
}
