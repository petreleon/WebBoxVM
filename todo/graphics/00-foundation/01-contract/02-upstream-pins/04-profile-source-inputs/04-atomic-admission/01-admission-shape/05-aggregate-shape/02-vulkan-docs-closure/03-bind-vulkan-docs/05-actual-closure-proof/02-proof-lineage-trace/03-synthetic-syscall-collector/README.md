# F02.4.4.1.5.2.3.5.2.3 — Collect a real synthetic syscall fixture

[Parent task](../README.md)

Task: F02.4.4.1.5.2.3.5.2.3
Depends: F02.4.4.1.5.2.3.5.2.1, F02.4.4.1.5.2.3.5.2.2
Evidence: pending

## Outcome

A bounded syscall-level tracer observes a self-contained fixture's raw read, temporary write, close, rename, derived
read, fork, exec, and exit events without `LD_PRELOAD`, then normalizes only its collector-owned pipe into the frozen
proof grammar.

## Starting points

- [ptrace primitive](../observer/lineage_ptrace_probe.c)
- [bounded C fixture](../observer/lineage_ptrace_fixture.c)
- [bounded C collector](../observer/lineage_ptrace_fixture_collector.c)
- [fixture wire normalizer](../lineage_synthetic_normalize.py)
- [collector-pipe binder](../lineage_synthetic_collect.py)
- [event grammar](../lineage_events.py)
- [binder](../lineage_bind.py)

## Checklist

- [ ] Trace only a collector-spawned fixture and descendants with bounded process, fd, path, and event state.
- [ ] Emit wire records solely through the collector pipe and map only `/vulkan`, `/work/temporary`, and `/work/generated`.
- [ ] Normalize one successful writer/close/rename/final/derived-read sequence into the proof grammar and binder.
- [ ] Reject partial, escaped, duplicated, mutated, malformed, or lifecycle-invalid fixture traces in focused tests.

## Verification

- No Docs source or build is mounted or invoked; a fixture result remains `observed-unadmitted`.
- The fixture accepts only a three-byte raw input. The collector captures it while the fixture is stopped, requires the
  same raw identity after every tracee exits, and only then emits matching raw/output identities through its own pipe.
  This is not a Docs snapshot or provenance assertion.
- `argv_sha256` is a static fixture-profile sentinel required by the frozen grammar, not an observed Docs argv hash.
- Root paths require `O_NOFOLLOW` and exact fixture flags; aliases or alternate I/O through a tracked descriptor fail closed.
- The local Docker runner mounts an unpinned worktree source and is diagnostic only; it cannot become hosted authority.
- The collector must require the existing no-privilege, no-network policy and fail closed on a missing trace primitive.
