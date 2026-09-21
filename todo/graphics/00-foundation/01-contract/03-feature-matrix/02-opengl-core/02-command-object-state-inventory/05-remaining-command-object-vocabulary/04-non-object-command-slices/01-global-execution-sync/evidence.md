# F03.2.2.5.4.1 evidence

Revision: ddc3ba85acc3d3a459fe50a3453f9c023101e982
Validation: local focused and repository gates; no remote CI run was created
Result: PASS
Artifacts: `opengl_global_execution_sync.json` (`6676b99090a0dbc2cda0d2cc349677ee4dffc34429fd89c34bd255cac913e597`)
Profile: OpenGL 4.6 core declarations only; Matrix incomplete

Task ID and date: F03.2.2.5.4.1, 2026-09-21.

Tested commit and patch: parent `fa6572aeb20eb33f0fc36d2ff1c2fac308dc6b12`;
the tested staged tree became `ddc3ba85`; its patch SHA-256 is
`c60a30733e1062038d7f4876949eec9083bf49890f5e2a351e04aaad5381f2df`.

Source identity: sealed OpenGL 4.6 core PDF, 851 physical pages, 3,003,752 bytes,
SHA-256 `a6f65e58cd8294188dc4d5cf9d2d581468f8f2e2282101149e14083d75ea9bee`;
authority revision `1cdd228e34966dd6b95bd203e9f84faba0f371a1`. Guest image/build,
browser/adapter/driver: not applicable to this source-only task.

Six literal forms are sealed in source order: `GetError` p38 §2.3.1,
`GetGraphicsResetStatus` p41 §2.3.2, `Flush` p42 and `Finish` p43 §2.3.3,
then `MemoryBarrier` p183 and `MemoryBarrierByRegion` p187 §7.13.2. The p43 and
p187 declarations retain their respective earlier section-heading witnesses.

Reproduce from repository root:

```sh
make graphics-opengl-global-execution-sync-test
python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/02-opengl-core/02-command-object-state-inventory/05-remaining-command-object-vocabulary/04-non-object-command-slices/01-global-execution-sync/opengl_global_execution_sync.py --cache-root /private/tmp/webboxvm-f0341.cqT6ZX
make test
cargo test -p emulator --test source_file_limits --quiet
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
```

Expected/actual: focused tests passed 4/4; the CLI passed `6 source-only global
execution/synchronization declarations; matrix-incomplete`. Full `make test` passed,
including 1,127 Rust unit tests and 338 web tests. Source limits passed 6/6; diff
and roadmap checks passed.

Negative checks reject a guessed declaration, missing source text or section witness,
substituted classifier or grammar binding, stale/re-hashed JSON, a promotion-shaped
claim, a changed row, duplicate JSON fields, and a shadowed private module.

Decision and limits: this is only a literal declaration inventory. Claims remain
false and Matrix/CTS counts are zero; it makes no ordering, visibility, state,
synchronization behavior, support, conformance, guest/browser, certification, or
performance claim.

Commit/push verification: `ddc3ba85` was pushed to
`origin/codex/graphics-f01-baseline`; remote SHA matched. `gh run list --commit
ddc3ba85 --limit 10` returned no runs.

Next ready task: F03.2.2.5.4.2.
