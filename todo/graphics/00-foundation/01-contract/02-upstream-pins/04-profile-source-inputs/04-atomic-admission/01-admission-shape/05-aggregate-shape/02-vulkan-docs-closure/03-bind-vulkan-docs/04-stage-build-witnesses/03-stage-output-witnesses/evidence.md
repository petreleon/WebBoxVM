# F02.4.4.1.5.2.3.4.3 receipt — staged Vulkan Docs output witnesses

Revision: `7a27d6ef` verified implementation
Validation: 12 focused tests, live stage/verify CLI, source limits, roadmap checker, `git diff --check`, and `make test`
Result: PASS
Artifacts: temporary private cache only; it was deleted after verification
Profile: Vulkan 1.4 recorded clean-run output witnesses only; unadmitted and not cutover-ready

## Result

The public one-flow contract first re-verifies the exact staged input receipt, then holds separate
no-follow descriptors for `runs/observer-a-r5/generated` and `runs/observer-b-r1/generated`.
It reconstructs, rehashes, atomically stages, rehashes again, and inventories each complete tree
under its own `outputs/<run-id>/` namespace. Equal tree and member digests never collapse the two
recorded origins.

Each selected run produced exactly `2,530` regular files and `17,019,466` bytes with tree SHA-256
`26e8e484d34222d9ba8a72c883ff8bb65a4e5194f399c24aa9bd5d7e47c49f32`. The required primary
`out/html/vkspec.html` is `10,377,052` bytes with SHA-256
`896452d3a3e4887ba1bad81a6b9b91e9b3b324e1fffbb9e1dec889d0b06ac041`.

The live cache contained 5,060 separately stored output payloads plus one compact output receipt;
with the prerequisite inputs it contained 6,822 files total. It published no `markers/*.complete.json`.
The receipt file SHA-256 was `e1bc709b5283ffdcbef61711c10481e3110bc8ef515e4a7725828f4c209b7fe5`;
its self-identity was `7092c0fc523eed1b49971aa311e6ad17146d01242d29d87399436c755db72348`.
The live plan digest was `5dcc55e3cd979010d73142e7dde13361e0b36f64ff42aaa1e436a8a5eab5de77`.

## Boundary and negative coverage

The receipt binds the live reviewed build witness, image/platform/argv/environment/mounts/toolchain,
semantic source identity, scope/comparison receipts, exact input-stage receipt, both run identities,
and ordered output manifests. It retains distinct capture artifacts and run/I/O identities even though
the two output trees have equal content.

Provider and cache traversal are descriptor-anchored and no-follow. Staging rejects symlinks,
non-regular members, hard links, unsafe or aliased roots, source/output substitution, missing/extra
members, count/byte/member-cap excesses, primary changes, root swaps, witness changes, partial prior
output worktrees, stale/resealed receipts, duplicate JSON keys, and cache mutations. Exact inventory
is limited to `outputs/`, deliberately leaving `inputs/` and future `markers/` to their own children.

Focused positive coverage instruments all 5,060 provider copies, confirms distinct `(run_id, selector)`
targets for equal-content trees, includes zero-byte members, rehashes through the public verifier, and
confirms no completion marker. Hostile coverage exercises symlink/FIFO/hardlink/mutation/alias/root-swap
filesystem cases, tightened bounds, untrusted plans, and receipt/capture/run/witness/primary mutations.

## Validation

Focused command from this directory:

```text
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  test_vulkan_docs_output_contract.py vulkan_docs_output_hostile_test.py -v
```

It passed 12/12 in 143.457 seconds. The live sequence staged prerequisite inputs, then printed both:

```text
OUTPUT-STAGE: staging-only-unadmitted, 2 witnesses, 5060 files, 0 markers
```

`cargo test -p emulator --test source_file_limits --quiet` passed 6/6. `make test` exited 0:
1,127 emulator unit tests passed (3 ignored), boundary/source-limit and roadmap checks passed, and
Node passed 337/337. `git diff --check` passed. No Rust, Wasm, or browser source changed, so
`make web-pkg` was not applicable. Remote CI was not run.

The live cache root was `/private/tmp/webboxvm-output-stage-live.aVUsZA`; it was removed after
the count/hash checks. First failing subcheck: none.

## Limits and next work

This is captured output evidence, not a fresh official build, a reusable completion marker, or a
compatibility/performance result. Descriptor checks and each public verification catch named mutations
and divergence; they cannot retroactively prove historical origin after a same-UID byte-identical
replacement occurring after the final check.

Implementation commit: `7a27d6ef`, pushed to `origin/codex/graphics-f01-baseline`. The next child is
`F02.4.4.1.5.2.3.4.4` — publish and reuse the staged witness cache. Its parent remains open.
