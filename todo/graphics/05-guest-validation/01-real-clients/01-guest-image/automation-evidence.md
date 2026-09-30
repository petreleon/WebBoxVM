# I01 stock-Mesa image automation checkpoint

Baseline: `73ae73573dfa0aded9d5aa2148b408255b772234`.
Scope: pinned native build, reproducible initramfs assembly, real guest commands
and both stock-Mesa startup probes. I01 remains open: startup is FAIL and an
independent clean Mesa binary reproduction has not been performed.

## Maintained automation

The [recipe](../../../../../guest/mesa-fixture/README.md) supplies locked build
inputs, offline native ARM64 compilation, kernel/module extraction, small API
clients, deterministic newc assembly, and bounded command execution.
`make graphics-mesa-image` creates a fresh fixture; `make graphics-mesa-check`
executes tools then startup. Image/attempt parameters select fresh outputs.
`make graphics-mesa-fixture-test` is included in the root test target.

Runtime bytes and links must match the compiled manifest. Installed Mesa bytes
and modes must match that runtime; tools need executable permission and
directories need search permission. The assembler requires the MMIO transport,
GPU module, Vulkan loader, stock libraries, tools and real startup clients.
Image publication uses a fresh staging directory and one directory rename.
An explicit ignore exception keeps the lowercase image recipe sources tracked
on the Mac's case-insensitive filesystem; large binary Image files stay ignored.
Every guest invocation retains raw UART, stdout, stderr and its own JSON receipt;
the latest receipt is a separate atomic copy. Input/runner/script drift fails.

Runner defaults: 900s, 20 billion instructions, 2 million per chunk, 1MiB UART;
host process bound: 960s. Image stages have a 600s host bound, depmod a 120s bound,
and probe compilation a 120s in-container bound with a 5s kill grace.
No software fallback or fabricated host GPU completion is used.

## Verified inputs and produced files

Unmodified Mesa 25.3.6/F02 revision:
`06f9e28304d5d3f109c33535c1c25b9df5769af2`.
Archive: 65,861,256 bytes, SHA256
`23fbb4fba5fc84d872ed8461b70edf0dd0da4dd2b19f558e0b7c4b7765975cb9`.
All 12011 source archive members and all 159 locked Debian packages were verified.
Source trees were checked before/after successful compilation with networking
disabled. Meson selects only VirGL and VirtIO Vulkan, EGL/GLES/GBM, without
LLVM, GLX, software DRI/ICD, dependency wraps or fallbacks.

Native builder ID:
`sha256:16b7e49d020541143f5a5fd9a26e8a277edcbad3c55726f0bdbe45880bbf9c2d`.
The actual successful 1057-step Ninja compile took 448.717s; configure 22.325s,
install 0.438s. Runtime recovery reused the same compiled install, without Ninja.
Its original compile receipt SHA256 is
`3c14ab00d3045382cc5523b9e81365deec18364123ed8d6e0dd551a3a2a6e28b`.

Kernel package: `linux-image-6.12.94+deb13-arm64_6.12.94-1_arm64.deb`,
92,732,600 bytes, SHA256
`72db7fcfb443a4b03448bda98f4e7c1a1fa0d6c21fc57f0b119d704442f8ad49`.
Six modules include independent `virtio_mmio` and `virtio_gpu` dependency roots;
no package maintainer script or preinstalled disk was used.
The canonical kernel is 37,605,312 bytes, SHA256
`cbe59a02e7ea979a150661032440c94e2c4db0b735af2416e11ae5cac15a58e4`.

Final runtime: 105 entries, 47,963,270 regular-file bytes. Runtime manifest SHA256:
`d16ceb94bd130191c9dbc2c4af60edb55e9cacaa156a79607e0badd314c79717`.
Final recovered build manifest SHA256:
`33997e6ef0d2ddd69e412d58000c06bd82b971c17f33abde8afde0b18fdb8d6a`.
It explicitly keeps clean-build reproduction and guest API success false.

`make graphics-mesa-image MESA_FIXTURE_ATTEMPT=06
MESA_FIXTURE_IMAGE=.artifacts/graphics/i01-mesa-image/image-06`: PASS, all stages0.
Final image: 168 nodes, initrd 48,726,568 bytes, SHA256
`b08fd9090de1b0758ebce52c43c6928051c6a48eaee9c859342e7a7cd8c3637f`.
Image manifest self-hash:
`cc2f70ae52db84887c1bffd1575a9b3fd13ddbc5a497e4ef7dcd4a08b1ea9534`.
A second fresh assembly of the same inputs reproduced Image/initrd/manifest
byte-for-byte. This proves deterministic assembly, not clean Mesa reproduction.
The final directory was promoted to the canonical `image` path; previous images
remain preserved, with mapping receipts `image-promotion-04.json` and `-06.json`.

