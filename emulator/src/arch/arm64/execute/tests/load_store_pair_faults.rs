use super::*;

#[test]
fn pair_store_faults_before_first_write_when_second_page_unmapped() {
    let (mut cpu, mut bus) = setup();
    let first_pa = RAM_BASE + 0x0100_0000;
    let second_pa = RAM_BASE + 0x0200_0000;
    let l3 = RAM_BASE + 2 * PAGE_SIZE;
    let va = 0x1ff8;

    map_two_user_pages(&mut cpu, &mut bus, 0x1000, first_pa, second_pa);
    bus.mem.write(l3 + 2 * 8, 8, 0);
    cpu.regs.set_x(0, 0x1122_3344_5566_7788);
    cpu.regs.set_x(1, 0x99aa_bbcc_ddee_ff00);
    cpu.regs.set_x(2, va);

    let instr = Instr {
        op: Opcode::Stp,
        rd: 0,
        rn: 2,
        rm: 1,
        sf: true,
        ..Instr::nop()
    };

    assert_eq!(execute(&mut cpu, &mut bus, instr), Err("translation fault"));
    assert_eq!(bus.mem.read(first_pa + 0xff8, 8), Some(0));
}
