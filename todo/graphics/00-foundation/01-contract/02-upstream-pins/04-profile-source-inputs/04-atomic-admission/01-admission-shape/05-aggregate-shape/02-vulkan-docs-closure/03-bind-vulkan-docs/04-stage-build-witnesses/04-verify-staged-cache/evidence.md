# F02.4.4.1.5.2.3.4.4 receipt — reusable staged Docs cache marker

Revision: `b2e47d3f` verified implementation
Validation: 4 focused tests, live stage/publish/reuse CLI, source limits, roadmap checker,
`git diff --check`, and `make test`
Result: PASS
Artifacts: private temporary cache only; it was removed after the final reuse/count/hash check
Profile: Vulkan 1.4 recorded Docs witness cache only; unadmitted and not cutover-ready

## Result

`publish_staged_cache()` constructs one live plan and retains one descriptor-held cache session.
It parses the prior receipts, rehashes every one of the 1,760 staged inputs, reconstructs both
independently stored 2,530-file output trees, and checks the exact whole-plan worktree before
atomically publishing the fixed V1 marker. It then repeats the complete rehash/inventory and
strictly re-reads the marker before confirming the cache root. `reuse_staged_cache()` has the
same held-session rehash/inventory path but neither stages nor opens a provider.

The marker is made only through the existing V1 grammar and binds the reviewed plan digest
`5dcc55e3cd979010d73142e7dde13361e0b36f64ff42aaa1e436a8a5eab5de77`, which transitively seals
the scope, comparison, build witness, canonical input manifest, both ordered run identities, and
both output manifests. Its state remains `staging-only-unadmitted`, `admitted=false`, and
`cutover_ready=false`. It is not a fresh build, admission, or cutover result.

The live external cache had exactly 6,823 regular files: 1,760 input payloads plus their receipt,
5,060 separately retained output payloads plus their receipt, and one marker. The marker was
1,464 bytes; its physical SHA-256 was
`5d49a3316330d8707406a6566dc6c4e8b0c4d2361d27b74523f9e98a8c6ba38a`, while its sealed
`marker_sha256` was `f600e7b12cb39532680c05fa8dba1ec51a47a765ebafb3212a7fd26f76a363ee`.

## Boundary and negative coverage

The aggregate descriptor walk accepts no sibling content in this plan worktree: pre-publication it
expects exactly 6,822 files, then exactly the one modeled marker extra. It rejects unlisted roots,
marker siblings, temporary entries, non-regular members, symlinks, hard links, excess/deep paths,
partial receipts/payloads, stale or malformed/self-mutated/active/reordered/cross-plan markers,
and divergent inputs or outputs. Atomic no-overwrite plus complete post-publication rehash makes a
competing marker publication fail closed; every successful reuse rehashes the full closure.

Focused positive coverage observes all 1,760 input rehashes and both 2,530-file witness scans on
publication and reuse, verifies the provider-free path, and checks the exact count. Hostile tests
cover missing stages, input/output mutation, malformed/stale/active/cross-plan/reordered markers,
closure junk, marker siblings, and the publication race. As with the prerequisite descriptor
contracts, a same-UID byte-identical replacement after the final check cannot be retroactively
distinguished from an unchanged artifact.

## Validation

From this directory the focused command was:

```text
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  test_vulkan_docs_cache_marker_contract.py vulkan_docs_cache_marker_hostile_test.py -v
```

It passed 4/4 in 733.984 seconds. A fresh temporary root first staged the inputs and outputs, then
published a marker and completed a later CLI reuse:

```text
CACHE-MARKER: staging-only-unadmitted, 1760 inputs, 2 witnesses, 0 cutover-ready
```

`cargo test -p emulator --test source_file_limits --quiet` passed 6/6. `make test` exited 0:
1,127 emulator unit tests passed (3 ignored), all boundary/source-limit and graphics-roadmap
checks passed, and Node passed 337/337. `git diff --check` passed. No Rust, Wasm, or browser
source changed, so `make web-pkg` was not applicable. Remote CI was not run. The exact temporary
root `/private/tmp/webboxvm-cache-marker-live.xsJhVJ` was removed and its absence verified.

## Decision and next work

Implementation commit: `b2e47d3f`. This completes the staged-witness parent without changing any
active F02 cache, V1 grammar, F03 state, guest/browser behavior, CTS/conformance, or performance
claim. Next, `F02.4.4.1.5.2.3.5` must prove the actual Docs closure; the independent VCTS closure
is separately blocked by its upstream selector evidence.
