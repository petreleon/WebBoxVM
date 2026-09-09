# F02.3.3.4.1.1.4 evidence

Revision: e80cb4a8dbad75198820efe9c9efcee15c9d880a
Validation: chunker 9/9 + reproducibility 4/4 + limits + roadmap + full suite
Result: PASS
Artifacts: v1/v2 chunk-schema, output, generator, and hermetic tests; SHA-256s below.
Profile: offline reproducibility only; raw lock identity, no inventory parsing, guest, browser, or graphics runtime

Task ID and date: F02.3.3.4.1.1.4, 2026-09-09 Europe/Bucharest.
Tested commit and dirty diff hash: e80cb4a8dbad75198820efe9c9efcee15c9d880a; clean tree before receipt.
Historical input identity at this receipt: the checked F06 schema-v1 fixture and its generated
metadata bytes remained unchanged; schema-v2 tests used a hermetic opaque raw lock only.
Artifacts and SHA-256: `graphics_chunk_schema.py`
2c12e751f00cb8a59363982ab71798ccb5fa47f8c3fc15462f6e989eecfbb556;
`graphics_chunk_output.py` 608bc1649cf433394de1f35631d13e3931d1819bee9aa4a06c73ab45802bfee5;
`graphics_chunker.py` 09229719b89035a1030ceecf0748a570d6575006a290614911e4e59175113536;
tests e236d281d5b9cf379ad93944c9b3f4ad06ebc5b4615ecb61a19a9033562d04e1 /
8aec7225c543eb36e4da9f5121c1e0faf341b753127dd2ef8e60c7801dab56ef.
Guest image and build hashes: not applicable; this task neither builds nor runs a guest.
Browser, OS, adapter and driver: not applicable; no browser or GPU execution path was exercised.
Exact commands, working directory and tool versions: Python 3.14 stdlib suites from
`/Users/petreleon/code/WebBoxVM`: `PYTHONDONTWRITEBYTECODE=1 python3 scripts/test_graphics_chunker.py`
and `... scripts/test_graphics_reproducibility.py`; then source limits, roadmap checker, diff check,
and `make test`.
Expected result and minimum nonzero case count: v1 keeps its historical output bytes; v2 uses only
the raw lock SHA-256, requires `--inventory-lock`, rejects stale locks/legacy fields, and reproduces.
Actual passed/failed/skipped counts and exit codes: chunker 9/0/0; reproducibility 4/0/0; limits
6/0/0; full Rust 1,151/0/3 ignored; Node 337/0/0; all exit 0.
Negative/reference checks and observed output: v2 rejects a manifest-only field, missing or stale
`--inventory-lock`, stale metadata, and reordered records. Repeated v1/v2 output maps are byte-equal.
The retained v1 fixture hashes are chunk `9eb18a787a5a3e877317a43dd4839ac41d10d44e7598773ab625722a1a03e78b`
and metadata `641f57ff68a2740d68911d34a500b113bfab8899ede250ea60be8e70e9045526`.
Raw log/image/sample paths, SHA-256 and retrieval/reproduction instructions: no external payload,
image, or runtime sample was retained; run the two listed Python suites to rebuild temp bundles.
Software fallback detection and actual execution route: Python stdlib JSON/bytes only; no parser for
the lock composition and no fallback renderer.
Performance conditions and frozen protocol version, when applicable: not applicable; no performance
or protocol behavior is measured.
First failing subcheck or blocker, when applicable: none; the first chunker suite passed 9/9.
Decision and limits of the evidence: accept only versioned deterministic chunk generation. It does
not cut over the checked fixture, parse inventory components, or claim guest-visible graphics support.
Commit/push verification: local feature commit e80cb4a8dbad75198820efe9c9efcee15c9d880a; not pushed
because explicit authorization for the remote destination has not been granted.
Next ready task: F02.3.3.4.1.1.5 after the three preparation receipts are recorded.
