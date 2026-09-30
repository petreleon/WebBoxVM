# I01 stock-Mesa input capture

Revision: `5e9561d3975536f9e5090616a9bf0fdc89d5f70d`
Validation: canonical archive capture, confined member/link paths, F02 source closure
Result: PASS for captured inputs only; I01 image and runtime checks remain OPEN
Artifacts: `.artifacts/graphics/i01-mesa-image/input-capture.json` and exact Mesa archive
Profile: unmodified Mesa source preparation; no guest driver execution

Date: 2026-09-30. Both F02 Mesa inputs pin revision
`06f9e28304d5d3f109c33535c1c25b9df5769af2`, whose VERSION is 25.3.6.
The canonical exact-commit archive returned HTTP 200 and contains 65,861,256 bytes:
SHA-256 `23fbb4fba5fc84d872ed8461b70edf0dd0da4dd2b19f558e0b7c4b7765975cb9`.
All 12,011 member paths and 10 links pass path, ancestor and target-confinement
checks. Captured disk bytes and hash were rechecked. No source was executed,
built or installed; no image or shared Docker state was modified.

| F02 source path | Bytes | SHA-256 |
| --- | ---: | --- |
| `src/gallium/drivers/virgl/virgl_screen.c` | 42,906 | `e558fa5550e572cffad113581b736696f33b3895e0516950181b8ff93eb38ff0` |
| `src/virtio/vulkan/vn_device.c` | 23,220 | `68f06ca4ddc2d5a62bebaa469fc43dc5e66e640eec57d46f38172327d087a4c4` |

Both archive members exactly match the existing F02 admitted file hashes.
Reproduce the immutable download/hash check from the repository root:

```sh
i01_rev=06f9e28304d5d3f109c33535c1c25b9df5769af2
i01_archive="/private/tmp/mesa-${i01_rev}.tar.gz"
curl --fail --proto '=https' \
  "https://gitlab.freedesktop.org/mesa/mesa/-/archive/${i01_rev}/mesa-${i01_rev}.tar.gz" \
  --output "$i01_archive"
printf '%s  %s\n' \
  23fbb4fba5fc84d872ed8461b70edf0dd0da4dd2b19f558e0b7c4b7765975cb9 \
  "$i01_archive" | shasum -a 256 -c -
```

The receipt preserves the first inline-script syntax failure (exit 1, before
HTTP/filesystem effects). The corrected first canonical GET succeeded; no branch
or unverified mirror was substituted.

Read-only inspection found an ARM64 Docker/Colima builder and the existing
Debian 13.5 rootfs, glibc 2.41-12+deb13u3 and kernel package 6.12.94-1 with its
matching virtio-gpu module. The installed rootfs has no Mesa/libdrm/Vulkan tools,
DRI drivers or ICDs. The old `.dockerbuild` recipe disables DRM and is unsuitable.
Meson, Ninja and pkg-config are absent from the inspected builder; a pinned
native ARM64 Debian builder and dependency-package closure remain to be created.

Next executable scope is a reproducible unmodified virgl/virtio Mesa build and
small kernel/initramfs fixture, with exact tool/package/file hashes and flags.
The custom-initrd BootContext path is available and requires an initrd below
240 MiB. A generic UART command runner must require expected command status and
output, fail on bounded timeout/steps/UART limits, and avoid synthesized GPU
completion. Register its exact fixture check through the existing F05 observer.
Boot success alone cannot complete I01. Stock VirGL startup is still unverified;
Venus runtime additionally needs currently unsupported standard capset 4.
