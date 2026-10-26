# CC author observation — Full round 1

**Status:** one frozen author round recorded; no topology selected and no acceptance/row closure claimed. Source remains `4b4158c8a8a489ead639b73706b01061de17cfe1`; release probe remains `e12ddf648d40f385a021591745b19c3e8b3ab8341ca88b780108bb83dbe1a215`. No source edit/build/store create occurred in this interval.

## Frozen identities and execution

I read `cc-observation01.md` and the complete round plan, request/expectation manifests, runner, and stage parser before running anything. The `/tmp` copies and canonical `cdc-prerequisite02` artifact copies have matching SHA-256s. The author plan is `887a18af2199e19be998fd544cefa0956085daf2908c3cec2351dd3bc0ee96f7`; requests `6fe407b2411263370524a44bd101c271fa17a3c3f4e92ee65425025f76d24c21`; expectations `a6ca6361e4d2af0c9fc6aa3a4d69a4653954e3b2953edc3c12a9008b717a2981`; probe `e12ddf648d40f385a021591745b19c3e8b3ab8341ca88b780108bb83dbe1a215`; runner `23669c99fad4e3a81916f1faf3f1eb368531965ee4652f31ea14f59eec73ffa1`; parser `3a6e884371c182933aa314b00de8f1717f126bf6d4fd0e2a7c6288e80f7498f5`. The request source-manifest identity is `d2cebaf40686db8f5194fc76b498bf3828e53f5457ba94758c650a3935d6db71`.

The exact frozen runner was invoked once:

```text
python3 -B /tmp/vlajmaji-p01-a04-s01-cdc/paired_runner02.py /tmp/vlajmaji-p01-a04-s01-cdc/full-freeze02/author-round01.json /tmp/vlajmaji-p01-a04-s01-cc/full-round01
```

It executed the six serialized U/A/F fresh and warm processes once each. All six exited 0, were below the 60-second timeout, and passed the runner's true `/usr/bin/time -l` OS-RSS parser and 8 GiB cap. Exact PIDs, outer wall, output sizes/hashes, RSS and raw stdout/stderr paths are in `full-round01/observations.json` (SHA `d22b9a9e939c4ec33a982e73913280b2eb4a4f37baa187fe348b6e617f719a73`). Controller stdout SHA is `e987507508c9e9dcf66bb23b445b9a51fe2905bc2d878b0b450d59a4dd82d6b5`; controller stderr is empty (SHA `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855`).

## Correctness results

Each fresh process emitted exactly 7,160,366 bytes with the frozen legacy SHA-256 `144b8b09b6e378243b36c8e01ab06353c3d573459668c8c158a86aebbea73556`:

| Topology | Fresh end-to-end wall | True OS peak RSS | Open profile: provider / load-compose-digest / common index / total |
| --- | ---: | ---: | --- |
| Unified | 2.708 s | 1,838,383,104 B | 2.024 ms / 1.351 s / 546.683 ms / 1.900 s |
| Attached | 4.913 s | 2,029,535,232 B | 8.514 ms / 3.584 s / 502.576 ms / 4.095 s |
| Fanout | 4.738 s | 2,033,123,328 B | 8.540 ms / 3.516 s / 480.220 ms / 4.005 s |

All three complete fresh wires match byte-for-byte. Each warm batch produced exactly 27 measured response files. Their paths, byte lengths and SHA-256s agree with the batch's own response listing. Parsed complete JSON values for all 27 requests are equal across U/A/F, including nested provenance and explicit missing/null values. The frozen `metrics02.py` parser accepted all stage accounting and request order: **18 exact and 9 reference rows per provider**.

The fixed query oracle passed for each topology: `farvi-all` reports one distinct spelling, five occurrences/native candidate occurrences and three groups (Lensisku 3, LLG 1, Sutysisku 1); source-filtered counts are 3/1/1. `xoprsnqv` is absent with zero occurrences/candidates; Sutysisku-filtered `zoizoizoi` has zero eligible occurrences. Page zero has no returned groups/cursor while preserving full totals. Page sizes 1, 2, 3 and 8 have the declared globally ordered hit sequence; concatenating each declared size-1/2/3 chain equals the complete page-8 hit objects and ends with a null cursor.

The five positive references all resolve: Jbotci local assertion, Jbotci native-333, La native-333, LLG `farvi` component, and Sutysisku native entry. Unknown remains Unknown, the frozen unresolved target remains Unresolved, the wrong-role local entry returns typed `invalid_reference`, and the later missing catalogue target returns NoMatch. All complete reference values compare equal across topologies.

