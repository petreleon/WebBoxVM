# Browser GPU resource residency: protocol and validation

[Residency boundary and invariants](virgl-resource-residency.md) define the
eligible private protocol. This companion retains the execution and test detail.

## Protocol phases

1. Eligible source-over v6 `VGB1`/v2 `VGM1`, direct v14 `VGB1`/v10 `VGM1`,
   and v2 clear packets render into one of at most 16 persistent textures
   totaling 16 MiB, with an individual texture capped at 4 MiB, present through
   a GPU copy, and return a resident completion.
2. Eligible source-over v7 `VGB1`/v3 `VGM1`, direct v15 `VGB1`/v11 `VGM1`,
   or v2 clear packets with a nonzero predecessor repaint and rekey that exact
   texture only after GPU completion; Rust accepts it only while the resource
   still names that predecessor.
3. Rust stores the producer sequence only after the matching completion
   validates the pending effect and context generation.
4. A full `TRANSFER_FROM_HOST_3D` emits private `VGR1` v1 naming that producer.
   The browser maps the persistent texture; Rust validates the full image,
   refreshes its shadow, and only then writes the requested backing range and
   completion response. A strictly partial transfer emits `VGR1` v2 with a
   bounded source origin and size. Rust validates and converts only those pixels
   directly into the requested backing rows, without shadow mutation, damage, or
   producer release; the same GPU target remains authoritative.
5. When a full CPU replacement or a new CPU-synchronized render ends residency,
   Rust emits a no-ack `VGL1` release for the old producer. The browser destroys
   that cached texture; duplicate or delayed releases are harmless.
   Context and resource teardown use the same release path.
6. A strict full `RESOURCE_COPY_REGION` from a resident source emits `VRC1` v1.
   The browser copies the source texture into a fresh bounded target without a
   canvas transfer or pixel map. Rust validates the source owner and promotes
   only the destination; a stale completion emits `VGL1` for that new sequence.
7. `VGM1` v12 source-over and v13 opaque replacement accept one resident sampled
   source for a fresh non-depth singleton. Both Rust and the browser revalidate
   its producer, dimensions, context attachment, and bounded durable texture;
   only the new target is released after a stale completion.

## Cost model

| Operation | Current | Resident path |
| --- | --- | --- |
| Eligible draw completion | O(W×H) readback and transfer | O(1) host control; GPU presentation copy |
| Repeated full eligible draw | O(W×H) readback and transfer | Repaint and rekey one persistent GPU texture |
| First guest CPU read | Already paid per draw | O(W×H), once at the synchronization boundary |
| Partial guest CPU read (w×h) | O(W×H) map before scatter | O(w×h) map; retain GPU authority |
| Full resident copy | O(W×H) readback plus CPU pixel copy | One GPU texture copy; retain source and promote destination |
| Resident texture sample | O(W×H) readback plus upload | One sampled durable texture; no pixel payload or map |
| Resident lookup | — | O(1) keyed by resource ID |
| Identical vertex input | Upload every frame | Exact cached bytes skip `queue.writeBuffer` |
| Browser memory | Transient target | Explicit bounded texture budget |

The resident path improves draw-heavy workloads that do not read pixels on the
CPU between draws. It deliberately defers, rather than removes, the cost of a
guest-visible readback.

## Validation plan

- Promote only a matching full-target packet; reject stale, wrong-resource, and
  wrong-generation completions.
- Prove a deferred `VGR1` transfer writes exact scatter-backed bytes only after
  its matching readback.
- Prove `VGR1` v2 maps only its validated source rectangle, converts BGRA/RGBA
  correctly, writes no partial backing range on failure, and retains its producer.
- Prove an upload, unref, reset, and device loss cannot revive a prior owner.
- Compare a resident batch followed by transfer against the current readback
  fixture byte-for-byte, including BGRA/RGBA normalization.
- Prove direct full/RGB/partial masks keep their target resident, preserve the
  exact WebGPU write mask, and do not issue a mapped GPU readback.
- Prove a `VRC1` completion retains its source, promotes only its fresh target,
  uses `COPY_SRC`/`COPY_DST` texture usage, and releases a stale target alone.
- Prove `VGM1` v12/v13 bind only an existing producer with `TEXTURE_BINDING`, map
  no pixels, preserve the requested opaque mask, lock the source, and release a
  stale new target alone.
- Measure mapped readbacks per N eligible draws and report browser/device data
  separately from guest protocol correctness.
