# F03.2.2.5.2 evidence

Revision: bf3b3ec96f8433061ed0df80f342c0b2c3ff6514
Validation: local focused and repository gates; no remote CI run was created
Result: PASS
Artifacts: `opengl_declaration_grammar.json` (`fd742314618af4b604e6dfdaf0fa997e6475355396233737ad8b18b39d3c9947`)
Profile: OpenGL 4.6 core declaration spelling only; Matrix incomplete

Task ID and date: F03.2.2.5.2, 2026-09-21.

Tested commit and patch: parent `e884fa453eb46b52e1ba21f8388bdcbc653347a0`;
the tested staged tree became `bf3b3ec9`; its patch SHA-256 is
`1202ec0b016e212dae97d5397f20f9493340227871624009c4ae749d201579e6`.

Source identity: sealed OpenGL 4.6 core PDF, 851 physical pages, 3,003,752 bytes,
SHA-256 `a6f65e58cd8294188dc4d5cf9d2d581468f8f2e2282101149e14083d75ea9bee`;
authority revision `1cdd228e34966dd6b95bd203e9f84faba0f371a1`. Guest image/build,
browser/adapter/driver: not applicable to this source-only task.

The rule set preserves the PDF distinction: the C-prefix paragraph on physical page
32 is witnessed as §2.1 and must precede the §2.2 heading; notation is anchored on
pages 33–34; square and non-square matrix notation is anchored at §7.6.1 on page
163. It records two literal examples, the eight `Uniform{1234}{if}` names, and a
bounded source-anchored expansion grammar; it is not an inventory.

Reproduce from repository root:

```sh
make graphics-opengl-declaration-grammar-test
python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/02-opengl-core/02-command-object-state-inventory/05-remaining-command-object-vocabulary/02-template-declaration-grammar/opengl_declaration_grammar.py --cache-root /private/tmp/webboxvm-f0341.cqT6ZX
make test
cargo test -p emulator --test source_file_limits --quiet
git diff --check
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
```

Expected/actual: focused tests passed 4/4; the CLI passed `2 literal examples and
8 formal template expansions`; full `make test` passed, including 1,127 Rust unit
tests and 338 web tests. Source limits passed 6/6; diff and roadmap checks passed.

Negative checks reject a misplaced C-prefix paragraph, missing section witness or
matrix anchor, guessed/reordered suffixes, unexpanded or duplicated declarations,
stale/rewritten JSON, cache absence, and a shadowed private module. No raw capture
was retained; the sealed cache and self-hashed artifact reproduce the result.

Decision and limits: this is only an extraction grammar. It makes no command,
behavior, implementation/support, conformance, certification, guest/browser, or
performance claim; claims remain false and Matrix/CTS counts remain zero.

Commit/push verification: `bf3b3ec9` was pushed to
`origin/codex/graphics-f01-baseline`; remote SHA matched. `gh run list --commit
bf3b3ec9 --limit 10` returned no runs.

Next ready task: F03.2.2.5.3.
