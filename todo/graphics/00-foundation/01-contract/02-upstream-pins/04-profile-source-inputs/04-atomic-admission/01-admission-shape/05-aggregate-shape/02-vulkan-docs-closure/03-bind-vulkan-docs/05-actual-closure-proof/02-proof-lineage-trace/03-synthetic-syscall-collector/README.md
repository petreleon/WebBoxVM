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
- [fixture wire normalizer](../lineage_synthetic_normalize.py)
- [event grammar](../lineage_events.py)
- [binder](../lineage_bind.py)

## Checklist

- [ ] Trace only a collector-spawned fixture and descendants with bounded process, fd, path, and event state.
- [ ] Emit wire records solely through the collector pipe and map only `/vulkan`, `/work/temporary`, and `/work/generated`.
- [ ] Normalize one successful writer/close/rename/final/derived-read sequence into the proof grammar and binder.
- [ ] Reject partial, escaped, duplicated, mutated, malformed, or lifecycle-invalid fixture traces in focused tests.

## Verification

- No Docs source or build is mounted or invoked; a fixture result remains `observed-unadmitted`.
- The collector must require the existing no-privilege, no-network policy and fail closed on a missing trace primitive.
