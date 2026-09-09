# Graphics compatibility roadmap

Build real guest-visible VirGL/OpenGL/GLES and Venus/Vulkan compatibility, pursuing near-native
guest graphics performance in the browser. This is an execution plan; no implementation task is
pre-completed.

Start at F01 in the foundation list. Read the worker instructions, then choose a ready leaf from the
checker. Follow its prerequisites even when another folder appears earlier.

- [Worker instructions and recursive splitting](workflow.md)
- [Evidence receipt template](evidence-template.md)
- [Current code and verification commands](baseline.md)
- [Primary sources and feasibility gates](sources.md)
- [Replacement goal text](goal.md)

## Phase lists

- [ ] [Contract and reproducibility](00-foundation/README.md)
- [ ] [Shared GPU runtime](01-shared-runtime/README.md)
- [ ] [General shader translation](02-shaders/README.md)
- [ ] [VirGL, OpenGL and GLES](03-virgl-opengl/README.md)
- [ ] [Venus and Vulkan](04-venus-vulkan/README.md)
- [ ] [Real guests and independent validation](05-guest-validation/README.md)
- [ ] [Near-native guest graphics performance](06-performance/README.md)
- [ ] [Qualification and delivery](07-qualification/README.md)

## Acceptance contract

Draft final targets are OpenGL 4.6 core, GLES 3.2 and Vulkan 1.4 core over standard VirGL/Venus. F03
freezes the exact mandatory inventories and extension scope; F04 proves mappings or identifies
blockers. Earlier API bring-up is intermediate. A mandatory blocker does not authorize lowering the
goal.

Proposed near-native gates: at least 80% of same-GPU native throughput, p95 frame time at most 1.25
times native, and p95 input latency at most native plus one display refresh. P01 freezes workloads,
comparison rules and statistics before optimization. These are planning targets, not achieved
results.

Completion requires all mandatory API behavior, independent guest/browser evidence and every
required performance threshold. Private demos, software fallbacks and submit-only measurements
cannot establish these claims. Sources and detailed scope distinctions are linked above.

## Check the plan

```sh
python3 scripts/check_graphics_roadmap.py
```

The checker validates links, dependency IDs/cycles, receipts on completed tasks, parent/child status
and the 180-line limit. It prints ready task IDs. It checks bookkeeping, not whether implementation
evidence is truthful; a worker must inspect the tests and output.
