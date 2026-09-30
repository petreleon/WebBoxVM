# Pinned native ARM64 Mesa builder

This recipe builds unmodified Mesa25.3.6 at F02 commit
`06f9e28304d5d3f109c33535c1c25b9df5769af2`. The output is an input to
the I01 guest fixture. Compilation and installed files alone prove no
guest rendering, Vulkan enumeration, acceleration or conformance.

`lock.json` fixes the canonical ARM64 Debian manifest, immutable Debian
snapshot Release hashes, Mesa archive/F02 source hashes and Meson flags.
`packages/*.json` fixes every additional Debian archive by exact version,
architecture, byte count and SHA-256. Base packages are covered by the
immutable image manifest and recorded separately in `base-packages.tsv`.

APT authenticates the snapshot InRelease/Packages with Debian's archive
keyring. Only the expired snapshot time window check is disabled. Priority
1001 selects snapshot dependencies even when the immutable base contains a
newer package. All resolved downgrade archives are locked before install.
The image install and Mesa build run without network access. Meson wraps
and dependency fallbacks are disabled; LLVM and GLX are disabled.

Reproduce from the repository root with Docker's native ARM64 engine and
Python3.12 or later. Place the exact verified archive from the canonical
commit archive URL in `.artifacts/graphics/i01-mesa-image/` first:

```sh
python3 -m unittest discover -s guest/mesa-fixture/build -p 'test_*.py'
python3 guest/mesa-fixture/build/prepare.py
sh guest/mesa-fixture/build/build.sh
python3 guest/mesa-fixture/build/verify.py
```

The archive URL is
`https://gitlab.freedesktop.org/mesa/mesa/-/archive/06f9e28304d5d3f109c33535c1c25b9df5769af2/mesa-06f9e28304d5d3f109c33535c1c25b9df5769af2.tar.gz`.
Do not substitute a branch or another release. The archive is checked for
traversal, links, node types, duplicate paths and exact F02 driver bytes.
The compressed archive mount is read-only. Every locally extracted path,
file and symlink is compared with it before and after compilation.
Sources use bounded container-local `/source`768MiB storage; intermediate
objects use `/work`3GiB. Only installed files, logs and manifests return
to the host mount. The canonical source/build paths are `/source` and
`/work/mesa-build`, independent of the attempt identifier.

Outputs live under `.artifacts/graphics/i01-mesa-image/build/`:

- `mesa-destdir/`: full install with prefix `/opt/mesa-f02`, libdir `lib`.
- `runtime-rootfs/`: that prefix plus standard tools and recursive ARM64
  ELF dependencies, shell, busybox applets, kmod and xz; no software DRI/ICD.
- `manifests/`: source/base/flags, package versions/hashes, actual Meson
  options, runtime files/dependency edges and builder identity.
- `kernel-package/`: exact6.12.94-1 Debian kernel archive, downloaded only.
- `logs/`: stage output and retained first failures.

The runtime closure must remain strictly below240MiB of regular files.
`libvulkan.so.1` is an explicit dependency root because `vulkaninfo` loads it
with `dlopen`; static ELF dependencies alone omit it. Recovery exports preserve
the sealed installed Mesa permissions after verifying unchanged file bytes.
Only versioned `libgallium-25.3.6.so` and the VirtIO Vulkan ICD are accepted.
This stock Mesa version links EGL/GBM directly to Gallium and installs no
legacy `virtio_gpu_dri.so` alias. Debian's
build/tool dependency archives may contain other drivers; they are not
copied into the runtime closure. The stock GLX utility is installed for
explicit diagnostics; this headless Mesa build has no GLX implementation.

The build does not assemble an image or change existing images/containers.
New named Docker containers/images belong solely to this recipe. It never
installs a package on the Mac or changes Docker daemon settings.

Stage logs and execution receipts are never overwritten. `I01_CAPTURE_ATTEMPT` and
`I01_BUILD_ATTEMPT` select a new two-digit attempt after a diagnosed recipe
correction. Keep each failed directory/log. For a capture that completed
downloads but failed while saving metadata, the explicit
`--resume-captured-packages` mode validates the same locked archive set
before continuing. A different set fails; lock changes require review.

Probe builders can mount `mesa-destdir/opt/mesa-f02` at `/opt/mesa-f02`
read-only in `webboxvm/i01-mesa-builder:06f9e283-snapshot20260901`.
The retained current builder tag is `webboxvm/i01-mesa-builder:locked-20260901-159`.
Its pkg-config and loader paths select `/opt/mesa-f02/lib/pkgconfig` and
`/opt/mesa-f02/lib`. Use the actual image ID from `manifests/builder.json`
when retaining tool provenance; Docker's config digest is distinct.

Runtime recovery can select an existing sealed build with `I01_SEALED_BUILD`.
Copy its verified Mesa prefix to container-local `/opt/mesa-f02` before running
`recover.py`; repeated ELF inspection through a Mac bind mount is expensive.
Retain a fresh output/attempt and its original compile receipt. This reuses the
same compiled install and does not prove an independent clean Mesa build.
