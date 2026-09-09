# F06.3.2 evidence

Revision: `a6ab065` baseline; scoped worktree change
Validation: five hermetic real-entrypoint checker fixtures, source limits, `make test`, roadmap, diff
Result: PASS
Artifacts: `scripts/test_check_graphics_roadmap.py`; temporary fixture roots are removed after each case
Profile: roadmap-maintenance fixture only; no guest protocol, graphics API, browser, or performance claim

Task ID and date: F06.3.2, 2026-09-09.

The runner invokes `scripts/check_graphics_roadmap.py` in a new temporary root for every case; it
does not mock checker internals or rely on the checked-in roadmap as its positive example. The
valid case has an immediate nested child list, completed evidence receipts, and `Depends: none`
metadata. It passes the real entrypoint and reports `Ready: none`.

Four deliberately invalid roots assert the first stderr diagnostic: a missing local Markdown link,
a child list linked from two levels below its parent, a parent checkbox inconsistent with a completed
child, and a 181-line maintained file. The focused runner executes five cases (one valid and four
invalid), exits zero only when every expected diagnostic matches, and is 117 physical lines.

Commands, working directory, and actual result:

```sh
python3 scripts/test_check_graphics_roadmap.py
cargo test -p emulator --test source_file_limits --quiet
make test
python3 scripts/check_graphics_roadmap.py
git diff --check
```

Working directory: `/Users/petreleon/code/WebBoxVM`.

The focused suite passed 5/5. The source-limit suite passed 6/6. `make test`, the roadmap checker,
and `git diff --check` exited zero after the completion metadata was updated. Temporary fixture
trees have random OS-provided paths and are removed when each test returns; no generated artifact
is retained. No browser adapter, guest workload, native reference, fallback route, Mesa,
OpenGL/GLES, Vulkan, or performance behavior is exercised here.

There was one fixture-authoring failure before final validation: the malformed-depth task initially
omitted its local receipt, so the real checker correctly reported that broken link first. Adding the
fixture-only receipt restored the intended depth diagnostic; no production checker behavior changed.

Commit/push verification: pending final scoped commit and remote SHA verification.
No remote CI result is claimed locally. Next ready task: F06.3.3, once this receipt is committed and
the parent structure is rechecked.
