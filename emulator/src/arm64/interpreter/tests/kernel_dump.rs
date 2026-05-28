#[cfg(test)]
mod kernel_dump_tests {
    use crate::arm64::{Armv8Cpu, decode, execute};
    use crate::arm64::mmu::translate;
    use crate::bus::SystemBus;

    #[test]
    #[ignore = "slow: dumps decompressed kernel to file"]
    fn dump_decompressed_kernel() {
        use crate::loader::kernel::{load_kernel, KERNEL_LOAD};
        use crate::efi::setup_efi_tables;
        use crate::dtb::{build_dtb, load_dtb};
        use crate::initrd::{build_cpio, load_initrd};

        let mut cpu = Armv8Cpu::new();
        let mut bus = SystemBus::new();

        let _entry = load_kernel(&mut bus, "/Users/petreleon/code/WebBoxVM/Image.gz").unwrap();

        let dtb_addr = 0x4700_0000u64;
        let (handle, st) = setup_efi_tables(&mut bus, KERNEL_LOAD, 0x024f_0000, dtb_addr);
        cpu.regs.set_x(0, handle);
        cpu.regs.set_x(1, st);
        cpu.regs.sp = 0x43F0_0000;

        bus.write(0x43EFE000, 4, 0xD65F03C0);
        cpu.regs.set_x(30, 0x43EFE000);

        // Minimal DTB (needed for boot but we're not booting the kernel)
        let dtb = build_dtb(0x4000_0000, 0x4000_0000, None, None, Some("earlycon=pl011,0x09000000"));
        load_dtb(&mut bus, dtb_addr, &dtb);

        cpu.regs.pc = KERNEL_LOAD + 0x01da7ee0;

        let mut steps = 0u64;
        let mut pages_bump = 0x4800_0000u64;
        let mut kernel_copied = false;

        for _ in 0..200_000usize {
            // Check for EFI stub completion
            if cpu.regs.pc == 0 {
                println!("EFI stub done at step {}", steps);
                break;
            }

            // EFI service traps
            if cpu.regs.pc == 0x8000_CE00 {
                // CopyMem
                let dest = cpu.regs.x(0);
                let src = cpu.regs.x(1);
                let len = cpu.regs.x(2);
                if dest == 0x48000000 {
                    println!("CopyMem kernel->final at step {}: src={:#x} dest={:#x} len={:#x}", steps, src, dest, len);
                    kernel_copied = true;
                }
                if len > 0x0400_0000 { break; }
                for i in 0..len {
                    let val = bus.mem.read(src + i, 1).unwrap_or(0);
                    bus.mem.write(dest + i, 1, val);
                }
                cpu.regs.set_x(0, 0);
                cpu.regs.pc = cpu.regs.x(30);
                steps += 1;
                
                // After copying kernel to final location, dump it
                if kernel_copied && dest == 0x48000000 {
                    let mut vmlinux = Vec::with_capacity(len as usize);
                    for i in 0..len {
                        vmlinux.push(bus.mem.read(dest + i, 1).unwrap_or(0) as u8);
                    }
                    std::fs::write("/tmp/vmlinux_decompressed", &vmlinux).unwrap();
                    println!("Dumped {} bytes to /tmp/vmlinux_decompressed", vmlinux.len());
                    break;
                }
                continue;
            }
            if cpu.regs.pc == 0x8000_D000 {
                // SetMem
                let buf = cpu.regs.x(0);
                let size = cpu.regs.x(1);
                let val = cpu.regs.x(2);
                if size > 0x0400_0000 { break; }
                for i in 0..size { bus.mem.write(buf + i, 1, val); }
                cpu.regs.set_x(0, 0);
                cpu.regs.pc = cpu.regs.x(30);
                steps += 1;
                continue;
            }
            if cpu.regs.pc == 0x8000_D200 {
                // AllocatePages
                let pages = cpu.regs.x(2);
                let ptr_memory = cpu.regs.x(3);
                let allocated = (pages_bump + 4095) & !4095;
                pages_bump = allocated + pages * 4096;
                bus.write(ptr_memory, 8, allocated);
                cpu.regs.set_x(0, 0);
                cpu.regs.pc = cpu.regs.x(30);
                steps += 1;
                continue;
            }
            if cpu.regs.pc == 0x8000_D400 {
                cpu.regs.set_x(0, 0);
                cpu.regs.pc = cpu.regs.x(30);
                steps += 1;
                continue;
            }

            let raw = match bus.mem.read(cpu.regs.pc, 4) {
                Some(v) => v as u32,
                None => {
                    println!("Memory fault at step {} PC={:#x}", steps, cpu.regs.pc);
                    break;
                }
            };
            if let Some(instr) = decode(raw) {
                if let Err(e) = execute(&mut cpu, &mut bus, instr) {
                    println!("Execute error at step {}: {:?}", steps, e);
                    break;
                }
                steps += 1;
            } else {
                println!("Unknown instruction at step {} PC={:#x} raw={:#x}", steps, cpu.regs.pc, raw);
                break;
            }
        }
        println!("Dumped after {} steps", steps);
    }
}
