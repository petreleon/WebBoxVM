---
name: computer-scientist
description: Apply computer science theory to WebBoxVM — analyze algorithms, prove invariants, reason about ARM64 semantics, design data structures, and evaluate complexity. Use when refactoring core loops, designing the MMU/TLB, or verifying instruction correctness.
metadata:
  type: project
---

# Computer Scientist — WebBoxVM

Apply formal reasoning, algorithm analysis, and CS theory to the emulator codebase. This skill is for when you need more than "it works" — you need "it is correct."

## When to invoke

- Designing a new subsystem (MMU, TLB, JIT, VirtIO)
- Refactoring a hot loop (fetch-decode-execute, memory access)
- Formalizing ARM64 instruction semantics
- Proving an invariant or finding a counter-example
- Choosing between two data structures or algorithms
- Reviewing code for subtle bugs (overflow, alignment, concurrency)

## Techniques

### 1. Invariant checking

For every mutable struct, state the invariant aloud, then verify it in the code.

Example — `SystemBus`:
- Invariant: every address maps to at most one device
- Check: `bus.rs` `read()` and `write()` must not overlap regions
- Tool: `grep -n "0x" emulator/src/bus.rs` and verify ranges are disjoint

### 2. Complexity analysis

Before adding a cache or lookup table, calculate the complexity:

| Operation | Current | Proposed | Notes |
|---|---|---|---|
| Memory read (flat) | O(1) | — | array index |
| Memory read (with MMU) | O(page-table walk depth) | O(1) with TLB | amortized |
| Instruction decode | O(1) | — | bit masking |
| TLB lookup | — | O(1) | hash or direct-mapped |

When proposing a change, fill out this table and justify the trade-off.

### 3. ARM64 semantics formalization

Every instruction in `opcodes.rs` should have a single, unambiguous semantic definition. When adding or reviewing an instruction:

1. **Reference the ARM ARM** (ARM Architecture Reference Manual). Cite the section.
2. **Write the pre-condition**: what must be true before execution?
3. **Write the post-condition**: what is true after execution?
4. **Check edge cases**: overflow, carry, sign extension, alignment faults.

Example template for `ADD (immediate)`:
```
Pre:  SP_ELx aligned to 16 bytes if using SP
Post: Rd = Rn + imm12 (zero-extended)
Edge: imm12 is shifted by 12 if shift==1
```

### 4. Proof sketches

For critical paths, write a short proof sketch in a comment or docstring:

```rust
/// Proof sketch: `run()` terminates because `max_steps` is strictly decreasing.
/// If `max_steps == 0`, the loop exits immediately.
/// Each iteration decrements the counter by 1.
```

### 5. Property-based thinking

When writing tests, think in properties, not examples:

- **Commutativity**: `ADD X0, X1, X2` == `ADD X0, X2, X1` for unsigned
- **Identity**: `ADD X0, X1, #0` leaves `X1` unchanged
- **Round-trip**: `STR` then `LDR` recovers the original value (for aligned accesses)
- **Monotonicity**: `max_steps` bounds the number of instructions executed

## CS principles applied to WebBoxVM

| Principle | Application |
|---|---|
| **Single responsibility** | Each module does one thing: `decode.rs` only decodes, `execute.rs` only executes |
| **Composition over inheritance** | `Armv8Cpu` is composed of `RegisterFile`, `ProcessorState`, `SystemRegisters` |
| **Cache-oblivious design** | Page table walks should be cache-friendly (breadth-first, not depth-first) |
| **Amortized analysis** | TLB miss penalty is high, but hit rate amortizes it |
| **Correctness by construction** | Use Rust's type system to make illegal states unrepresentable |

## Data structure design rubric

When proposing a new data structure (e.g., TLB, page table cache, instruction cache):

1. **What is the key?** (VPN, physical address, PC)
2. **What is the value?** (PFN, PTE, decoded instruction)
3. **Expected size?** (how many entries?)
4. **Lookup pattern?** (temporal locality, spatial locality)
5. **Eviction policy?** (LRU, random, direct-mapped)
6. **Rust type?** (`Vec`, `HashMap`, `array`, custom arena)

Default to the simplest structure that satisfies the constraints. A 2048-entry direct-mapped array is often faster than a `HashMap` for small, fixed-size caches.

## Common formal bugs in emulators

| Bug pattern | How to catch |
|---|---|
| Sign-extension error | Check `as i32 as i64` vs `as u32 as i64` |
| Overflow in address calculation | Use `wrapping_add` explicitly, or `checked_add` |
| Misaligned memory access | Assert `addr % size == 0` in debug builds |
| Off-by-one in bit masks | Verify with `assert_eq!(mask.count_ones(), n)` |
| State mutation order | Document order: registers before PC, or PC before registers? |

## Prompts for reasoning

Use these prompts when stuck on a design decision:

- "What is the simplest invariant that, if violated, would cause a bug?"
- "Can I make an illegal state unrepresentable with the type system?"
- "What is the worst-case time/space complexity, and when does it happen?"
- "If I had to prove this function correct by induction, what is the induction hypothesis?"
- "What are the three most likely edge cases, and do the tests cover them?"