## Timing and resource observations

The outer fresh-process observations (including process launch and output capture) and the internal `run-total` stage are separate. The warm process wall includes 27 priming plus 27 measured requests. Per-request stage figures below are descriptive for this one round; they are not a three-round median or a topology decision. Exact warm lookup is SQL plus outer residual. Reference warm time is the conservative whole lookup/resolution call. Full output cost includes hydration plus Value conversion and JSON/LF rendering; file writing is separately timed.

| Topology | Warm exact lookup min / median / max | Warm reference lookup min / median / max | Full `farvi-all` hydration + Value + render | Warm process wall / peak RSS |
| --- | ---: | ---: | ---: | --- |
| Unified | 71 / 145 / 204 µs | 2,059 / 6,155 / 11,578 µs | 13,233 µs | 4.044 s / 2,397,503,488 B |
| Attached | 111 / 193.5 / 295 µs | 2,047 / 5,803 / 10,639 µs | 12,886 µs | 6.306 s / 2,166,374,400 B |
| Fanout | 114 / 188.5 / 225 µs | 2,083 / 6,294 / 10,342 µs | 12,542 µs | 6.277 s / 2,153,201,664 B |

The relevant observed values in this round are below the frozen per-trial ceilings (8 s startup, 10 s fresh end-to-end, 750 ms warm exact/reference maximum, 1 s full hydration/output maximum, and 8 GiB read-process RSS). These six processes do not provide three paired rounds or a winner. No cache purge or cache-state control was performed; the retained Cargo-cache delta cannot be stated because no initial size baseline was recorded. No per-process OS file-descriptor sample was taken. Logical DB handles follow the frozen providers (U: one; A: one connection with four logical databases total, one main plus three attached; F: four connections/handles); this is not an OS-FD measurement.

The prior CDC prerequisite receipt `artifacts/cdc-full-build02/receipt.md` records creation/full verification and independent raw-derived native SQL checks for the same five stores, including unchanged database pre/post hashes. This author round only queried those stores; it did not create, mutate, re-hash or re-verify them. No historical/default store was opened.

## Attempts, retained outputs and limits

One local result-audit script attempt incorrectly guessed 19 exact and 8 reference rows; it stopped at the first Unified count assertion, where the immutable request manifest showed 18 exact and 9 reference. The frozen runner had already completed its six processes. The corrected author audit used the manifest-derived IDs and passed; **no runner process was repeated**. This was an author-side audit-denominator correction, not a product or query failure. The correction and resulting compact audit are retained in `/tmp/vlajmaji-p01-a04-s01-cc/full-round01-analysis.json` (SHA `2a917778d0c20b3b4d8172eaf9d26ed0613a8d1f6c1f5af31247a18da6caa153`).

Complete raw process streams are retained at `/tmp/vlajmaji-p01-a04-s01-cc/full-round01/round1-{unified,attached,fanout}-{fresh,warm}.{stdout,stderr}`. The runner's machine record is `full-round01/observations.json`; batch responses are retained, uncommitted, under `/tmp/vlajmaji-p01-a04-s01-cc/full-responses02/round1-{unified,attached,fanout}/` (27 per provider). Controller streams are `full-round01-controller.stdout` and `full-round01-controller.stderr`. No scripts/configs, source files, stores, inputs or prior data artifacts were modified. Shared target/cache was not purged. All six processes have exited; source remains clean at the frozen commit. Ownership has been explicitly returned to CDC. All six slice rows remain OPEN pending its rounds 2/3 and independent disposition.

### Receipt correction


The original receipt SHA-256 was `8c9e21975fa7c66ca3cfc009220094c7039f33f8dcec9fdc257c249c4582d1cd`. It incorrectly reported Attached as one connection with four attachments/five logical database handles. Source `snapshot.rs:206-225` opens the first member as `main` and attaches only the remaining three; the correct count for this fixed four-member manifest is one connection and four logical databases total (one main plus three attached). This is a reporting correction only: no runtime result, database, or source changed. No process or data scan was run for this correction.

### RSS receipt erratum

The Attached fresh-process RSS above is corrected from `2,029,532,232 B` to the retained machine receipt value `2,029,535,232 B`. The original `8c9e...` wording and SHA are retained above, but no verified byte-for-byte copy of that original receipt was saved before editing; the original bytes are unavailable. This correction changes only the report, not any recorded process result. No rerun or data scan was performed.
