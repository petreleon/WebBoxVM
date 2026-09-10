# F02.4.4.1.5.2.3.5.2.2 receipt — bounded ptrace primitive

Revision: `e5106dae12f1988339cadcb59402c453cd5e1888`
Validation: 20 focused capability/probe/hosted-witness tests; GitHub-hosted Docker run 34453881443; repository `make test`
Result: PASS
Artifacts: public GitHub Actions annotation; temporary Docker work root removed after execution
Profile: pinned-image ptrace primitive only; observed-unadmitted and not a Docs collector or closure receipt

Task ID and date: F02.4.4.1.5.2.3.5.2.2, 2026-09-10 Europe/Bucharest.

The focused command passed 20/20 tests:

```sh
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest \
  lineage_probe_test.py lineage_github_witness_test.py \
  lineage_github_docker_witness_test.py -v
```

GitHub-hosted [run 34453881443](https://github.com/petreleon/WebBoxVM/actions/runs/34453881443) completed the
reviewed 6,700-byte C gate in the pinned image, with `parent-fork-exec-complete` and `errno: 0`. The receipt is
`observed-unadmitted` and `pinned-image-ptrace-observed`; it used no Docs source/build/argv and no privilege,
capability, seccomp, source-write, or network relaxation. The parent evidence records the full image, helper blob,
source blob, runner, and Docker metadata. This child proves only the bounded primitive policy, not a collector.
