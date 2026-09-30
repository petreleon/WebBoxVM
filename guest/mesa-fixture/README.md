# Stock-Mesa graphics fixture

I01 prepares a bounded ARM64 guest using the unmodified F02 Mesa commit
`06f9e28304d5d3f109c33535c1c25b9df5769af2` (25.3.6). The recipe selects VirGL and
Venus only. This fixture has no software DRI/ICD fallback and does not manufacture
GPU completions. Source archives, packages, images and raw logs stay under
`.artifacts/graphics/i01-mesa-image/`.

The authoritative outcome and acceptance requirements remain in
[I01](../../todo/graphics/05-guest-validation/01-real-clients/01-guest-image/README.md).
Building binaries or passing host tests cannot complete real guest startup,
conformance or the final GL4.6/GLES3.2/Vulkan1.4 and performance gates.

## Commands

Run from the repository root:

```sh
make graphics-mesa-fixture-test
python3 guest/mesa-fixture/build/prepare.py
sh guest/mesa-fixture/build/build.sh
make graphics-mesa-image
make graphics-mesa-check
```

The build commands require the verified Mesa archive, native ARM64 Docker, and
the locked Debian snapshot. Preparation downloads and verifies the exact package
closure, then constructs an offline builder; compilation runs with networking
disabled. They preserve previous logs/build outputs and reject an accidental
overwrite. Fresh attempt names identify separate executions.

`graphics-mesa-image` automates kernel/module extraction, ARM64 client compilation
and atomic initramfs publication using the small scripts in `image/` and `probes/`.
Use `make graphics-mesa-image MESA_FIXTURE_ATTEMPT=02 MESA_FIXTURE_IMAGE=NEW_DIRECTORY`
for a separate fresh image; pass the same `MESA_FIXTURE_IMAGE=NEW_DIRECTORY` to
the guest targets. Failed stage logs and partial outputs are preserved.
Runtime file bytes and links must match the compiled manifest, and installed
Mesa files must match that verified runtime before and after client compilation.
Their exact observed results belong in the I01 receipt. A second clean source build is required before
claiming reproducibility of Mesa binaries. Equal initramfs builds from the same
verified inputs establish only deterministic assembly.

`graphics-mesa-guest-tools` tests boot, command execution and installed inputs.
`graphics-mesa-check` runs tools first and startup second, stopping at failure.
`graphics-mesa-guest-startup` requires both real driver startup markers, emitted
after actual API queries and cleanup. Either missing marker, a guest error or a
nonzero command status fails the check. The startup probes use minimum contexts
to locate the first unsupported operation; they retain the complete final goal.

The command runner uses fixed upper bounds: 900 seconds, 20 billion instructions,
2 million instructions per chunk, and 1 MiB UART. Its standalone CLI may reduce
these limits. The host wrapper has a 960-second process bound and kills its own
process group on expiration. Every invocation writes a fresh UART/log directory;
each directory retains its JSON receipt. The latest JSON result links and hashes
those retained artifacts. Both API probes run independently, retaining the first
nonzero status and requiring both success markers.

F05 registers `i01-stock-mesa-startup` as a neutral guest-fixture baseline:

```sh
python3 todo/graphics/00-foundation/02-reproducibility/02-check-runner/02-profile-bound-registration/profile_registration_run.py \
  --select i01-stock-mesa-startup \
  --result .artifacts/graphics/i01-mesa-image/f05-startup-result.json
```

Missing assets produce BLOCKED before execution. Existing assets with failed
driver startup produce FAIL. Neither result is evidence of API compatibility.
