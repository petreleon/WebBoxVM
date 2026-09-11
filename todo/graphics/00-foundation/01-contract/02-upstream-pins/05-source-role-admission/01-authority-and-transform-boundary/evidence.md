# F02.5.1 evidence

Revision: `04cdec9c`
Validation: `make test`; source-role focused tests; source-file limits; roadmap checker; diff check
Result: PASS
Artifacts: `source_role_records.py`, `source_role_contract.py`, `source_role_artifacts.py`, and 11 focused cases
Profile: planning-only source-admission contract; no guest API, browser, CTS execution, conformance, certification, or performance claim

Task ID and date: F02.5.1, 2026-09-11
Tested commit and dirty diff hash: `04cdec9c`; code worktree was clean before this receipt/status update.
Upstream manifest revision: no new upstream pin admitted by this schema task.
Guest image and build hashes: not applicable; no guest or browser workload ran.
Browser, OS, adapter and driver: not applicable.
Exact command(s), working directory and tool versions:

- `/Users/petreleon/code/WebBoxVM`: `make test`
- `/Users/petreleon/code/WebBoxVM`: `cargo test -p emulator --test source_file_limits --quiet`
- `/Users/petreleon/code/WebBoxVM`: `python3 scripts/check_graphics_roadmap.py` and `git diff --check`

Expected result and minimum nonzero case count: all role checks fail closed; at least 8 structural and
3 streamed-artifact cases pass; local transforms above 8 MiB or with changed bytes fail.
Actual passed/failed/skipped counts and exit codes: `make test` exit 0; source-role structural 8/8;
artifact verifier 3/3; roadmap suites 10+4+2; generic runner 5/5; emulator unit run reported 1,130
cases with no failure; Node 338 pass/0 fail; source-file limits 6/6; no skips in the new tests.
Negative/reference checks and observed output: mutable/foreign raw URLs, false authority/support claims,
arbitrary shell argv, unresolved or cyclic inputs, filtered/fraction/Vulkan-Docs suite roots, declared-small
8 MiB+1 local artifacts, tampered builders, and changed shard bytes all raise `RoleError`.
Raw log/image/sample paths, SHA-256 and retrieval/reproduction instructions: no retained binary capture;
the temporary artifact fixtures are created and streamed by `source_role_artifacts_test.py`.
Software fallback detection and actual execution route: no graphics execution; Python validation only.
Performance conditions and frozen protocol version, when applicable: not applicable.
First failing subcheck or blocker, when applicable: none for F02.5.1.
Decision and limits of the evidence: the contract proves catalog shape and local artifact identity only.
F02.5.2 must audit/pin the actual builder and normative inputs; F02.5.3 must verify the full CTS root
and its ordered member ledger. This receipt is not full-suite, compatibility, or performance evidence.
Commit/push verification: implementation is local commit `04cdec9c`; no remote push or CI claim.
Next ready task: F02.5.2 and F02.5.3, independently.
