# F02.4.4.1.5.5.3.1 evidence

Revision: 82d4951e0d01a0072bddfd42a664eb12cd639e70
Validation: cache contract 5/5 + `make test` (Rust 1,127 pass/3 ignored; Node 337 pass)
Result: PASS
Artifacts: contract sha256=35926a5078b4e1aedc73bd8d91319f4bf690b83f774a46fba5dc71067dce1195;
filesystem sha256=1dd9dcf4be5c663c7f1a51f94ab8d9e91559c64f3dcfd5406d207869b6f40cc4;
content sha256=db5b2f8aa9f5231e48298834a9a224337c37d9088ce8fd2bc2e4a3a875262a54;
test sha256=f67fc06631343a99d72b681a4e22bb57ceb1758b044d257f742047429629d3b9
Profile: hermetic external-cache mechanics only; no live VCTS payload, CTS, guest, browser, or performance run

Task ID and date: F02.4.4.1.5.5.3.1, 2026-09-10 Europe/Bucharest.
Tested commit and dirty diff hash: `82d4951e0d01a0072bddfd42a664eb12cd639e70`; task-local paths
were clean after the commit.
Pinned identity consumed by the fixture: annotated `vulkan-cts-1.4.6.2` tag object
`42c723aa10d2652590f02741827aef43b0421d23`, peeled commit
`f6a29701220f34dd1407513bfe80d74ca7b392ce`, root SHA-256
`b689703bdc65a04764db3b9a8f6fe872b3fe94d0df68d78f6da6e5a06cfa9ed4`.
Guest image and build hashes: not applicable; this leaf validates source-cache mechanics only.
Browser, OS, adapter and driver: not applicable; no browser or GPU route was exercised.
Exact commands and working directory: from `/Users/petreleon/code/WebBoxVM`, Python 3.14.6 ran
`vcts_cache_contract_test.py`, `scripts/check_graphics_roadmap.py`, and `make test`; `git diff --check`
and physical line counts covered all maintained cache files.
Expected result and minimum nonzero case count: five hermetic methods use 98 real ordered selector paths,
99 bounded synthetic streams, and a temporary external cache without retaining a full suite in Git.
Actual passed/failed/skipped counts and exit codes: cache 5/0/0; `make test` exited 0 with Rust
1,127/0/3, source limits 6/0/0, Node 337/0/0; roadmap reports 295 documents, 178 tasks, 49 PASS-complete,
and 27 superseded. All maintained cache files are at most 179 physical lines; whitespace check exited 0.
Negative/reference checks: redirect, length, unsafe URL/path, missing/tampered/symlinked cache, unsafe
permissions, a repository symlink alias, CRLF/missing terminal LF, and a supplied-ledger source swap fail.
Offline verification rehashes without creating cache paths or making transport requests.
Raw log/image/sample paths, SHA-256 and reproduction: no remote payload or GPU capture was retained.
Reproduce with `python3 .../01-cache-contract/vcts_cache_contract_test.py`.
Software fallback detection and actual execution route: not applicable; synthetic `BytesIO` transport only.
Performance conditions and frozen protocol version: not applicable; no workload ran.
First failing subcheck or blocker: none for this contract leaf. Its POSIX `/dev/fd` snapshot is intentional;
the real 98-member capture, fresh-cache requirement, Git-tree membership proof, and offline receipt remain `.3.2`.
Decision and limits of evidence: PASS means secure synthetic cache mechanics and an unadmitted marker only;
it does not admit a VCTS source, classify Vulkan core, run CTS, establish conformance, or measure speed.
Commit/push verification: `82d4951e0d01a0072bddfd42a664eb12cd639e70` was pushed; `git ls-remote`
returned that exact SHA for `refs/heads/codex/graphics-f01-baseline`.
Next ready task: F02.4.4.1.5.5.3.2 — capture the live closure.
