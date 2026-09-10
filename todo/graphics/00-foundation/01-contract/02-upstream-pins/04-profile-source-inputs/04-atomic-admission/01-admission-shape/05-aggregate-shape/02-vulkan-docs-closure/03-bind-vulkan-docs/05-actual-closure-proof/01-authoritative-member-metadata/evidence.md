# F02.4.4.1.5.2.3.5.1 evidence

Revision: tested atop `af430ce647621edc9eb8113a4bc6a49875b005b6`
Validation: 6 hermetic fail-closed metadata tests; exact live snapshot/CLI blocker probe; source-limit, roadmap, and diff checks
Result: BLOCKED
Artifacts: ignored `.artifacts/graphics/f02.4.4.1.5.2.3.3.1`, replayable from F02.4.4.1.5.2.3.3 evidence
Profile: `vulkan-1.4-core`, source-closure metadata only; all states remain unadmitted and not cutover-ready

## Result

The evaluator reifies all 298 raw members only after it binds the sealed observer capture, checks that the source
worktree is clean, and snapshots the pinned commit through its immutable Git object database. It enumerates the
commit tree and reads blobs only: it never reopens a worktree path after eligibility. The snapshot must reproduce the
reviewed full-tree digest before selected bytes, SPDX, and `REUSE.toml` are interpreted. It creates reversible,
collision-free IDs and logical cache keys. A cache key is a namespace spelling, not a claim that bytes currently
reside in an external cache.

The first unproven static derived field is `license`: zero of 1,462 generated members has an SPDX header, and no
pinned per-member generated-license authority exists. The same missing authority prevents derived role and
provenance labels. This receipt deliberately does not generate a closure or fill any of those fields.

## Raw authority

- 171 selected raw files have direct SPDX headers: 130 `CC-BY-4.0`, 39 `Apache-2.0`, one `MIT`, and one
  `Apache-2.0 OR MIT`.
- The remaining 127 are covered by pinned aggregate `REUSE.toml` annotations: 42 image SVG files and one docinfo
  header are `CC-BY-4.0`; 81 KaTeX files and `config/khronos.css` are `MIT`; `package.json` is `Apache-2.0`; and
  `config/copyright-spec.adoc` is `LicenseRef-KhronosSpecCopyright`.
- Thus the reified total is 173 `CC-BY-4.0`, 40 `Apache-2.0`, 83 `MIT`, one dual license, and one Khronos-spec
  license. The reviewed root remains the exact pre-existing `vulkan-14-spec` identity.

For non-root raw members, the role/provenance strings are explicitly this evaluator's structural vocabulary:
`captured raw Docs source input` and a pinned-tree/commit/selector record. They do not assert upstream authorship.
The root uses its already reviewed exact role and provenance. The source-tree digest binds `REUSE.toml`, even though
that metadata file was not itself a runtime build input.

## Reproduction

From this directory, run the focused suite and exact blocker probe:

```text
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest test_vulkan_docs_member_metadata.py -v
PYTHONDONTWRITEBYTECODE=1 python3 vulkan_docs_member_metadata.py \
  ../../03-capture-core-closure/01-observe-pinned-build-inputs/vulkan_docs_core_input_observation.json \
  /Users/petreleon/code/WebBoxVM/.artifacts/graphics/f02.4.4.1.5.2.3.3.1 \
  /Users/petreleon/code/WebBoxVM/.artifacts/graphics/f02.4.4.1.5.2.3.3.1/sources/observer-a
```

From the repository root, run `cargo test -p emulator --test source_file_limits --quiet` and
`PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py`. Before staging, check each new file with
`git diff --no-index --check /dev/null FILE`: exit 1 is expected for an added file, so require no `--check`
diagnostics rather than treating that status as a clean verdict. The final staged integration uses
`git diff --cached --check`, which covers these new files.

The hermetic focused suite passed 6/6: raw SPDX/REUSE reification; mutated digest and source/output swap rejection;
ambiguous license evidence rejection; snapshot-only license interpretation; immutable-Git-blob root/component-swap
handling; and derived-authority/static-key rejection. The separate live CLI intentionally exited 2 after reporting:

```text
STATIC-METADATA: 298 raw, 1462 derived
BLOCKED: derived member license, role, provenance, and producer authority is absent
```

The source-file limit suite passed 6/6. The roadmap checker passed with 274 documents, 164 tasks, and 59 complete.
The final staged integration must use `git diff --cached --check`; an ordinary unstaged `git diff --check` would not
cover this evaluator or its test because they were newly added. Remote CI was not run.

Repository integration also ran `make test` successfully: 1,151 Rust tests passed, 3 were ignored, and 337 Node
tests passed with no failures. This static reifier does not change Rust, Wasm, browser, guest, or API behavior.

## First blocker and limits

`COPYING.adoc` states that transient generated files may have no copyrights. Neither the generated tree nor the
sealed read trace supplies a per-member license authority, so assigning one universal license would be fabricated.
No current input or output manifest supplies an authoritative derived role/provenance mapping either.
`producer_input_ids` remains a separate lineage obligation for F02.4.4.1.5.2.3.5.2 and is not inferred here.

No cache was created, no payload was admitted, and no root/output/inventory/support claim changed. A future
authorized policy must provide an exact source and applicability for every derived static field before the lineage
and closure children may consume it. Parent `.5` must remain BLOCKED with this child checkbox open.

Next structured task: F02.4.4.1.5.2.3.5.2 has its own tracer blocker record; no later closure PASS may fill this
static blocker.
