# F03.3.2.2.2 evidence

Revision: bf3b3ec96f8433061ed0df80f342c0b2c3ff6514
Validation: local focused and repository gates; no remote CI run was created
Result: PASS
Artifacts: `gles_declaration_grammar.json` (`7de694c6169331689f8ea48127cfbc80b5dbc96fc8c74e10332267503ee711cf`)
Profile: GLES 3.2 declaration spelling only; raw-only and promotion disabled

Task ID and date: F03.3.2.2.2, 2026-09-21.

Tested commit and patch: parent `e884fa453eb46b52e1ba21f8388bdcbc653347a0`;
the tested staged tree became `bf3b3ec9`; its patch SHA-256 is
`1202ec0b016e212dae97d5397f20f9493340227871624009c4ae749d201579e6`.

Source identity: sealed GLES 3.2 PDF, 601 physical pages, 2,198,754 bytes,
SHA-256 `5028bd55b9ed7072757944f117a682ff3a0d09ab7b7a9a09cd144b7928db661c`;
authority revision `1cdd228e34966dd6b95bd203e9f84faba0f371a1`. Guest image/build,
browser/adapter/driver: not applicable to this source-only task.

The artifact distinguishes an empty document prefix from C prefix `gl`; it has five
base notation anchors and 27 unique sealed-PDF `void NAME{...}` templates in source
order. Their witnesses span physical pages 27, 126, 128–129, 160, 162, 206,
209–210, 283, and 420; this records spelling expansion only, not command facts.

Reproduce from repository root:

```sh
make graphics-gles-declaration-grammar-test
python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/03-gles/02-api-inventory/02-command-object-state-raw-inventory/02-template-declaration-grammar/gles_declaration_grammar.py --cache-root /private/tmp/webboxvm-f0341.cqT6ZX
make test
cargo test -p emulator --test source_file_limits --quiet
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
```

Expected/actual: focused tests passed 4/4; the CLI passed `5 base anchors and 27
formal GLES templates`; full `make test` passed, including 1,127 Rust unit tests
and 338 web tests. Source limits passed 6/6; diff and roadmap checks passed.

Negative checks reject guessed or unexpanded templates, a prefixed/lowercase name,
duplicates, registry-shaped forms, stale/promotion-rehashed JSON, a missing cache,
and a changed section witness. The complete source catalog is self-hashed; no
external capture, native/browser run, fallback, or performance protocol applies.

Decision and limits: this grammar establishes no GLES implementation, state
behavior, support, conformance, certification, guest/browser result, or performance.
It remains raw-only, with no Matrix rows or CTS execution.

Commit/push verification: `bf3b3ec9` was pushed to
`origin/codex/graphics-f01-baseline`; remote SHA matched. `gh run list --commit
bf3b3ec9 --limit 10` returned no runs.

Next ready task: F03.3.2.2.3.
