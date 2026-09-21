# F03.3.2.2.3.1 evidence

Revision: ddc3ba85acc3d3a459fe50a3453f9c023101e982
Validation: local focused and repository gates; no remote CI run was created
Result: PASS
Artifacts: `gles_generic_sync_query_raw_inventory.json` (`ca43e662ada91a2c72a0265e500f2055dbec09da30defcd432ec3ced04815013`)
Profile: GLES 3.2 raw declarations only; promotion disabled

Task ID and date: F03.3.2.2.3.1, 2026-09-21.

Tested commit and patch: parent `fa6572aeb20eb33f0fc36d2ff1c2fac308dc6b12`;
the tested staged tree became `ddc3ba85`; its patch SHA-256 is
`c60a30733e1062038d7f4876949eec9083bf49890f5e2a351e04aaad5381f2df`.

Source identity: sealed GLES 3.2 PDF, 601 physical pages, 2,198,754 bytes,
SHA-256 `5028bd55b9ed7072757944f117a682ff3a0d09ab7b7a9a09cd144b7928db661c`;
authority revision `1cdd228e34966dd6b95bd203e9f84faba0f371a1`. Guest image/build,
browser/adapter/driver: not applicable to this source-only task.

The closed raw set has 19 source-ordered declarations: four generic-context, six
sync, seven query, and two memory-barrier forms. It binds domain-family orders
8/12/13/16 and the sealed grammar's empty document prefix plus C `gl` prefix;
state, lifetime, ordering, result, limit, and format material are routed out.

Reproduce from repository root:

```sh
make graphics-gles-generic-sync-query-test
python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/03-object-resource-command-slices/01-generic-sync-query/gles_generic_sync_query_raw_inventory.py --cache-root /private/tmp/webboxvm-f0341.cqT6ZX
make test
cargo test -p emulator --test source_file_limits --quiet
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
```

Expected/actual: focused tests passed 4/4; the CLI passed `19 bounded GLES
generic/sync/query declarations; source-only`. Full `make test` passed, including
1,127 Rust unit tests and 338 web tests. Source limits passed 6/6; diff and roadmap
checks passed.

Negative checks reject a missing literal or source witness, family/order drift,
prefix or cache substitution, duplicate/stale JSON, promoted facts, and a shadowed
private module. The self-hashed artifact records raw declarations only.

Decision and limits: this creates no API support, behavior, state, lifetime,
synchronization, conformance, guest/browser, certification, or performance claim.
It remains raw-only with promotion disabled and no Matrix or CTS execution.

Commit/push verification: `ddc3ba85` was pushed to
`origin/codex/graphics-f01-baseline`; remote SHA matched. `gh run list --commit
ddc3ba85 --limit 10` returned no runs.

Next ready task: F03.3.2.2.3.2.
