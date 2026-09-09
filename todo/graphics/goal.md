# Replacement goal text

Implement real guest-visible VirGL/OpenGL/GLES and Venus/Vulkan compatibility in
WebBoxVM, with **near-native guest graphics performance in the browser** as the
measured end-to-end objective. Execute the nested roadmap at
`todo/graphics/README.md`, following `todo/graphics/workflow.md` and each leaf's
dependencies, checklist, verification and evidence requirements. Start with F01,
then work ready tasks; split oversized tasks into small linked child lists in
subfolders, recursively when needed.

Use standard unmodified Mesa guest drivers and real guest applications. Freeze
the exact API profiles and mandatory feature inventory through F03/F04; the draft
final targets are OpenGL 4.6 core, GLES 3.2 and Vulkan 1.4 core. Earlier bring-up
versions are milestones. Advertise only implemented and tested behavior; keep
unsupported mandatory semantics as visible blockers rather than narrowing the goal.

Be imaginative about emulation efficiency: investigate persistent GPU resources,
dirty-range transfers, shader/pipeline caching, bounded worker queues, parallel
compilation, safe command batching, GPU-driven work and deferred readback. Preserve
guest memory coherence, ordering and API semantics. Retain optimizations only with
correctness evidence and measured benefit.

Make testing a required part of every graphics implementation task. Add meaningful
unit, protocol and integration tests alongside each feature, and a reproducing
regression test for each bug fix. Cover valid behavior, boundary conditions,
malformed input, resource lifetimes, memory coherence, synchronization and device
loss as applicable. Compare shader and rendering results against independent native
references; tests must verify API behavior rather than merely mirror implementation.

Run focused tests during development and `make test`, the 180-line source check,
`git diff --check` and the roadmap checker before committing completed code. For
Rust/Wasm or browser integration changes, also run `make web-pkg` and the relevant
real guest/browser tests with fresh serial and threaded builds. Require pinned
GL/GLES and Vulkan conformance suites, stock Mesa application tests, and repeated
same-GPU native-versus-browser performance tests for final acceptance. Record exact
commands, tested revisions, nonzero test counts, failures/skips and evidence for
each leaf. Missing tests, unavailable prerequisites or skipped mandatory cases do
not count as passing; fix regressions before marking work complete or pushing it
as verified. Keep local test results distinct from remote CI results.

Freeze the same-hardware native comparison protocol before optimizing. Use the P01
defaults unless explicitly revised before measurement: at least 80% of native
completed-work throughput, p95 frame time at most 1.25 times native, and p95 input
latency at most native plus one display refresh, for every required workload.
Include guest CPU, driver, translation, transfer, GPU completion and presentation.
Private demos, software fallbacks and submission microbenchmarks do not satisfy
compatibility or near-native performance acceptance.

Keep files and folders well organized and every maintained file at most 180 physical
lines. Commit and push regularly after verified, coherent tasks; preserve unrelated
work and verify pushed revisions. Update leaf evidence and parent checkboxes only
after their required checks pass. Run the roadmap checker throughout execution.
Complete the goal only after the final compatibility, real-browser application and
performance gates pass; report concrete blockers and continue independent work
without silently weakening the target.
