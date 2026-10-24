# Frozen initial S02 independent specimen

Candidate2e640de failed84 controls (61 pass/23 fail). `src/main.rs` and expected files are immutable repair oracles; hash inventory and every failed rule are in ../../cdc-verification.md. `initial-fixture.json` and `initial-run.log` preserve the failing snapshot, including three caught panic-path invocations.

The actual caller is `/tmp/vlajmaji-p01-a01-s02-cdc-controls/`, package/binary `dictionary-model-s02-cdc-controls`, distinct from every positive caller. Its source/expectations are byte-identical to these preserved files. It uses public lexicon plus serde_json and an explicit fixture JSON argument. For repaired-candidate acceptance, generate a fresh output with the repaired shipped example, then run this **unchanged** program against that new output:

```bash
CARGO_TARGET_DIR=/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target cargo +1.98.0 run --locked --offline --manifest-path /tmp/vlajmaji-p01-a01-s02-cdc-controls/Cargo.toml -- /tmp/NEW-REPAIRED-FIXTURE.json
```

Expected repaired result:84 controls,0 failures, exit0; no panic. Do not edit the specimen, loosen source values, or use initial-fixture.json as the successful repair input. The preserved Cargo manifest records the task source path; no private accessors are used. This is finite synthetic/pinned-witness verification data, not a corpus import or publication.
