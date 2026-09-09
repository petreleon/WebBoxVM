# F03.1 evidence

Revision: `12a420f3b7469f6e2fb131cb971b0b5887544359`
Validation: focused 15-case contract suite; expected blocked live gate; local full, limit, roadmap, and whitespace gates
Result: PASS
Artifacts: lock-bound `profile_scope.json`, `source_requirements.json`, reusable validators, and the six-ID blocked-gate output
Profile: planning-only targets OpenGL 4.6 core, GLES 3.2, Vulkan 1.4 core; all remain blocked, with no runtime claim

Task ID and date: F03.1, 2026-09-09 Europe/Bucharest.

Tested revision: `12a420f3b7469f6e2fb131cb971b0b5887544359`, based on F02 inventory lock
`08be83edf7949e0d406bd4d3b9827b6abbd91d0312e3e08fcc74b87f6786e1a6`.

## Decision and boundary

This is a design/provenance gate, not an API implementation. It fixes the intended final scopes:
OpenGL 4.6 core only, GLES 3.2 only, and Vulkan 1.4 core only. Compatibility profiles and optional
extensions are not promised. Vulkan WSI and portability remain separately recorded scope, and a
promoted extension is provenance rather than independently advertised support.

The contract requires a future matrix row to bind a profile-specific normative input and a distinct
CTS/must-pass input by ID, revision, and SHA-256; it also requires a concrete locator, owner, test
selector, status, blocker, and local `evidence.md`. It rejects stale locks, a substituted existing
input, missing roles, cache-name collisions, cross-profile inputs, source/test reuse, placeholders,
and malformed matrix JSON. These checks validate matrix structure/provenance only; no matrix row,
guest driver, Mesa context, browser adapter, conformance result, or performance result is supplied.

## Commands and observed results

Working directory: `/Users/petreleon/code/WebBoxVM`.

```sh
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/01-profile-scope/validate_profile_scope_test.py
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/03-feature-matrix/01-profile-scope/validate_profile_scope.py
PYTHONDONTWRITEBYTECODE=1 make test
PYTHONDONTWRITEBYTECODE=1 cargo test -p emulator --test source_file_limits --quiet
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
git diff --check
```

The focused suite passed 15 tests, failed 0. It includes valid blocked-row structure, stale lock,
substituted source, missing role, malformed matrix CLI, evidence placeholder, cross-profile source,
non-independent test source, and cached-module collision checks. The live gate exited 3 as designed:

```text
BLOCKED: missing required inventory inputs: opengl-46-core-spec, opengl-cts-manifest,
gles-32-spec, gles-cts-manifest, vulkan-14-spec, vulkan-cts-mustpass
```

This expected BLOCKED result is the completed audit result: it exposes F02.4 rather than lowering
a profile target or treating the existing GL 3.0 Piglit/Vulkan version-check samples as coverage.
`make test` passed locally (Rust and 337 Node tests with no reported failures); the source-file limit
suite passed 6/6. The roadmap and whitespace checks passed before the completion documentation.
The macOS `xcrun` FSEvents/cache warnings did not fail any test command.

## Artifacts and handoff

No external payload, browser run, guest image, hardware adapter probe, or remote CI result is claimed.
The code commit was pushed as `12a420f3b7469f6e2fb131cb971b0b5887544359` and verified with
`git ls-remote` on `origin/codex/graphics-f01-baseline`. F02 is reopened through F02.4 to review and
atomically admit the six missing immutable inputs; after that, the source gate can pass only to the
still-blocked `matrix-incomplete` state. The next source work is F02.4.1–F02.4.3, independently.
