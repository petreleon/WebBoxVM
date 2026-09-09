# Research: WebGPU generator-source audit

## Question

Is there an immutable, machine-readable WebGPU API artifact that can be admitted
as a future binding/interop-generator input without relabeling the current
semantic reference, WGSL material, or a conformance test as generated code input?

## Method

The audit kept the F02.1 inventory unchanged. It compared the local pinned
`webgpu-spec`, `wgsl-spec`, `wgsl-grammar-syntax`, and WebGPU CTS entry with
public, commit-addressed upstream candidates. It verified byte identity and
licensing only for candidates that have a concrete generator consumer.

## Rejected inputs

- `webgpu-spec` is the Bikeshed semantic reference at the local GPUWeb pin. It
  is intentionally `host API semantic reference; no generated code`.
- `wgsl-spec` and `wgsl-grammar-syntax` describe WGSL, not the WebGPU DOM API.
  The grammar's nonstandard BNF dialect is a separate limitation.
- `webgpu-cts-buffer-map` is an independent test input, not an API definition.
- `webgpu-native/webgpu-headers`' `webgpu.yml` is a useful native-C schema: it
  generates `webgpu.h` and documentation. Its own text says it maps the Web
  API into C as faithfully as practical, so it is not the browser DOM API
  definition required by this boundary. It remains a possible later native-C
  adapter source, not this task's unblocking input.

## Candidate: GPUWeb WebIDL

GPUWeb's historical `gh-pages` commit
[`e95743d3940e0ff3c267ab55ced9ae6120c7d416`](https://github.com/gpuweb/gpuweb/tree/e95743d3940e0ff3c267ab55ced9ae6120c7d416)
contains one relevant immutable artifact:

| Field | Verified value |
| --- | --- |
| Path | [`webgpu.idl`](https://raw.githubusercontent.com/gpuweb/gpuweb/e95743d3940e0ff3c267ab55ced9ae6120c7d416/webgpu.idl) |
| Bytes | 38,618 |
| SHA-256 | `bd35b2fc04f12f7ec22a9c1ae2826060778f980d8ce45dc055b8a5a8c644266a` |
| License signal | W3C Software License in the file header |
| Form | upstream-generated WebIDL; the header says not to edit it |

The commit is a generated `gh-pages` deployment, not the maintained Bikeshed
source branch. That provenance is a limitation, but its WebIDL is machine
readable and is used by a real generator. At audit time, Dawn
[`0b03a52221015de54471798594566d6d5f7750e0`](https://github.com/google/dawn/tree/0b03a52221015de54471798594566d6d5f7750e0)
pins this exact GPUWeb commit in [`DEPS`](https://github.com/google/dawn/blob/0b03a52221015de54471798594566d6d5f7750e0/DEPS).
Its [Node interop build](https://github.com/google/dawn/blob/0b03a52221015de54471798594566d6d5f7750e0/src/dawn/node/interop/CMakeLists.txt)
passes `WEBGPU_IDL_PATH` to Go `idlgen` to generate `WebGPU.h` and `WebGPU.cpp`.

## Decision

Treat this WebIDL as the only candidate eligible for a narrowly scoped F02
admission review. If admitted, it must use a new `webgpu-idl` source family and
the explicit role `future WebGPU WebIDL binding/interop generator input; no
semantic or runtime implementation`. It must not overwrite or reclassify
`webgpu-spec`, which stays the semantic reference.

Admission would establish provenance for a future WebIDL binding/interop
declaration fixture only. It does not implement WebGPU, provide a browser API,
expose a guest API, render anything, establish Mesa/VirGL/Venus compatibility,
or measure performance. The final boundary probe must bind its exact ID, family,
role, digest, license, and lock; it must reject the semantic reference, both
WGSL inputs, CTS, and the native-C `webgpu.yml` candidate.

## Required follow-up

1. Add the distinct family and immutable source through the inventory/fetch
   contracts; regenerate the canonical lock.
2. Renew every lock-bound F02.3 record and F06 reproducibility output rather
   than leaving them with a stale inventory revision.
3. Bind a minimal WebIDL provenance marker and test it against changed bytes,
   identity, role, family, license, and lock.
4. Replace the blocker probe with an offline positive-and-negative boundary
   check, while retaining the no-runtime-claim limitation in its receipt.

No upstream payload was copied into this repository during the audit. The
temporary downloaded candidates were used only to calculate the values above.
