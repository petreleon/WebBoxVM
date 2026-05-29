You are continuing a WebBoxVM ARM64 Linux boot debugging session.
The repo is at https://github.com/petreleon/WebBoxVM — clone it and pick up.

## DO NOT STOP until you see UART output from the kernel.

The end goal: `cargo run --example wait_uart --release` prints Linux boot messages
to the PL011 UART (address 0x09000000). When you see lines like
"[    0.000000] Linux version 6.6.70", you're done. Commit and push.

## Current State (as of commit 4c897a9)

**The kernel boots with ZERO faults.** MMU, page tables, instruction decode,
exception handling all work correctly. The kernel reaches `start_kernel()`
in `__primary_switched` and begins init. No hacks — pure standard ARM64 boot:

- Kernel loaded at RAM_BASE (0x40000000), not KERNEL_LOAD_ADDR+offset
- X0 = DTB physical address, MMU off, jump to Image start
- EFI stub completely skipped (run_efi_phase is a no-op)
- All relocation patches, register injections, memory intercepts REMOVED
- Code is clean in memory.rs and machine.rs

**The kernel hits an early panic before console_init().** It ends up in
`local_cpu_stop` (a polling loop at VA 0xffff800080023128). This happens
because some init function (likely in setup_arch or early driver probe)
returns an error or hits an assertion, triggering a panic. The panic
handler masks IRQs and calls smp_send_stop, which stalls because there's
only one core.

**Key files to modify (src is emulator/src/):**
- `boot/mod.rs` — BootContext, kernel loading
- `arm64/system_regs.rs` — MSR/MRS handlers, add reads for missing sysregs
- `arm64/execute/mod.rs` — check_timer_irq, advance_pc
- `arm64/machine.rs` — main loop, GIC MSR intercepts
- `devices/gicv3.rs` — GICv3 distributor + redistributor
- `devices/pl011.rs` — PL011 UART (already working, tests pass)
- `bus.rs` — MMIO dispatch
- `dtb.rs` — Device Tree Blob builder
- `memory.rs` — physical memory (clean, no hacks)
- `constants/` — all addresses, sysreg IDs, MMU constants
- `arm64/execute/system.rs` — MSR/SVC/ERET/BRK execution

**Known-good kernel (prebuilt):**
Image.gz at repo root — Linux 6.6.70, arm64 defconfig, CONFIG_RELOCATABLE=y
(forced by defconfig, can't easily disable). Built with KALLSYMS.
Docker rebuild: `docker build -t webbox-kernel .dockerbuild && docker run --rm -v $PWD:/host webbox-kernel`

**Test commands:**
```
cargo test                          # 98 pass, 1 fail (needs kernel file)
cargo run --example wait_uart --release   # main boot test
cargo run --example boot_test --release   # shorter version, 20M steps
```

**Debug approach:**
1. Run `cargo run --example wait_uart --release 2>&1 | tee /tmp/boot.log`
2. Observe DIAG output: zero fetch_faults, zero exec_faults = success
3. PC stuck at ~0xffff800080f0f4xx or ~0xffff800080023128 = early panic
4. Find WHAT panics by adding eprintln! on BRK#0x800 (imm16=0x800) or by
   tracing initcall failures in the kernel

**Most likely root causes (try in order):**

A) GICv3 not fully operational — the kernel probes the GIC during
   `init_IRQ()`, reads GICD_TYPER, GICR_TYPER, etc. If these return
   unexpected values, the GIC driver fails and the kernel panics.
   Check `devices/gicv3.rs` for correct register values, especially:
   - GICD_TYPER (needs to report 0 or 1 SPI ranges)
   - GICR_TYPER (needs to report Last=1, processor number)
   - GICD_CTLR (should start with EnableGrp0=1, EnableGrp1=1)

B) Missing system register reads — the kernel does MRS for various ID
   registers during init. If one returns 0 or an unexpected value, the
   kernel might BUG(). Add eprintln! in `system_regs.rs` read_sys_reg
   to log every MRS and check if any return 0 unexpectedly.

C) DTB missing required nodes — the kernel expects certain devices in
   the device tree. Check `dtb.rs` build_dtb to see what's included.
   Essential: /chosen/stdout-path, /memory, /cpus, /timer, /intc (GIC).

D) Timer interrupt storm — if the forced timer ticks (100Hz) fire during
   critical sections, the kernel's IRQ handler might fault/BUG. The
   current code in execute/mod.rs check_timer_irq delivers ticks every
   100M cycles even when PSTATE.I is set (to break deadlocks). Try
   different tick rates or only deliver when the kernel has IRQs unmasked.

**Symbols lookup:**
Use `docker run --rm webbox-kernel aarch64-linux-gnu-nm vmlinux | grep <name>`
to find kernel function addresses. The kernel is loaded at 0x40000000, so
PA = 0x40000000 + (VA - 0xffff800080000000).

**When you succeed:**
`git add -A && git commit -m "boot: kernel reaches console_init — UART output working" && git push`

You must NOT stop until UART output appears. If stuck, add diagnostics, not hacks.
The code must stay clean — no memory intercepts, no register injections, no
literal pool patches. The kernel should boot via the standard ARM64 protocol
with all state set correctly at entry.
