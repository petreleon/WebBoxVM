# F02.4.4.1.5.2.3.5.2 — Proof-level lineage trace

[Parent task](../README.md) · [Worker instructions](../../../../../../../../../../../workflow.md)

Task: F02.4.4.1.5.2.3.5.2
Depends: F02.4.4.1.5.2.3.3.2, F02.4.4.1.5.2.3.3.3
Evidence: [blocker record](evidence.md)

Prerequisite lists: the [sealed read observer](../../03-capture-core-closure/01-observe-pinned-build-inputs/README.md),
[actual Docs grammar](../../02-actual-closure-identity/README.md), and the [blocker record](../evidence.md).

## Outcome

Specify, then build when the capability gate permits, a separate proof-only lineage collector that can bind each final
derived input to a successful writer and a conservative, ordered observed dependency set, without modifying the
sealed read-only observer.

## Starting points

- [current input trace](../../03-capture-core-closure/01-observe-pinned-build-inputs/observer/input_trace.c)
- [pinned replay plan](../../03-capture-core-closure/01-observe-pinned-build-inputs/vulkan_docs_observer_plan.py)
- [derived-member rule](../../02-actual-closure-identity/vulkan_docs_identity_scope.py)

## Checklist

- [x] [F02.4.4.1.5.2.3.5.2.1 — Freeze the proof-only grammar and binder](01-grammar-and-binder/README.md)
- [x] [F02.4.4.1.5.2.3.5.2.2 — Validate the bounded ptrace capability primitive](02-capability-primitive/README.md)
- [ ] [F02.4.4.1.5.2.3.5.2.3 — Collect a real synthetic syscall fixture](03-synthetic-syscall-collector/README.md)
- [ ] [F02.4.4.1.5.2.3.5.2.4 — Witness the synthetic collector on hosted Docker](04-hosted-synthetic-witness/README.md)
- [ ] [F02.4.4.1.5.2.3.5.2.5 — Bind the collector to an authorized Docs replay](05-docs-collector-binding/README.md)

## Verification

- `producer_input_ids` is a conservative observed-process dependency set, never inferred minimal semantic dataflow.
- Public proof receipts rebind the sealed capture from paths/run ID, and all raw/derived IDs must be reversible from
  their selectors; a manually constructed scope is never receipt authority.
- Do not add `--privileged`, `SYS_PTRACE`, seccomp relaxation, writable source, or an active consumer to obtain a trace.
- Status: **BLOCKED**. The same pinned image and no-privilege security flags return `ENOSYS` for child
  `PTRACE_TRACEME`; this gate deliberately does not reproduce a full Docs mount/build topology. The bounded parser
  and hostile tests remain proof-only scaffolding. Docker daemon path mounts also cannot issue a trusted positive
  capability result without an anchored execution client. GitHub run
  [`34449710520`](https://github.com/petreleon/WebBoxVM/actions/runs/34449710520) separately observed the reviewed
  direct-host primitive from immutable Git blobs, but it is `observed-unadmitted`: it is neither the pinned image,
  network-confined, nor a Docs-build trace. It trusts GitHub job isolation, `git`, `gcc`, and no hostile same-UID
  peer; the unprotected branch and unattested log prevent it from becoming lineage or closure authority.
- The separate GitHub-hosted pinned-image Docker primitive witness is intentionally narrower than a Docs build: it
  has an empty read-only `/vulkan`, a temporary `/work`, and only the reviewed C gate over stdin. It neither mounts
  Docs nor sets deterministic Docs inputs or runs the Docs argv. It validates the source-envelope byte count and
  SHA-256 inside the container before compilation, uses `--network none` and `--pull=never` after its pinned-image
  preflight, and fails rather than emits a receipt if Docker cannot produce the terminal primitive record. Even an
  observed result is only a capability prerequisite; it cannot establish a collector trace, Docs provenance, or
  closure, and still trusts the Docker daemon plus the existing GitHub-run limitations.
- Run [34453881443](https://github.com/petreleon/WebBoxVM/actions/runs/34453881443) observed that bounded primitive:
  the reviewed C gate reached `parent-fork-exec-complete` with `errno: 0` and outcome
  `pinned-image-ptrace-observed`, while its receipt remained `observed-unadmitted`. It does **not** mean `ptrace` is
  available to a Docs collector: no Docs source/build/argv was present, no lineage was captured, and the Docker
  daemon, unprotected branch, and unattested GitHub log remain outside the authority model. F02 and every parent
  task remain **BLOCKED**.

## Split rationale

The existing grammar/binder and narrow capability gate are separately verifiable, but a real syscall collector has
independent wire, path/fd-state, normalization, hosted-fixture, and authorized-Docs-replay responsibilities. The
split prevents a synthetic trace from being misrepresented as a Docs capture: only `.3` and `.4` may prove a
collector mechanism, while `.5` remains dependent on authoritative per-member metadata. This parent stays open
until all five children and the original proof requirements are satisfied.
