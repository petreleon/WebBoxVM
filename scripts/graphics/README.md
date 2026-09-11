# Local graphics check runner

`run.py` executes only checks explicitly selected from a caller-provided JSON catalog.
It has no built-in API profile, GPU capability, source-admission, conformance, or
performance inventory.

```sh
python3 scripts/graphics/run.py --root . --catalog checks.json --select local-unit \
  --result .artifacts/graphics/local-unit.json
```

The catalog uses schema 1. Each check has exactly `name`, `command`,
`expected_count`, `artifacts`, `tools`, and `prerequisites`. Commands and tools are
argument arrays and never pass through a shell. `artifacts` are relative paths whose
bytes are SHA-256 hashed after the child exits.

```json
{
  "schema": 1,
  "checks": [{
    "name": "local-unit",
    "command": ["python3", "-c", "print('WEBBOXVM_GRAPHICS_OBSERVED_COUNT=1')"],
    "expected_count": 1,
    "artifacts": [],
    "tools": [["python3", "--version"]],
    "prerequisites": [{"kind": "executable", "value": "python3"}]
  }]
}
```

The child must emit exactly one `WEBBOXVM_GRAPHICS_OBSERVED_COUNT=N` line across
standard output and standard error. A missing, zero, duplicate, or mismatched count
is `FAIL`, even when the child exits zero. The JSON receipt records the command,
revision, dirty-diff SHA-256, declared tool output, duration, exit status, counts,
output digests, and artifact hashes.

Prerequisites are `executable`, `asset`, `browser`, `hardware`, and `permission`.
Hardware names an opt-in environment variable; permission names a relative probe path
and access (`read`, `write`, or `execute`). Missing prerequisites yield `BLOCKED` and
exit 3, never `PASS`. Invalid catalogs or selections exit 2. A single failing child
preserves its exit status and standard streams; other runner failures exit 1.
