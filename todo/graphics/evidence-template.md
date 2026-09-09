# Evidence receipt template

[Worker instructions](workflow.md)

Copy the structure below into the task folder as `evidence.md`. Replace every pending
field with observed information. Keep the receipt under 180 lines; split large reports
into linked files. The fields below are plain text metadata used by the checker.

```text
# TASK-ID evidence

Revision: pending
Validation: pending
Result: pending
Artifacts: pending
Profile: pending

Task ID and date:
Tested commit and dirty diff hash:
Upstream manifest revision:
Guest image and build hashes:
Browser, OS, adapter and driver:
Exact command(s), working directory and tool versions:
Expected result and minimum nonzero case count:
Actual passed/failed/skipped counts and exit codes:
Negative/reference checks and observed output:
Raw log/image/sample paths, SHA-256 and retrieval/reproduction instructions:
Software fallback detection and actual execution route:
Performance conditions and frozen protocol version, when applicable:
First failing subcheck or blocker, when applicable:
Decision and limits of the evidence:
Commit/push verification:
Next ready task:
```

Use `Result: PASS` only when this task's required checks actually pass. Use BLOCKED
or FAIL otherwise and leave the completion checkbox open. A rejected optional
optimization can have PASS for the experiment protocol while its decision explicitly
says REJECTED; its acceleration feature and overall target are not thereby achieved.

`Revision` identifies the tested code, not an untested later HEAD. `Validation` names
the exact command or linked command log. `Artifacts` includes hashes and retrievable
or reproducible evidence. `Profile` states API version, enabled features and workload,
or a concrete reason this is a planning-only task. No field may say merely “done”.
