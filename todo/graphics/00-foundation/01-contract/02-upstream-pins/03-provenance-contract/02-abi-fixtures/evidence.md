# F02.3.2 evidence

Revision: `16cd943eb67fa3249850870f2abc079dbe27b971` baseline plus uncommitted F02.3.2 metadata
Validation: seven hermetic ABI-sidecar tests, six-record CLI check, source limits, `make test`, roadmap, and whitespace checks
Result: PASS
Artifacts: six JSON sidecars under `records/`, each bound to F02.1 inventory-lock SHA-256 `cb85958df7f8e2a9b6b749f6218621d6b24b1f6de5dc9f5be3468ab5fb7ddd5b`
Profile: provenance metadata only; no guest ABI, graphics API, rendering, or network behavior change

## Scope and provenance result

The selected maintained ABI surfaces are `guest/virgl-clear-demo/{uapi,kms,virgl}.h`,
`guest/webgpu-demo/uapi.h`, `emulator/src/devices/virtio_gpu/three_d/capset.rs`, and
`emulator/src/devices/virtio_gpu/tests/virgl_draw_fixture.rs`. Their adjacent F02.3.2 records bind
the raw-byte SHA-256 of F02.1's canonical inventory lock, exact IDs/digests/licenses, a manual review
command, `none` generator identity, and each current local artifact SHA-256.

All six are maintained `handwritten` adapters or fixtures; no record claims copied upstream bytes.
The Linux records bind the pinned `linux-virtio-gpu-uapi` input. The VirGL header and fixture bind
the pinned Mesa VirGL behavior and virglrenderer protocol inputs; the capset binds virglrenderer
protocol. `kms.h` records Linux virtio-gpu UAPI context only: it makes no claim that F02.1's
`virtio_gpu.h` is a byte-origin for the local DRM/KMS declarations. No current selected artifact is
a Venus generator output; that future output family remains outside this leaf.

`validate_abi_records.py` calls F02.3.1's generic validator for every exact sidecar name, then
requires the fixed artifact path, selected input-ID tuple, handwritten origin, and rehashed local
bytes. It rejects missing or unexpected records, any known-but-wrong input, stale inventory/input
metadata, dishonest copied origin, and a changed local artifact hash before acceptance.

## Commands and actual results

```sh
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/03-provenance-contract/02-abi-fixtures/validate_abi_records_test.py
PYTHONDONTWRITEBYTECODE=1 python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/03-provenance-contract/02-abi-fixtures/validate_abi_records.py
cargo test -p emulator --test source_file_limits --quiet
make test
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
git diff --check
```

The focused suite passed 7/7 and the CLI printed `PASS: 6 ABI provenance records`. Its negative
cases mutate only temporary sidecars, never runtime files: unknown input, known-but-wrong Venus
input, stale inventory digest, dishonest copied origin, changed output hash, and a missing record all
failed closed. Source limits passed 6/6. `make test` exited zero; its Node suite reported 337/337
passing with zero failures, cancellations, skips, and todos. Before F02.3.2 completion markers,
the roadmap checker passed 164 documents, 106 tasks, and 14 complete, with F02.3.2 ready; `git diff
--check` exited zero.

No network request, cache fetch, generated protocol output, guest execution, browser execution, or
ABI compatibility claim is made here. The sidecar hashes deliberately make a future local adapter
edit fail provenance validation until its review record is renewed. This leaf has no commit or push;
the concurrent F02.3.3 worktree changes were preserved.
