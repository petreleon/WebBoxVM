# F02.4.4.1.5.2.3.5.2 blocker record — proof-level lineage trace

Revision: hosted witness source `cf1ae29e4885eb730c668d65a769cb22fcf1a543`
Validation: 26 focused positive/hostile/witness tests; sealed 298-member scope probe; Docker gate; GitHub-hosted direct primitive witness
Result: BLOCKED
Artifacts: historical ignored `...ptrace-probe-r2`; fresh `...ptrace-probe-r9` compiled, reported, then was removed
Remote receipt: [GitHub Actions run 34449710520](https://github.com/petreleon/WebBoxVM/actions/runs/34449710520), public check annotation
Profile: unadmitted Docs provenance evidence only; no source, output, cache, or consumer state changes

Task ID and date: F02.4.4.1.5.2.3.5.2, 2026-09-10 Europe/Bucharest.

This child adds a bounded normalized event grammar and a proof-only binder. A public receipt or verification re-runs
the pinned capture binder from its observation, artifact root, and run ID; it never accepts a caller-supplied scope.
Every raw read must then match the exact ID, selector, digest, and byte count from that 298-member capture. IDs are
reversible common raw/derived selector identities. A process instance inherits its parent's ordered dependency snapshot,
an `exec` creates a new instance for the same process ID, and an `exit` closes its lifecycle. A final derived member
requires one closed writer object, an exact temporary-to-generated lineage where applicable, a nonempty conservative
producer set, and a later exact derived read. The future trusted normalizer must emit the close and post-close hash/byte
facts. The resulting relation is an observed execution overapproximation, never an asserted minimal semantic graph.

The binder rejects absent producers, forged or duplicate-ID scopes, raw scope expansion or omission, unread finals,
duplicate writers, mutation after close, mismatched or escaping temporary/rename paths, pre-final derived reads, PID
reuse, events after `exec`, noncanonical IDs, stale receipts, and active states. It keeps the 200,000-event / 32 MiB
bounds and rejects producer cycles. The implementation is separate from the sealed `03-capture-core-closure` observer;
no new trace has been captured.

Focused contract command, run from this directory:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  lineage_test.py lineage_hostile_test.py lineage_probe_test.py lineage_github_witness_test.py -v
```

It passed 26/26 tests: three positive lineage/receipt checks, ten hostile scope/lifecycle/receipt checks, nine
Docker gate confinement/source-identity/decoder checks, and four hosted-witness anchor/blob/receipt checks. The
sealed capture probe separately reported `SCOPE: 298 raw, proof-lineage-only-unadmitted`. Current source files are
at most 145 physical lines.

Repository integration also ran `make test` successfully: 1,151 Rust tests passed, 3 were ignored, and 337 Node
tests passed with no failures. This proof-only task does not change Rust, Wasm, browser, guest, or API behavior.

Run that explicit capture anchor from this directory with:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 lineage_capture.py \
  ../../03-capture-core-closure/01-observe-pinned-build-inputs/vulkan_docs_core_input_observation.json \
  /Users/petreleon/code/WebBoxVM/.artifacts/graphics/f02.4.4.1.5.2.3.3.1 observer-a
# SCOPE: 298 raw, proof-lineage-only-unadmitted
```

The capability probe compiles only a reviewed, hash-checked C file in the pinned image; it does not mount a Docs
source or start a Docs build. Its C diagnostic tests decoded `getpid` entry/exit plus fork/exec and clean exits, but
the Python boundary rejects every `available` result: daemon path mounts cannot bind what the daemon compiled or ran
against a same-UID swap. It retains `--network none`, `--platform linux/amd64`, and `--user 501:20`, mounts that one
C file read-only, detects source/work-root replacement at checkpoints, and omits `--privileged`, `--cap-add`, and
`--security-opt`.

```sh
PYTHONDONTWRITEBYTECODE=1 python3 lineage_probe.py \
  --work /Users/petreleon/code/WebBoxVM/.artifacts/graphics/f02.4.4.1.5.2.3.5.2-ptrace-probe-r9
# CAPABILITY: BLOCKED child-traceme errno=38
```

The `-r9` pathname did not exist before invocation; the gate created it only for that one probe. It was removed after
this reproduction. The command exited 77 by design after the child `PTRACE_TRACEME` gate returned errno 38 (`ENOSYS`).
This is the first failing subcheck for the tested pinned-image/no-privilege route. Its Docker daemon path mount also
cannot issue a trusted positive receipt until inputs/client are anchored. The pinned image has `gcc` but no `strace`.

## Hosted primitive witness

Run 34449710520 succeeded on GitHub-hosted Ubuntu x64. Its immutable-blob receipt binds commit `cf1ae29e...`, C blob
`a510acda419260f6e5cca12523f3a7a8c807b155`, SHA-256
`76c5fd93ca9c56fd48d8f02a037e87fa3c0d72ff13ba360ffcae44203ff9da89`, 6,700 bytes, Linux
`6.17.0-1022-azure x86_64`, and `/usr/bin/x86_64-linux-gnu-gcc-13` 13.3.0. The exact C gate reached
`parent-fork-exec-complete`, returning its deliberately fail-closed `{status:"blocked",errno:0}` terminal record.
The workflow reads both helper and C from `GITHUB_SHA` Git blobs, not the checkout paths, and emits only
`observed-unadmitted`; it never converts that record into `available` or a lineage receipt.

This proves only the direct-host child/getpid/fork/exec primitive under GitHub runner isolation. It assumes the
provider, `git`, `gcc`, private temporary root, and no hostile same-UID peer; it has no pinned Docs image, network
confinement, actual Docs build, durable artifact attestation, or protected branch. It therefore cannot discharge
the full collector, two fresh captures, derived `producer_input_ids`, or any parent checkbox.

Do not replace this gate with `LD_PRELOAD`: it can miss direct syscalls and static executables, lacks reliable
parent/exec lineage, and its writable trace file could be target-controlled. Do not relax Docker confinement to pass.
No two fresh lineage captures exist, so no `producer_input_ids`, actual closure manifest, staging marker, inventory,
candidate, guest API, browser, CTS, conformance, or performance claim changes here.

The historical `-r2` workspace artifact was only a compiled disposable probe; its tracked source and a fresh-root
reproduction compile anew. This child and every parent checkbox remain unchecked pending an authorized, equally
trustworthy trace facility that preserves the existing container restrictions.
