# F02.5.4.2 evidence

Revision: `831af1d5`
Validation: 23 focused F03 tests, full local suite, source-file limits, diff, and roadmap checks
Result: PASS
Artifacts: [active runner](../../../../03-feature-matrix/01-profile-scope/validate_profile_scope.py),
[v2 requirements](../../../../03-feature-matrix/01-profile-scope/source_requirements_v2.json), and
[sealed source lock](../01-role-aware-source-contract/source_contract.lock)
Profile: source provenance only; no guest, browser, CTS execution, or qualification result

Task ID and date: F02.5.4.2, 2026-09-20. Tested code commit: `831af1d5`, pushed before this receipt.
The code worktree was clean before this metadata update. The active gate accepts only sealed
source-contract hash `d2be08ced8a806f001543e9218b6758a0a4b89825b0c4c940b9ced7c119f1ac3` and inventory-lock
`44a0f280e0ed854091a33e458122bd8cd9a0f9fc2e4c51a7a5be92bf48d8c6f4`. No guest image, browser,
adapter, driver, CTS binary, or renderer executed.

## Gate result

The active `validate_profile_scope.py` has no `--manifest` fallback. It loads the F02.5.4.1 raw-byte
lock, projects six ordered binding records, and reports `PASS: source gate is complete; profile
matrices remain blocked`. Every profile remains `blocked` with `matrix-incomplete`; source admission
does not mark support, conformance, certification, performance, or CTS execution.

The immutable v1 profile files and their runner remain for superseded F02.4 evidence. The active v2
files are side by side, so historic schema-1 byte hashes are not rewritten. Matrix v2 rows contain only
`normative-root` and `full-suite-root`; the validator resolves their exact records through the sealed
API rather than accepting a parallel source-ID catalog.

## Commands and results

From `/Users/petreleon/code/WebBoxVM`:

```sh
make graphics-profile-source-gate-test
# v1: 15 passed; v2: 8 passed
make test
# graphics checks passed; Rust suite passed; Node checks passed
cargo test -p emulator --test source_file_limits --quiet
# 6 passed
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
```

Negative checks reject a legacy alias, missing/duplicate/reordered role, cross-profile record, wrong
kind/scope/revision/digest, filtered selector, missing closure, registry/transform substitute, stale
header or raw lock, cache-module decoy, legacy matrix identity fields, and a supported row or positive
CTS state. The focused test also proves the loader restores `sys.path` after F02 import isolation.

Decision and limits: the only completed transition is source sufficiency to `matrix-incomplete`. This
is not evidence of a guest API, browser operation, renderer behavior, coverage, CTS execution,
conformance, certification, profile support, performance, or a near-native VM claim. F03.5 is now a
later audit of actual imported rows, so it does not duplicate this gate.

Commit/push verification: `831af1d5` is pushed to `origin/codex/graphics-f01-baseline`. `gh run list`
returned no run for that SHA, so no GitHub Actions validation is claimed. Next ready work includes
F02.5.4.3 and the now-unblocked per-API F03 inventory leaves.
