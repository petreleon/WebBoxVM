# F02.4.4.1.5.2.3.5.2.4 receipt — hosted synthetic witness

Revision: `166023cd3b561ab8b75a4a608a6fddd9bafb7ef4`
Validation: public GitHub-hosted Linux x64 run 34464021075; 22 focused contract tests; repository gates
Result: PASS
Artifacts: [run 34464021075](https://github.com/petreleon/WebBoxVM/actions/runs/34464021075), attempt 1; collector receipt SHA-256 `5d89e3799e4bfefeac901c5a39b0d0d5b98d23b4d5d670605d7e12856f3f086a`
Profile: immutable synthetic fixture under pinned Docker; `observed-unadmitted`, never Docs lineage evidence

Task ID and date: F02.4.4.1.5.2.3.5.2.4, 2026-09-10 Europe/Bucharest.

The successful public run checked out `166023cd3b561ab8b75a4a608a6fddd9bafb7ef4` on a GitHub-hosted Linux x64
runner (kernel `6.17.0-1022-azure`) and required `GITHUB_WORKFLOW_SHA` to equal that commit. The runner verified the
external helper blob `07a1482dfac94d8c7d05c4d7fd2f90544a4d7f69` plus every transmitted source blob before Docker started.

The five exact source payload SHA-256 values were state header `fa5ea08f44c2446c0904468ab0545e54a8e13b5a23096180e586499fff2ea6c5`,
state implementation `a2bca1bfae1bdfa7be3afdd4808467e16c882dd526b467199ca9d1a85eca42f6`, fixture
`9c0dfa754641e8ac7b4559c8e0b6d642f088979535230c4ad63f019313b27e7a`, collector
`08112299a1252d3eaf832b04b1a57c5fbc8ae3da14833461188b00a1d626169c`, and raw hex
`4447db2af9610585bd237a01f1f8adc303015710968fc7db64106a12d886153d`.

The image was `khronosgroup/docker-images@sha256:f1ca671f3bdb10ad49e238b9bf28853088a21af49504498fc9084c9b4fea4762`
(image ID `sha256:cb99714e571afcf7a1efe3b163413b74dec34a004b62014ce1f0c7618bd07da5`). The run used
`--pull=never`, `--network none`, `--platform linux/amd64`, user `501:20`, reviewed stdin source framing, a read-only
raw mount, and no privilege, capability, or security-option relaxation. Image pulling is a separate host preflight;
the collector container itself has no network.

Expected and actual result: exactly 19 canonical rows — 16 lifecycle rows, two identical three-byte snapshots, and
the terminal `observed-unadmitted` row. Actual was 19/19 with raw/output hex `726177`, successful `fork`/child
`exec`, and exit code zero. A malformed trace or nonzero collector return makes the workflow fail; the prior failed
diagnostic runs exposed the precise rejected syscall rather than issuing a receipt.

The 22 focused tests named in the sibling collector receipt passed, as did the broader 55/55 focused set, `make test`
(1,130 Rust and 337 Node tests), source limits (6/6), whitespace, and roadmap validation.

Decision and limits: this is a reproducible hosted fixture receipt only. It trusts GitHub runner isolation, the Docker
daemon, `git`, `gcc`, the unprotected branch, and a public log; it contains no Docs source, Docs build, Docs argv, or
per-member authority. It must not be used as Docs provenance or closure evidence.