## Real guest results

`make graphics-mesa-guest-tools`: PASS, 105.534s, exit0.
Kernel 6.12.94, kmod 34.2 and BusyBox 1.37.0 were reported by actual guest commands.
The kernel bound the MMIO GPU, exposed `/dev/dri/card0` and `/dev/dri/renderD128`,
and reported capset IDs 1/2/7. The stock VirtIO ICD was present.
UART: `.artifacts/graphics/i01-mesa-image/guest-vyfdkn94/uart.log`, 14,636 bytes,
SHA256 `e7d3fa437f6ac7d4c24ccd50b69e7841d06fb8391031847435a982b59a9e3438`.

F05 `--select i01-stock-mesa-startup`: FAIL, exit 1, 109.291s, no timeout.
Both independent API clients ran against the real DRM device:

- GLES: `gbm_create_device` failed; kernel returned 0x1205 for GET_CAPSET 0x109.
  Source diagnosis: stock Mesa requests capset version 0; the emulator currently
  accepts only `(id1, version1)` and `(id2, version2)`. No capability was inflated.
- Vulkan: loader 1.4.309 loaded successfully; physical-device enumeration returned
  `VK_ERROR_INITIALIZATION_FAILED` (-3). Source diagnosis: Venus needs capset 4,
  absent from the advertised IDs. Actual renderer/ICD properties were not obtained.

UART: `.artifacts/graphics/i01-mesa-image/guest-y0md88w5/uart.log`, 14,684 bytes,
SHA256 `adb6895e9c6053dccf6e3b706953add0f8420fde3a57fcb27d539a4c0cba8967`.
F05 receipt: `.artifacts/graphics/i01-mesa-image/f05-startup-result.json`, SHA256
`278e1a911aac97adebc65b2f6d03a76a9c3c7c96071620b367515bcf8bd5a3e5`.
The registration is a neutral guest-fixture baseline with no profile effect;
CTS execution count remains 0. No API, conformance or performance PASS is claimed.

## First failures retained and corrected

All local binaries/images/logs live under `.artifacts/graphics/i01-mesa-image/`.
Failed preparation/build logs preserve missing PyYAML, bind-mount compilation
stall, source `__pycache__` contamination, stale DRI-alias assumptions and portable
export failure. Exact source guards and build fences remained enforced.
Kernel extraction retains failed builder identity, depmod timeout, missing Docker
stdin and an unrelated malformed softdep before their source corrections.

Initial guest images failed before readiness: image02 localized missing card0.
Adding the separately pinned MMIO transport produced real DRM initialization.
Guest03 completed its command but failed because an ICD without a trailing newline
glued the success marker to JSON; the check now emits a separating newline.
First F05 startup found missing `libvulkan.so.1`, omitted by static DT_NEEDED
closure because vulkaninfo uses dlopen. It is now an explicit closure root.
Runtime recovery06 timed out at 240s; unchanged first command/partial configure
output were retained. Recovery07 copied the verified prefix to container-local
storage and completed in 70.325s without increasing the bound or rerunning Ninja.
Fixture05 failed on a recovered GBM file's changed permissions with identical
bytes. Recovery export now restores sealed install modes only after byte equality;
the final fixture passed the original permission guard.

## Host verification and source scope

`make -j4 test`: exit0,722.41s; 585 Python tests in 84 suites, 1198 Rust tests,
3 pre-existing ignored Rust tests, 342 web tests. This ran before the final
loader/permission corrections; those corrections received the focused checks below.
Full log SHA256:
`a307c7533635ea3124e303cf653695abedaaf68ca962c0906f9e7836ea1cfd5d`.
Final `make graphics-mesa-fixture-test`: 92 Python (38 build/41 image/13 wrapper),
25 dedicated runner Rust tests, all PASS. Source-limit tests: 6 PASS.
F05 registration tests: 6+3 PASS. Roadmap and `git diff --check`: PASS.
Production GPU/Wasm/browser sources are unchanged by this checkpoint.

The 55 runnable recipe/Make/F05 source files have sorted repository-relative
UTF8 path+NUL+ASCII file-SHA256+LF fingerprint
`e37fc1c52f286add154f6842b9af813fa708cc2463bc4d4cbfb3108e31794734`.
Per-file digests are retained in `automation-source-fingerprint.json`.
The [runner checkpoint](runner-evidence.md) separately seals the unchanged runner.

Next real work is capset version negotiation and Venus transport/backend support.
Final GL 4.6/GLES 3.2/Vulkan 1.4, conformance, real browser apps and near-native
performance requirements remain unchanged; none is inferred from these results.
