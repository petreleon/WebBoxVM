# F06 — Enforce modular structure and the 180-line limit

[Parent list](../README.md) · [Worker instructions](../../../workflow.md)

Task: F06
Depends: F01
Evidence: pending

Prerequisite lists: [F01](../../01-contract/01-baseline/README.md).

## Outcome

Code, tests, generators and nested task documents remain small and discoverable.

## Starting points

- [emulator/tests/source_file_limits.rs](../../../../../emulator/tests/source_file_limits.rs)
- [todo.md](../../../../../todo.md)
- [web/js/webgpu-3d.js](../../../../../web/js/webgpu-3d.js)

## Checklist

- [ ] Define separate protocol, shared runtime, shader frontend/backend, platform and test-fixture
  folders with public boundaries.
- [ ] Extend line-limit coverage to todo, guest and generated maintained sources; split existing
  touched oversized files by responsibility.
- [ ] Add deterministic generator chunking and extend the roadmap checker when a new sublist depth
  is introduced.
- [ ] Run the verification below, review the result, and attach the completed evidence receipt.

## Verification

- Run cargo test -p emulator --test source_file_limits --quiet and python3
  scripts/check_graphics_roadmap.py.
- A 181-line fixture is rejected; all files created or changed for the feature are at most 180
  physical lines.

Register the exact task check through F05 before implementation completion. Bootstrap F01/F02/F05
use direct reproducible commands and receipts until that runner exists. Record commands, nonzero
test counts (or explicit design checks), output and expected results; a planned check is not PASS.

For a task that discovers a design choice, multiple independent feature families or too much work
for one coherent commit, create child folders first using the worker instructions. Keep this parent
open until every child and its acceptance checks pass.
