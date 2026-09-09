# F02.4.4.1.5.2.3 blocker receipt — official Vulkan Docs closure

Revision: `b064bff1c0485e97dfeed2d5000b361eac45ecb0` preflight baseline
Validation: two detached fresh source checkouts, two new pinned-image containers, identical fixed core-build recipe
Result: BLOCKED
Artifacts: ignored `.artifacts/vulkan-docs-official.{NY6qhJ,K5cEx0}` and
`.artifacts/vulkan-docs-build.{cSyDng,40GtvM}`; no generated Docs output exists
Profile: official-toolchain preflight only; no source admission, guest API, browser, CTS, conformance, or performance claim

Task ID and date: F02.4.4.1.5.2.3, 2026-09-10 Europe/Bucharest.
Pinned source: two fresh detached clones of `https://github.com/KhronosGroup/Vulkan-Docs.git`, each clean at
`f84d432d5b8912362f96f581f29bbc4f3c8c7843` (Vulkan-Docs `v1.4.362` peeled commit).
Pinned toolchain image: `khronosgroup/docker-images@sha256:f1ca671f3bdb10ad49e238b9bf28853088a21af49504498fc9084c9b4fea4762`,
locally verified AMD64/Linux image created `2026-01-10T00:01:41Z`.

## Reproducible preflight

Each checkout was mounted read-only at `/vulkan`; a distinct writable ignored output directory was mounted at
`/work`; both fresh containers used `--platform linux/amd64 --network none --user 501:20`, `LC_ALL=C`, `TZ=UTC`,
and empty `MAKEFLAGS`. A temporary `/work/home/.gitconfig` adds only `safe.directory=/vulkan` so Git can read the
read-only host-owned checkout. The generator uses the same internal source and output paths in both runs.

```text
./makeSpec -clean -spec core -version 1.4 -genpath /work/generated \
  SPECREVISION=1.4.362 'SPECDATE=2026-09-10 00:00:00Z' \
  'SPECREMARK=from pinned commit f84d432d5b8912362f96f581f29bbc4f3c8c7843' \
  VULKAN_API=vulkan EXTENSIONS= DIFFEXTENSIONS= EXTRAATTRIBS= \
  IMAGEOPTS=inline 'NOTEOPTS=-a editing-notes -a implementation-guide' html
```

The explicit make options bind the otherwise dynamic date/remark, API, extension scope, and output configuration.
They are forwarded by upstream `makeSpec`; the fixed core route resolves the required core/base/compute/graphics
version set rather than silently using ambient extension variables.

## First failing subcheck

Both independent official-image invocations fail before `make` or generated output:

```text
File "/vulkan/scripts/parse_dependency.py", line 37, in <module>
  from pyparsing import (...)
ModuleNotFoundError: No module named 'pyparsing'
```

Direct container inspection confirms Python `3.13.5`, `/usr/bin/pip3`, and no importable `pyparsing`. The pinned
Vulkan-Docs checkout has no requirements file or CI step declaring an installation of that dependency. The two
fresh output directories contain only the temporary Git config; no generated member, image, HTML, or output tree was
produced. Both source trees remain clean at the pinned commit.

Expected result and minimum nonzero case count: two clean complete HTML builds with every closure member available
for identity binding and comparison. Actual: 0 successful builds, 2/2 identical toolchain failures, 0 generated
outputs. No 8 MiB, closure-member, WSI/video/extension scope, stale/mutable/partial, cache-stage, or output-digest
claim is made because the prerequisite build never started.

Decision and limits: do not add an unpinned `pyparsing` package ad hoc and call it the pinned official toolchain.
Supply a corrected official image or an official, versioned dependency installation procedure; then repeat both fresh
builds and bind the actual raw/generated/image/output closure before changing any successor inventory or admission.
Commit/push verification: the tested baseline `b064bff1c0485e97dfeed2d5000b361eac45ecb0` was already published;
this blocker receipt is the next scoped status commit.
Next ready task: F02.4.4.1.5.3 remains independently blocked by the lack of an immutable core-only Vulkan CTS
selector; neither blocker narrows the final VirGL/GLES/Vulkan target.
