# F03.2.2.5.3.2 evidence

Revision: 827d7a9f6f26d2243a21133f490f7cc7d5c19866
Validation: local focused and repository gates; no remote CI run was created
Result: PASS
Artifacts: `opengl_buffer_command_inventory.json` (`12b34844f8b938fbe8b7e7c41f2a3fc7f307850d0500c9b14988511a20192074`) and five bound fragments
Profile: OpenGL 4.6 core buffer command declaration inventory only; source-only

Task ID and date: F03.2.2.5.3.2, 2026-09-21.

Tested commit and patch: parent `30702dc6435dc0207ae307ecc51495a91e149363`;
the tested tree became `827d7a9f`; its patch SHA-256 is
`7b78315f659be7e1c060ad81b1935e98ea044243c3ed452290e4a7853a7af83d`.

Source identity: sealed OpenGL 4.6 core PDF, 851 physical pages, SHA-256
`a6f65e58cd8294188dc4d5cf9d2d581468f8f2e2282101149e14083d75ea9bee`;
authority revision `1cdd228e34966dd6b95bd203e9f84faba0f371a1`. Guest image/build,
browser/adapter/driver: not applicable to this source-only task.

The inventory covers §§6.1–6.7 (physical pages 81–105): 39 formal source
declarations produce 38 raw command entries in binding, storage, mapping, transfer,
and query fragments. `CreateBuffers` is explicitly excluded because it is baseline-owned;
the pp. 85–86 `BindBuffersBase` span keeps its canonical F03.2.1-valid page-85 locator.
§6.4/page 101 is bound explicitly as a zero-declaration window. Declaration spellings,
including the source's `bitfield acesss`, are retained literally.

Reproduce from repository root:

```sh
make graphics-opengl-buffer-command-inventory-test
python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/02-opengl-core/02-command-object-state-inventory/05-remaining-command-object-vocabulary/03-object-resource-command-slices/02-buffer-commands/opengl_buffer_command_inventory.py --cache-root /private/tmp/webboxvm-f0341.cqT6ZX
make test
cargo test -p emulator --test source_file_limits --quiet
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
```

Expected/actual: focused tests passed 5/5; the CLI passed `38 buffer command raw
declarations; source-only`; `make test` passed, including 1,127 Rust unit tests and
338 web tests. Source limits passed 6/6; diff and roadmap checks passed.

Negative checks reject altered PDF declarations or span anchors, non-canonical locator
forms, classifier/state binding changes, rehashed promotion or partial fragments,
duplicate JSON fields, symlinked artifacts, and preloaded private dependency aliases.
The root receipt binds exact serialized fragment bytes and self-hashed contents.

Raw captures: no external capture was retained; the sealed cache, exact locators,
state/baseline receipts, root receipt, and five fragment receipts reproduce this bounded
declaration result. No native/browser run, software fallback, or performance protocol applies.

Decision and limits: this is not buffer lifetime, binding, mapping, visibility, or
numeric-property behavior evidence. It asserts no implementation/API support,
guest/browser behavior, conformance, certification, or performance result.

Commit/push verification: `827d7a9f` was pushed to
`origin/codex/graphics-f01-baseline`; remote SHA matched. `gh run list --commit
827d7a9f --limit 10` returned no runs.

Next ready tasks: F03.2.2.5.3.3, F03.2.2.5.3.4, F03.2.2.5.3.5, and F03.2.2.5.3.6.
