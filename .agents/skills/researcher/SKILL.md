---
name: researcher
description: Research ARM64 architecture, virtualization techniques, and browser technologies for WebBoxVM. Find relevant papers, specs, and prior art. Design benchmarks and evaluate novel approaches. Use when exploring how to implement MMU, JIT, VirtIO, WebGPU, or any uncharted subsystem.
metadata:
  type: project
---

# Researcher — WebBoxVM

Investigate architecture details, find prior art, and design experiments. This skill is for when you need to know "how does this actually work?" or "what is the state of the art?"

## When to invoke

- Implementing a new subsystem you have not built before (MMU, TLB, JIT, VirtIO)
- Debugging an instruction that behaves unexpectedly
- Choosing between two implementation strategies
- Benchmarking or profiling
- Writing documentation that references specifications
- Proposing a novel feature and need to justify it

## Primary sources

### ARM64 architecture

| Document | Purpose | URL |
|---|---|---|
| ARM ARMv8-A Architecture Reference Manual | Instruction semantics, system registers, memory model | developer.arm.com/documentation/ddi0487/latest |
| ARM64 ABI for Linux | Calling convention, syscall numbers | github.com/ARM-software/abi-aa |
| Linux ARM64 Boot Protocol | How the kernel expects to be booted | kernel.org/doc/Documentation/arm64/booting.rst |

### Virtualization & Emulation

| Topic | Key references |
|---|---|
| QEMU internals | qemu.org/docs/master/devel/index.html |
| KVM ARM64 | kernel.org/doc/Documentation/virtual/kvm/api.txt |
| Firecracker (microVM) | github.com/firecracker-microvm/firecracker |
| Bellard's TinyEMU | bellard.org/tinyemu |
| Unicorn engine | unicorn-engine.org |
| wasmtime (Wasm runtime) | bytecodealliance.org |

### Browser / WebAssembly / WebGPU

| Topic | Key references |
|---|---|
| WebAssembly spec | webassembly.github.io/spec/core |
| WebGPU spec | w3.org/TR/webgpu |
| Wasm memory model | linear memory, shared memory, atomics |
| Emscripten | emscripten.org/docs |
| WASI (system interface) | github.com/WebAssembly/WASI |

## Research workflow

### 1. State the question

Write a clear, answerable question:

> ❌ "How do I do MMU?"
> ✅ "On ARM64, what is the exact algorithm for a 3-level page table walk with 39-bit VAs, and where in the Linux kernel does it validate the page table entries?"

### 2. Search strategy

1. **Primary spec**: ARM ARMv8-A, section on memory management (D4-D5)
2. **Linux source**: `arch/arm64/mm/`, `arch/arm64/include/asm/pgtable.h`
3. **QEMU reference**: `target/arm/ptw.c` for a working C implementation
4. **Academic papers**: search `site:arxiv.org ARM64 MMU virtualization` or `site:usenix.org`
5. **Prior emulator code**: search GitHub for `ARM64 MMU emulator Rust`

### 3. Synthesize findings

Produce a short research note (1-2 paragraphs + code snippets) with:
- What you learned
- Where you learned it (citation)
- How it applies to WebBoxVM
- Open questions remaining

Save these notes in `research/<topic>.md` if they will be referenced by multiple tasks.

## Benchmarking methodology

When adding a new feature, measure before and after:

```bash
# Baseline: how many instructions per second can the interpreter execute?
cargo test real_kernel_runs_past_prologue_trace -- --ignored --nocapture
# Note the "executed N instructions" output and wall-clock time

# After change: re-run and compare
```

### Metrics to track

| Metric | How to measure | Target |
|---|---|---|
| Instructions per second | Count `steps` / wall time | > 1 MIPS |
| TLB hit rate | Add counters to TLB code | > 95% |
| Page table walk depth | Trace average walk depth | ≤ 3 |
| Memory overhead | `valgrind --tool=massif` or custom allocator | Minimal |
| Wasm bundle size | `wasm-pack build --release` | < 5 MB |

### Benchmark harness

Add a `benches/` directory when you have a comparative measurement to make:

```rust
// benches/interpreter.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_add_sequence(c: &mut Criterion) {
    c.bench_function("add_sequence_1000", |b| {
        b.iter(|| {
            // Setup CPU + bus, run 1000 ADD instructions
            black_box(run(&mut cpu, &mut bus, entry, 1000))
        })
    });
}

criterion_group!(benches, benchmark_add_sequence);
criterion_main!(benches);
```

Add to `Cargo.toml`:
```toml
[[bench]]
name = "interpreter"
harness = false

[dev-dependencies]
criterion = "0.5"
```

## Experimental design

When evaluating two approaches (e.g., direct-mapped vs. set-associative TLB):

1. **Hypothesis**: "A 2-way set-associative TLB will have a higher hit rate than direct-mapped for real kernel traces."
2. **Variable**: TLB organization (direct-mapped vs. 2-way SA)
3. **Constant**: TLB size (2048 entries), workload (real kernel boot trace)
4. **Measurement**: Hit rate, miss latency, total boot time
5. **Significance**: Run 3+ times, report mean + stddev
6. **Conclusion**: Pick the simpler one unless the improvement is > 10%

## Literature review shortcuts

| If you need to know about... | Search terms |
|---|---|
| ARM64 page tables | `ARM64 3-level page table walk VMSA` |
| Wasm virtualization | `WebAssembly virtual machine in browser` |
| JIT for ARM64 | `ARM64 baseline JIT compiler design` |
| VirtIO in browsers | `VirtIO over WebTransport` |
| GPU virtualization | `paravirtualized GPU display WebGPU` |
| Persistent storage in Wasm | `OPFS origin private file system` |

## Citation format

When referencing a source in code comments or docs, use:

```rust
/// See ARM ARMv8-A DDI 0487G.b, section D4.3.2, "Translation table walk".
/// Also: linux/arch/arm64/mm/fault.c, function __do_page_fault().
fn page_table_walk(...) { ... }
```

This makes the reasoning traceable and saves future-you from re-researching.

## Research notes template

When you spend > 15 minutes researching something, write it down:

```markdown
# Research: ARM64 Page Table Walk

## Question
What is the exact algorithm for a 3-level page table walk with 39-bit VAs?

## Findings
- ARM ARMv8-A D4.3: VMSAv8-64 uses a 3-level lookup for 39-bit VAs
- Level 0 is not used; levels 1, 2, 3 are indexed by bits [38:30], [29:21], [20:12]
- Each level is a 512-entry table (9 bits), 8 bytes per entry = 4KB page
- Final page size is 4KB (bits [11:0] are the offset)

## Application to WebBoxVM
- `emulator/src/memory.rs` needs a `page_table_walk(vpn)` function
- TLB key should be VPN [38:12], value is PTE + PFN
- `SCTLR_EL1.M` bit enables the MMU; when clear, VA == PA

## Open questions
- How does the Linux kernel set up the initial page tables? (check `start_kernel`)
- What is the exact behavior of `TCR_EL1` for 39-bit vs 48-bit?

## Sources
- ARM ARMv8-A DDI 0487G.b, D4.3
- linux/arch/arm64/mm/fault.c
- QEMU target/arm/ptw.c
```

Save to `research/mmu-page-tables.md`.
