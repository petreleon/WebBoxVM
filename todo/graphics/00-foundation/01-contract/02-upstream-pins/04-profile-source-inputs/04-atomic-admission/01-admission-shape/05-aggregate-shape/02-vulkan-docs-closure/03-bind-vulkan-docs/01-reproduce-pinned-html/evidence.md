# F02.4.4.1.5.2.3.1 receipt — pinned official core HTML witness

Revision: `5ba2a53fc52cd31892be9ff562ff4622c6c0b784` clean tested baseline
Validation: two fresh network-isolated official builds, complete-tree comparison, source-clean and artifact checks
Result: PASS
Artifacts: ignored `.artifacts/vulkan-docs-official.{NY6qhJ,K5cEx0}` and
`.artifacts/vulkan-docs-build.{SfExfa,nkp7Vk}`; no fetched or generated payload enters Git
Profile: reproducible build witness only; no source admission, guest API, browser, CTS, conformance, or performance claim

Task ID and date: F02.4.4.1.5.2.3.1, 2026-09-10 Europe/Bucharest.
Two detached official Vulkan-Docs clones were clean at `f84d432d5b8912362f96f581f29bbc4f3c8c7843`, the peeled
`v1.4.362` commit. Each used the official `linux/amd64` image
`khronosgroup/docker-images@sha256:f1ca671f3bdb10ad49e238b9bf28853088a21af49504498fc9084c9b4fea4762`.

Each source was mounted read-only at `/vulkan`, its own fresh output directory at `/work`, and the container used
`--network none --user 501:20`, `LC_ALL=C`, `TZ=UTC`, empty `MAKEFLAGS`, and `PYTHONDONTWRITEBYTECODE=1`.
The non-login image environment keeps `/opt/venv/bin/python3`, which imports the image's `pyparsing 3.3.1`; no package
installation or toolchain substitution occurred.

```text
./makeSpec -clean -spec core -version 1.4 -genpath /work/generated \
  SPECREVISION=1.4.362 'SPECDATE=2026-09-10 00:00:00Z' \
  'SPECREMARK=from pinned commit f84d432d5b8912362f96f581f29bbc4f3c8c7843' \
  VULKAN_API=vulkan EXTENSIONS= DIFFEXTENSIONS= EXTRAATTRIBS= \
  IMAGEOPTS=inline 'NOTEOPTS=-a editing-notes -a implementation-guide' html
```

Both commands exited 0. Both source worktrees remained clean. Each generated tree has 2,530 regular files and
24,924 KiB; each has zero `.pyc` files; `diff -qr` over both complete trees exited 0. The rendered HTML identity is
`out/html/vkspec.html`, 10,377,052 B, SHA-256
`896452d3a3e4887ba1bad81a6b9b91e9b3b324e1fffbb9e1dec889d0b06ac041`.

Expected/actual nonzero case count: two successful clean builds and two 2,530-file trees; actual 2/2 builds and
2/2 identical trees, 0 failures and 0 skips. The 10 MiB HTML is recorded for the next actual-output grammar; it is
not an F02.2 fetched `SourceInput` or a closure/admission assertion. Remote CI, guest execution, browser execution,
CTS, conformance, and performance tests were not run.
