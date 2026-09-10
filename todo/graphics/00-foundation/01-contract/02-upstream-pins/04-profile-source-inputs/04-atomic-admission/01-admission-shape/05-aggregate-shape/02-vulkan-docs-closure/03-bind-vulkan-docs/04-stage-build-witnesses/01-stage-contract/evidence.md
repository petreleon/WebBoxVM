# F02.4.4.1.5.2.3.4.1 receipt — isolated Docs staging contract

Revision: `a7a41e1e20374c18cf9d16b92afcca941167f489` verified implementation
Validation: 15 focused tests, 56 prerequisite tests, live plan CLI, source limits, roadmap checker,
`git diff --check`, and `make test`
Result: PASS
Artifacts: no payload was staged; the two recorded inputs remain ignored under
`.artifacts/graphics/f02.4.4.1.5.2.3.3.1/`
Profile: Vulkan 1.4 core recorded-capture staging only; unadmitted and not cutover-ready

## Contract

The live plan binds the reviewed build witness
`8cfab6fe4527f3973d7d7923ffe0922ab689fc2daf2f6e955ff7603fc8ba78d6`, the scope identity
`9c0e5fb53986b22ee5d774de350127cdecfe11c021d39106a20a106203ef3339`, and the independent
comparison `f76e489565365d2b326185d2f4d511422c8b83b6b084fee0f90cd3b5646cc9ce`. It retains the
reviewed image, platform, argv, environment, mounts, recipe/toolchain, source identities, both
run IDs (`observer-a`, `observer-b`), 298 raw records, 1,462 derived records, and the known
two-run output witness (`2,530` files, `17,019,466` bytes, tree
`26e8e484d34222d9ba8a72c883ff8bb65a4e5194f399c24aa9bd5d7e47c49f32`).

`build_plan()` accepts only an absolute, non-root, non-symlinked cache root outside WebBoxVM.
Descriptor traversal uses no-follow opens; the root and every cache-relative directory must be
owned by the current user and non-group/world-writable. Canonical relative selectors reject
traversal, aliases, separators, and URL-reserved forms. Reads require a byte bound and can bind an
expected SHA-256; writes require a type and explicit public byte bound, publish through an
exclusive temporary hard-link, refuse overwrite/hard-link aliases, fsync, and re-read the result.

Plans and future markers are exact-schema, self-hashed records fixed to
`staging-only-unadmitted`, `admitted=false`, and `cutover_ready=false`. A public filesystem or
marker route also requires private live provenance, so a self-sealed plan object cannot authorize
an operation. This leaf writes no payload and publishes no completion marker.

## Validation

The focused command, run in `01-stage-contract`, was:

```text
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest vulkan_docs_stage_test.py vulkan_docs_stage_hostile_test.py -v
```

It passed 15/15 with no failures or skips: six live-plan/public-bound tests and nine hostile
filesystem/marker tests. The hostile cases cover relative/repository/root/symlink cache roots,
traversal, leaf aliases, hard links, same-inode content changes, parent swaps, both post-link
in-place and replacement mutations, non-private root/nested directories, mutable plans, and
resealed active/reordered marker fields.

The live command used an empty private `mktemp -d /private/tmp/webboxvm-stage-contract.XXXXXX`
root, then removed it:

```text
PYTHONDONTWRITEBYTECODE=1 python3 vulkan_docs_stage_contract.py \
  ../../03-capture-core-closure/01-observe-pinned-build-inputs/vulkan_docs_core_input_observation.json \
  /Users/petreleon/code/WebBoxVM/.artifacts/graphics/f02.4.4.1.5.2.3.3.1 \
  <temporary-private-root>
STAGING-CONTRACT: staging-only-unadmitted, 298 raw, 1462 derived, 0 payloads, 0 cutover-ready
```

The actual-identity prerequisite suite passed 30/30; the bound-scope suite passed 10/10; and the
independent-comparison suite passed 16/16. `cargo test -p emulator --test source_file_limits --quiet`
passed 6/6. `make test` exited 0: its emulator-unit portion passed 1,127 tests (3 ignored), all
boundary/source-limit/roadmap checks passed, and Node passed 337/337. No Rust, Wasm, or browser
source changed, so `make web-pkg` was not applicable. Remote CI was not run.

## Boundary and next work

This is a recorded-capture contract, not a fresh official build, a cache payload, a reusable cache
receipt, or a graphics compatibility/performance result. Private ownership blocks other-user cache
writers; a same-UID mutation after the final check cannot be retroactively prevented. Children
`.4.2`–`.4.4` must rehash their exact member/tree SHA-256 values immediately before marker
publication and every reuse, and reject any later mutation.

The staged source was executed immediately before `a7a41e1e`; that commit contains the exact
implementation and tests, while this receipt follows as documentation. First failing subcheck:
none. Next ready task: `F02.4.4.1.5.2.3.4.2` — stage and verify captured core inputs.
