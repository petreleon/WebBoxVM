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

- [ ] Freeze an event grammar for process instance, parent/exec, ordered in-scope reads, writes/finalization, and rename.
- [ ] Implement a bounded syscall-level collector with a capability gate and self-tests, split into files of at most 180 lines.
- [ ] Bind each final generated input to exactly one writer plus its own and causal-ancestor reads before child start.
- [ ] Reject absent/multiple writers, unsafe temporary or renamed paths, mutation, cycles, missing producers, and path escape.
- [ ] Preserve source/image/network confinement; record BLOCKED if tracing needs privilege or a Docker-policy relaxation.

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
