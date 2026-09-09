# Working one small graphics task

[Roadmap](README.md) · [Receipt template](evidence-template.md) · [Baseline](baseline.md)

This roadmap is a plan, not evidence that any feature works. All implementation
checkboxes start empty. Old implementations may be reused after current verification.

## Choose work

1. Read this file, then run `python3 scripts/check_graphics_roadmap.py` from the repo root.
2. Open the first ready leaf in the relevant lane. Follow its `Depends` IDs and linked
   prerequisite receipts. Folder numbers describe organization, not a global serial order.
3. Check `git status --short` and read the listed starting points and pinned sources.
   Keep pre-existing edits intact. Record the baseline revision and dirty diff hash.
4. Work on one behavior per coherent commit. Different workers may own independent
   ready leaves with disjoint files; agree ownership before editing shared modules.

## Make the task executable

Each leaf provides an outcome, dependencies, starting files, a small checklist and
acceptance requirements. The shared F05 runner adds exact per-task commands as tests
are implemented. Before F05 exists, use the existing commands in the baseline document.

Future harnesses and proposed output paths are deliverables, not commands that already
exist. Before implementing a feature, record its exact focused test command, fixtures,
expected result and nonzero test count in the local receipt. A matching test name
alone is insufficient; inspect what the assertion actually proves.

For a research/design task, the checked result is a reproducible probe or decision
document. A discovered mandatory blocker cannot complete an implementation task.
For an optional optimization experiment, rejection is a valid recorded experiment
outcome after measurement and removal of the failed change. It never completes an
unmet compatibility or performance requirement.

## Split a task again when necessary

Split before implementation when the leaf includes independent API feature families,
requires unresolved architecture choices, or will exceed one understandable commit.
Do not cram many unrelated behaviors into the existing checklist.

1. Keep the existing task folder and ID as a parent. Preserve its outcome,
   prerequisites, starting points and final verification contract.
2. Create 2–6 immediate child folders, each containing `README.md`. Use descriptive
   names such as `01-layout/`, `02-commands/` and `03-negative-cases/`.
3. Give children stable IDs such as `S03.1`, `S03.2`, `S03.3`; keep the same leaf fields
   and at most six checkboxes. Each may contain further subfolders when necessary.
4. Replace the parent's action checkboxes with unchecked links to those child lists.
   Move all obligations into a child or retain them in the parent's verification.
5. Children inherit necessary external prerequisites, not a dependency on their own
   parent. Siblings may depend on earlier children. Downstream tasks continue to depend
   on the original parent ID, which stays open until all children and final checks pass.
6. Re-run the roadmap checker. Attach an aggregate receipt before completing the parent.

Every list has a `README.md` entry point. Keep 2–6 children per ordinary list, at most
eight phase links in the root, and at most 180 physical lines in every maintained file.
Split by responsibility; minification or giant generated lines do not satisfy the intent.

## Verify and record

1. Run the leaf's focused positive, boundary and negative checks. Add independent
   reference comparisons where specified; do not make tests merely mirror the code.
2. For changed GPU code, run `make test`, the source-file limit check and
   `git diff --check`. For browser/Rust-Wasm integration changes also run `make web-pkg`.
   Run the required real guest/browser lanes when behavior crosses those boundaries.
3. Identify the exact first failing subcheck. A missing asset, browser or comparable
   native GPU is a blocked check, never PASS. Keep dependent tasks unchecked.
4. Complete `evidence.md` from the receipt template with commands, revision, hashes,
   observations, expected/actual counts and artifacts. Keep the small receipt in Git;
   large captures belong in `.artifacts/graphics/` with hashes and durable retrieval
   or reproduction instructions. Do not publish credentials or unrelated local data.
5. Set the leaf's `Evidence:` field to `[receipt](evidence.md)`. Check actions only when
   they are true, then the verification/receipt checkbox after inspecting the output.
6. Update immediate parent checkboxes up to the root. A parent is complete only when
   every child is complete and its own acceptance is met. Run the roadmap checker.

The checker validates structure and presence of receipts; it cannot certify that a
logged command passed, an image is correct or a performance comparison is fair.
Review the actual evidence before checking anything complete.

## Commit, push and hand off during execution

Make a focused commit after each verified leaf or coherent child task. Include only
owned implementation, tests, evidence and task-status changes; inspect staged diffs.
Use the existing task branch or a descriptive `codex/` branch when isolation is needed.
Never stage unrelated work, discard user changes or force-push to resolve divergence.

Push regularly after green required checks and verify the remote branch SHA. Record
the implemented revision in its receipt; if the receipt follows in another commit,
identify the tested feature revision accurately. Remote CI completion, when configured,
must be reported separately from local checks. On a push failure, preserve the local
commit, state the concrete blocker and continue independent authorized work.

Before handoff, state completed IDs, tested/pushed revisions, the next ready leaf and
any blocker. The overall objective stays open while a mandatory compatibility or
performance gate is unresolved. No roadmap entry authorizes silently narrowing it.
