# F02.2 evidence

Revision: `59d65bf56c88d548cea919d8e1eceab86a6e9f9a` aggregate implementation commit
Validation: F02.2.1 contract 13/13, F02.2.2 fixtures 7/7, F02.2.3 live fetch 15/15 then offline re-hash 15/15
Result: PASS
Artifacts: child receipts and the disposable F02.2.3 external cache audit
Profile: source-acquisition contract only; no guest API, Mesa integration, browser route, or performance claim

F02.2 is complete only as an aggregate of its three checked children:

- [F02.2.1 receipt](01-fetch-contract/evidence.md) defines and tests immutable HTTPS, redirect, byte,
  hash, and external-cache acceptance behavior.
- [F02.2.2 receipt](02-hermetic-fixtures/evidence.md) adds seven in-memory transport failures without
  a public network request or a production-policy relaxation.
- [F02.2.3 receipt](03-live-inventory/evidence.md) freshly fetched all 15 reviewed inputs, then
  independently reused/re-hashed them from its external temporary cache.

The final combined local gates are the child focused suites, `cargo test -p emulator --test
source_file_limits --quiet`, `make test`, `python3 scripts/check_graphics_roadmap.py`, and `git diff
--check`. The local live availability result is a time-bounded source-integrity check, not proof that
any guest driver, OpenGL/GLES, Vulkan, Venus, browser GPU route, or native-like performance works.

Commit/push verification: aggregate implementation commit
`59d65bf56c88d548cea919d8e1eceab86a6e9f9a` was pushed to
`origin/codex/graphics-f01-baseline`; `git ls-remote` resolved that branch to the same SHA. No remote
CI result is claimed locally. Next ready task: F02.3.1, subject to its own provenance-contract split.
