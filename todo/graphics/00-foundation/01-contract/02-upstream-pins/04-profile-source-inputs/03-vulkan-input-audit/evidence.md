# F02.4.3 evidence

Revision: `f3d7b5e520260b1bc3cc3b3d19518d8dedb587c5`
Validation: pinned-root retrieval, local candidate/closure contracts, hostile tests, limits, roadmap,
whitespace, and `make test`
Result: PASS
Artifacts: [candidates](candidates.json), [spec directives](spec_includes.json),
[VCTS references](mustpass_references.json), [spec contract](spec_include_contract.py),
[VCTS contract](mustpass_reference_contract.py), [candidate tests](candidate_audit_test.py), and
[spec tests](spec_include_test.py)
Profile: Vulkan 1.4 core source provenance and rejection analysis only; no guest, browser, CTS-run,
conformance, or performance claim.

Task ID and date: F02.4.3, 2026-09-09.

Tested commit: `f3d7b5e520260b1bc3cc3b3d19518d8dedb587c5`; the receipt and
status edits are deliberately outside that tested code commit. Upstream inventory revision:
`08be83edf7949e0d406bd4d3b9827b6abbd91d0312e3e08fcc74b87f6786e1a6`.

Source tags: `v1.4.362` resolves from annotated tag `cafe0e3f42089d3543219dbd92b15bf9e0af72fe`
to Vulkan-Docs commit `f84d432d5b8912362f96f581f29bbc4f3c8c7843`; `vulkan-cts-1.4.6.2`
resolves from annotated tag `42c723aa10d2652590f02741827aef43b0421d23` to VK-GL-CTS commit
`f6a29701220f34dd1407513bfe80d74ca7b392ce`.

Rejected `vulkan-14-spec`: [vkspec.adoc](https://raw.githubusercontent.com/KhronosGroup/Vulkan-Docs/f84d432d5b8912362f96f581f29bbc4f3c8c7843/vkspec.adoc),
SHA-256 `069b7e6d6326969df7b4a86f189f7ba22359e93ca7aa76c23301504667d3c4b0`,
8,685 bytes, and SPDX `CC-BY-4.0`. Its 73 direct `include::...[]` directives are recorded in
`spec_includes.json` in source order; their independently derived transcript SHA-256 is
`b35c3b8907a3338e688f31bfffd98c174858d31b95d88607046a07314f6e964b`. That transcript is not
the 8,685-byte root and cannot prove a resolved document closure.

Rejected `vulkan-cts-mustpass`: [vk-default.txt](https://raw.githubusercontent.com/KhronosGroup/VK-GL-CTS/f6a29701220f34dd1407513bfe80d74ca7b392ce/external/vulkancts/mustpass/main/vk-default.txt),
SHA-256 `b689703bdc65a04764db3b9a8f6fe872b3fe94d0df68d78f6da6e5a06cfa9ed4`, 3,347 bytes,
and Apache-2.0 repository terms. Its 98 direct `vk-default/*.txt` lines are reproduced byte-for-byte
by the reference artifact. They include `wsi`, `video`, `ray-query`, `cooperative-vector`,
`data-graph`, and `tensor`, so the root is a broader upstream default rather than a Vulkan-1.4-core
selector. The inspected committed member set has 14 files over F02.2's 8 MiB source limit and totals
434,669,348 bytes; it is not admitted or fetched into the repository. `vk-gl-cts-api-version` remains
only a related one-case input, not a must-pass catalog.

Exact commands, from `/Users/petreleon/code/WebBoxVM`:

```sh
git ls-remote --tags https://github.com/KhronosGroup/Vulkan-Docs.git 'v1.4.362*'
git ls-remote --tags https://github.com/KhronosGroup/VK-GL-CTS.git 'vulkan-cts-1.4.6.2*'
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/02-fetch-verifier/01-fetch-contract/source_fetch_test.py
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/04-profile-source-inputs/03-vulkan-input-audit/candidate_audit_test.py
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/04-profile-source-inputs/03-vulkan-input-audit/spec_include_test.py
cargo test -p emulator --test source_file_limits --quiet
PYTHONDONTWRITEBYTECODE=1 make test
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
git diff --check
```

Expected result and minimum nonzero case count: two rejected exact candidates, one 73-directive
transcript, one 98-line VCTS root record, and hostile tests. Actual: candidate contract reports
`rejected, rejected`; VCTS contract reports 98 direct selectors; specification contract reports 73
direct includes; source fetch contract 15/15; candidate audit 10/10; specification audit 6/6;
source-file limits 6/6; roadmap checker `PASS: 216 documents, 130 tasks, 39 complete`; and local
`make test` exited zero (Cargo: 1,127 passed, 3 ignored; Node: 337 passed).

Negative/reference checks reject mutable or substituted sources; changed digest, bytes, license,
cache, generator role, provenance, decision, count, list member, order, duplicate, unsafe path,
schema, candidate binding, and boundary erasure. The F02.2 policy remains separately covered 15/15.

Raw payload handling: direct pinned-source probes used disposable `/private/tmp/webboxvm-vulkan-audit.*`
and `/private/tmp/webboxvm-vulkan-spec-audit.*` directories; SHA-256 and byte count matched the
records above, then the directories were deleted. Reproduce with the raw URLs and hashes above; no
upstream payload or large CTS closure is tracked here.

Software fallback detection, actual execution route, and performance conditions: not applicable; this
is an offline audit and no graphics workload ran. The local xcrun FSEvents/cache warnings during
`make test` were non-fatal; no test reported a failure.

First failing subcheck: the initial roadmap check tried to decode generated `__pycache__` files as
roadmap documents. Those generated directories were removed; the final checker passed. This was not
a candidate, protocol, guest, or browser regression.

Decision and limits: retain both roots as rejected. Future admission needs a fresh-cache, exact
resolved/transitive AsciiDoc closure with macro/conditional configuration, plus a per-member VCTS
closure with hashes, size policy, and explicit core-versus-WSI/extension exclusions. This receipt
does not claim Vulkan, Venus, VirGL, Mesa, conformance, browser execution, or near-native performance.

Commit/push verification: code commit `f3d7b5e520260b1bc3cc3b3d19518d8dedb587c5` and its receipt
commit `5df6eca28cd1c57193f09f37113edb665cdbcf9b` were pushed. `git ls-remote origin
refs/heads/codex/graphics-f01-baseline` returned
`5aaa11f250a2e7e81b0c33d7c80ebeeb87817220`, matching local HEAD. No remote-CI result is claimed.

Next ready task: split F02.4.4 into admission-model, inventory-cutover, provenance-consumer,
fresh-cache, and F03-gate leaves; F02.4 remains open.
