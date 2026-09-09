# F02.4.4.1.5.2.3.3.1 receipt — pinned Docs input observation

Revision: `64b725c99f39f28999e29039200a6f665f3d3210` verified implementation
Validation: 6 positive and 10 hostile observer tests, contract CLI, pinned-image C compile/probe, source limits,
roadmap checker, `git diff --check`, and `make test`
Result: PASS
Artifacts: compact tracked header only; two raw traces, two source checkouts, and two rendered trees remain ignored
under `.artifacts/graphics/f02.4.4.1.5.2.3.3.1/`
Profile: input observation only; unadmitted and not cutover-ready

## Capture

Both runs used the pinned Vulkan-Docs commit
`f84d432d5b8912362f96f581f29bbc4f3c8c7843`, the pinned image
`khronosgroup/docker-images@sha256:f1ca671f3bdb10ad49e238b9bf28853088a21af49504498fc9084c9b4fea4762`,
the official core HTML argv/attributes, `linux/amd64`, user `501:20`, `LC_ALL=C`, `TZ=UTC`, no network, a
read-only `/vulkan` source mount, and a separate writable `/work` mount. The exact runtime producer argv was
observed and bound as `9a985b9e36cdf53e7baa0a3b30c9df41d33d8feb04f8f08904532a807b4dbf3c`.

`observer-a` used `sources/observer-a` and `runs/observer-a-r5`; `observer-b` used `sources/observer-b` and
`runs/observer-b-r1`. Each clean checkout retained the reviewed source-tree identity
`99cfd3132413891764f98f246567d0ce0344f2f0bb5499bfbc7fc47646fa122b`. The observer implementation identity is
`7d19442b14697280050efae4f291fc1ac272ae24c703e496262089dfb3bfaa95`; it binds the C interposer, Ruby include
hook, and all runtime Python modules.

The interposer records content reads and read-only generated inputs, while the Ruby hook records successfully
resolved Asciidoctor includes. It covers `read`, `pread`, `readv`, `mmap`, `fread`, `fgets`, copy, sendfile, and
splice. The pinned-image probe observed `fgets` on `/vulkan/Makefile`, closing the earlier GNU Make stdio gap.
For a generated input, the normalizer rejects a changed size or second-resolution mtime after observation before
hashing its final identity.

## Compact header

[`vulkan_docs_core_input_observation.json`](vulkan_docs_core_input_observation.json) binds the two separate raw
artifacts to build witness `8cfab6fe4527f3973d7d7923ffe0922ab689fc2daf2f6e955ff7603fc8ba78d6` and has observation identity
`dbd75e819e3021e28e2bdb85eeccd675a688ffdb6692fddcd8ff58a221363554`.

| Run | I/O trace | Include trace | Input manifest | Raw / derived / includes |
| --- | --- | --- | --- | --- |
| `observer-a` | `f263e0aa51baaafce451dd27ebad3f9ab387cdfa5aa14264090744f02342e13a` | `9bdd4e1a79ffc224832ec58763034dcf87e9660af7652fda5193891c226ec024` | `1896b1a211ecf82ff8b7826718ed797083fa80f41eba0a985376cef124b3056c` | 298 / 1,462 / 1,973 |
| `observer-b` | `62335512b79c31a93fa822223ed6524d3abfb7a0eb48db1be8696ce840dc91e0` | `9bdd4e1a79ffc224832ec58763034dcf87e9660af7652fda5193891c226ec024` | `1896b1a211ecf82ff8b7826718ed797083fa80f41eba0a985376cef124b3056c` | 298 / 1,462 / 1,973 |

Each run separately binds output tree `2,530 / 17,019,466 B /
26e8e484d34222d9ba8a72c883ff8bb65a4e5194f399c24aa9bd5d7e47c49f32` and primary HTML
`896452d3a3e4887ba1bad81a6b9b91e9b3b324e1fffbb9e1dec889d0b06ac041`. Its phase-record counts are
producer 8, make-control 1, generator 28, Asciidoctor 1,646, postprocess 3, and asset-copy 81.

## Validation

```text
PYTHONDONTWRITEBYTECODE=1 python3 vulkan_docs_observer_test.py
PYTHONDONTWRITEBYTECODE=1 python3 vulkan_docs_observer_hostile_test.py
PYTHONDONTWRITEBYTECODE=1 python3 vulkan_docs_observer_contract.py vulkan_docs_core_input_observation.json
cargo test -p emulator --test source_file_limits --quiet
PYTHONDONTWRITEBYTECODE=1 python3 scripts/check_graphics_roadmap.py
git diff --check
make test
```

The focused suites passed 6/6 and 10/10. Source limits passed 6/6. `make test` passed: 1,127 emulator unit tests,
3 ignored, plus all boundary, roadmap, and 337 Node tests. No Rust/Wasm or browser code changed, so `make web-pkg`
was not applicable.

## Boundary

This receipt does not compare the two normalized identities, bind conditional WSI/video/extension/promotion scope,
stage or re-hash a payload tree, admit an input, or make a guest/browser/CTS/conformance/performance claim. Dynamic
process observation is deliberately not a proof that an arbitrary uninstrumented static or direct-syscall tool has
no additional inputs. Child `.3.2` binds the observed scope and child `.3.3` performs the independent-capture
comparison; this header remains unadmitted regardless of their outcome. Remote CI was not run.
