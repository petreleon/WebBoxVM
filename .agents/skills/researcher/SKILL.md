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

## Extended guidance

For benchmark design, search shortcuts, citation format, and the research-note
template, see `references/research-methods.md`.
