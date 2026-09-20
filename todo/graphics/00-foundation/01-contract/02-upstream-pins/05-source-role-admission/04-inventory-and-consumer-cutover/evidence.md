# F02.5.4 aggregate evidence

Revision: `f5337681`
Validation: child receipts F02.5.4.1–F02.5.4.3, full local suite, source-file limits, diff, and roadmap checks
Result: PASS
Artifacts: [sealed contract](01-role-aware-source-contract/source_contract.json), [F03 gate receipt]
(02-f03-gate-and-matrix-binding/evidence.md), and [F05 aggregate receipt]
(03-future-f05-adapter-and-receipt/f05_source_aggregate_receipt.json)
Profile: source-consumer cutover only; no guest, browser, CTS, certification, or performance result

Task ID and date: F02.5.4, 2026-09-20 Europe/Bucharest. The raw-byte F02 seal, active F03 v2 gate,
and profile-neutral future-F05 accessor consume the same complete source contract: eight records, six
ordered mandatory roles, and three unfiltered closure receipts. The contract remains no-claim and all
F03 profiles remain `blocked` by `matrix-incomplete`.

F05.1 remains source- and profile-independent. The completed accessor is outside its generic runner,
does not register a profile or check, and cannot turn source provenance into support. F05.2 remains the
future owner of any profile-bound registration.

The aggregate capture refreshed `/private/tmp/webboxvm-f02543.bO41B6` from empty and verified exactly
nine external selector/license files. The tracked receipt has SHA-256
`85980a92e5223261e7fce6fc162e10171bd671800a5fe9d4c2460e01b68d5242`; it records zero CTS executions,
zero imported real matrix rows, and false API, conformance, certification, profile, and performance
claims. No cache payload is in Git.

The mandatory local gates passed: `make test`, `cargo test -p emulator --test source_file_limits --quiet`
(6/6), `git diff --check`, and `PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py`.
The child receipts retain focused counts and hostile cases. No guest/browser/renderer ran, so those lanes
remain unclaimed. `f5337681` is pushed to `origin/codex/graphics-f01-baseline`; no Actions run exists
for that SHA. Next ready task: F03.2, F03.3, or F03.4.1.
