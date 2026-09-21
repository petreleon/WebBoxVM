# F03.2.2.5.3.1 evidence

Revision: ddc3ba85acc3d3a459fe50a3453f9c023101e982
Validation: local focused and repository gates; no remote CI run was created
Result: PASS
Artifacts: `opengl_generic_object_sync_raw_inventory.json` (`ec1713ad1cdec538fb001ddccdfefe9fceaab71679bff5cff77c3f813758ea85`)
Profile: OpenGL 4.6 core raw declarations only; Matrix incomplete

Task ID and date: F03.2.2.5.3.1, 2026-09-21.

Tested commit and patch: parent `fa6572aeb20eb33f0fc36d2ff1c2fac308dc6b12`;
the tested staged tree became `ddc3ba85`; its patch SHA-256 is
`c60a30733e1062038d7f4876949eec9083bf49890f5e2a351e04aaad5381f2df`.

Source identity: sealed OpenGL 4.6 core PDF, 851 physical pages, 3,003,752 bytes,
SHA-256 `a6f65e58cd8294188dc4d5cf9d2d581468f8f2e2282101149e14083d75ea9bee`;
authority revision `1cdd228e34966dd6b95bd203e9f84faba0f371a1`. Guest image/build,
browser/adapter/driver: not applicable to this source-only task.

The source window records zero formal declarations in §2.6.1 on pages 50–51 and
25 in §§4.1–4.3 on pages 58–73. It excludes exactly baseline `glFenceSync` and
`glCreateQueries`, retaining 23 raw declarations. Thirteen query declarations defer
returned-state handling to F03.2.2.3; no returned state fact is recorded here.

Reproduce from repository root:

```sh
make graphics-opengl-generic-object-sync-test
python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/02-opengl-core/02-command-object-state-inventory/05-remaining-command-object-vocabulary/03-object-resource-command-slices/01-generic-object-sync/opengl_generic_object_sync_raw_inventory.py --cache-root /private/tmp/webboxvm-f0341.cqT6ZX
make test
cargo test -p emulator --test source_file_limits --quiet
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
```

Expected/actual: focused tests passed 4/4; the CLI passed `23 generic object/sync
raw declarations; source-only`. Full `make test` passed, including 1,127 Rust unit
tests and 338 web tests. Source limits passed 6/6; diff and roadmap checks passed.

Negative checks reject a missing declaration or PDF page, altered/reordered fragments,
wrong baseline exclusion, cross-profile artifact, promotion-shaped field, stale hash,
and a shadowed private module. The root and four fragment receipts reproduce the
bounded result without retaining a raw external capture.

Decision and limits: this is declaration provenance only. Claims remain false and
Matrix/CTS/semantic-fact counts are zero; it establishes no object, synchronization,
query-result, support, conformance, guest/browser, certification, or performance fact.

Commit/push verification: `ddc3ba85` was pushed to
`origin/codex/graphics-f01-baseline`; remote SHA matched. `gh run list --commit
ddc3ba85 --limit 10` returned no runs.

Next ready task: F03.2.2.5.3.2.
