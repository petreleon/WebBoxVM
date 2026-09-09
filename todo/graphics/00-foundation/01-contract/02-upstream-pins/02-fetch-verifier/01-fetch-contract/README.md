# F02.2.1 — Define the fail-closed fetch and cache contract

[Parent task](../README.md) · [Worker instructions](../../../../../workflow.md)

Task: F02.2.1
Depends: F02.1
Evidence: [receipt](evidence.md)

Prerequisite lists: [F02.1](../../01-input-inventory/README.md).

## Outcome

A small standard-library verifier accepts only declared immutable inputs and writes verified bytes
atomically below an explicit external cache root.

## Starting points

- [input inventory](../../01-input-inventory/README.md)
- [graphics workflow](../../../../../workflow.md)
- [source-file limit test](../../../../../../../emulator/tests/source_file_limits.rs)

## Checklist

- [x] Define cache-root, URL, redirect, revision, and atomic-write policy with no repository cache.
- [x] Implement modular manifest loading, byte-count/hash verification, and safe cache naming.
- [x] Reject malformed inputs before creating or accepting a cache entry.
- [x] Record focused command, nonzero test count, and observed cache layout in a receipt.

## Verification

- A declared immutable input can reach only a cache path below the caller-supplied external root.
- Invalid URL, revision, path, or metadata is rejected before a cache entry is accepted.

## Contract

`source_model.py` parses the schema-v2 lock-bound inventory using F02.1's exact 17-family catalog
and constructs an `ExternalCache` only from an absolute path outside the repository. `source_cache.py` accepts only immutable HTTPS URLs
with a 40-hex path segment, denies HTTP redirects, verifies exact byte count plus SHA-256, then
uses `os.replace` after an fsynced temporary write. Existing cache bytes are re-hashed before use.
The implementation uses only Python 3.11+ standard-library modules (`tomllib`, `urllib`, and
`hashlib`); the manifest's `$XDG_CACHE_HOME` value is policy metadata, never an implicit destination.

Run the hermetic contract suite with:

```sh
python3 todo/graphics/00-foundation/01-contract/02-upstream-pins/02-fetch-verifier/01-fetch-contract/source_fetch_test.py
```

`source_fetch.py --cache-root ABSOLUTE_EXTERNAL_PATH` is the later live-fetch entrypoint. F02.2.1
does not invoke it against the inventory; F02.2.2 owns network fixtures, F02.2.3 owns the historical
15-input run, F02.3.3.4.1.2 records the grammar renewal's 16-input run, and F02.3.3.4.4.1 owns
the current 17-input WebIDL renewal.
