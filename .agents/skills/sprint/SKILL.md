---
name: sprint
description: Query WebBoxVM development sprints, check progress against todo.md, and identify the next task to implement. Use when asked about roadmap, what to work on, sprint status, or todo items.
metadata:
  type: project
---

# Sprint Tracker — WebBoxVM

WebBoxVM uses a sprint-based roadmap in `todo.md`. This skill helps agents read the roadmap, validate progress, and pick the next task.

## Quick start

```bash
# View current sprint status
cat todo.md

# Check which tests currently pass
cargo test --workspace 2>&1 | tail -15
```

## Sprint structure

| Sprint | Theme | Status | Tests (approx) |
|---|---|---|---|
| 1 | CPU Core | ✅ Complete | 23 |
| 2 | Bootloader | ✅ Complete | 40 |
| 3 | EFI Stub Protocols | ✅ Complete | — |
| 4 | PE Relocations & Decompressor | ✅ Complete | 69 |
| 5 | MMU | 🚀 Active | — |
| 6 | Busybox Shell | 📅 Planned | — |

## Active sprint: Sprint 5 — MMU

Open items in `todo.md` (as of today):

- [ ] 3-level page table walk (39-bit VA)
- [ ] 2048-entry software TLB
- [ ] `SCTLR_EL1` enables MMU

## How to verify a task is done

1. **Code exists**: grep for the feature in `emulator/src/`.
2. **Tests pass**: `cargo test` passes with zero warnings.
3. **Real kernel boots further**: the slow `#[ignored]` tests in `arm64::interpreter::tests` execute more instructions without crashing.

Run the real-kernel trace to see current boot depth:

```bash
cargo test real_kernel_runs_past_prologue_trace -- --ignored --nocapture
```

## How to update the roadmap

When you complete a sprint item:

1. Edit `todo.md` and change `- [ ]` to `- [x]` for the item.
2. Run the full test suite: `cargo test --workspace`
3. If all pass, update the **Result** line under the sprint with the new test count.
4. If the sprint is now fully complete, change the sprint header from `(Active development)` to `(Complete)`.

## Next-task heuristic

1. Read `todo.md`.
2. Find the first unchecked item in the lowest-numbered active sprint.
3. Read the relevant source files (see architecture below).
4. Implement + test.
5. Update `todo.md`.

## Code architecture for active sprint

| Task | Likely files to touch |
|---|---|
| Page table walk | `emulator/src/memory.rs`, `emulator/src/bus.rs` |
| TLB | `emulator/src/memory.rs` (new cache struct) |
| `SCTLR_EL1` MMU enable | `emulator/src/arm64/system_regs.rs`, `emulator/src/arm64/execute.rs` |

## Gotchas

- `todo.md` uses GitHub-style task lists (`- [ ]`). Do not change the format or parsing will break.
- The `Image.gz` kernel is 37 MB. Tests that load it are marked `#[ignore = "slow"]` and must be run with `--ignored`.
- The real kernel path is hard-coded to `/Users/petreleon/code/WebBoxVM/Image.gz` in `loader::kernel` tests. On a different machine, place the kernel at that exact path or the slow tests will fail.
- Sprint 6 (Busybox Shell) depends on Sprint 5 (MMU). Do not start Sprint 6 items until MMU is at least minimally working.
