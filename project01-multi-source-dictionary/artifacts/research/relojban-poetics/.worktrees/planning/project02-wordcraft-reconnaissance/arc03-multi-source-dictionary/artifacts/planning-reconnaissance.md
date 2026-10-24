# Arc03 planning reconnaissance and receipt

Purpose: preserve the finite evidence behind raw-vs-compiled import, source grouping and the later SQLite dispatch comparison. Planning-only inspection; no full dictionary payload fetched, import/build/benchmark or refresh run, no credentials/automation/publication. Accepted baseline source7a05516/planning1b9f741. Earlier corpus selection/coverage and standards facts remain pinned E02/E03/Arc02 evidence; these families are required priority inputs, not a freshly measured best-quality ranking.

## Current jbotci source recheck (parent research; locally inspected by CDC)

Root's TLS-valid system curl rechecked GitHub HEAD at **2026-10-23T20:14:54.210550+00:00**: commit3d9fd41d9ed32de26311ee27ec86b6b165985aa1, committed2026-10-11T19:15:49Z, exactly the previously accepted pin. Receipt scope is metadata/code only. A cached web main page showed older September metadata and is superseded for this observation; it is not a separate corpus revision. Initial configured Git HTTPS→SSH/DNS and Python urllib local-CA failures were retained in the parent receipt; TLS validation was not disabled. Source home https://github.com/int19h/jbotci ; exact raw URLs use https://raw.githubusercontent.com/int19h/jbotci/3d9fd41d9ed32de26311ee27ec86b6b165985aa1/ plus each path below.

| Pinned path | Bytes | SHA256 |
| --- | --- | --- |
| README.md | 7035 | a7a306800f32441e2cf4366dfdc08f2bb2ad976681225f181fc1fb4dc86bc6a9 |
| Cargo.toml | 9400 | 07f5e9b85f4bd3bf768d335f1da786f2924e9a0d6692d64586026884ec4096a2 |
| crates/jbotci-dictionary-data/Cargo.toml | 959 | edf066258e514447e94e03565d424ae47a3e533f8b0f2e4cdb7eec9ae616d306 |
| crates/jbotci-dictionary-data/data/dictionary-en.metadata.toml | 514 | fe101eead51e0be1b885f3ee1b4555136e333e839baa250ddbd7b84c9af5fc98 |
| crates/jbotci-dictionary-data/src/lib.rs | 30161 | 535a0453678e73db35e7c433aac4b4628cb8360f03150e0a1514c08c32929d00 |
| crates/jbotci-dictionary-data/build.rs | 46020 | 9d1952b172f914879dead894d48a298ff7a065728bead98f1b5682504afef845 |
| crates/jbotci-dictionary-data/data/README.md | 6522 | 018408d19ce7adb2b0b72e3d5c980c281b6b8ece9fa5f5efd4add50f92170388 |
| LICENSE.md | 1296 | ade139bd35873b8e1f1f50ff62e3ca133432d2063b28e8b1c3d71538ee270abd |
| THIRD-PARTY-NOTICES.md | 337532 | eb10de003ff1ee0e6716a1f0986612a9ad6d6495c19ec2c95eabe11bd921e3c7 |

Exact received metadata (supplier claims; payload hash and row counts **not independently validated now**):
```toml
language_tag = "en"
language_realname = "English"
source_language_tag = "jbo"
format = "json"
positive_scores_only = false
filename = "dictionary-en.json"
metadata_url = "https://lensisku.lojban.org/api/language/languages"
download_url = "https://lensisku.lojban.org/api/export/dictionary/en?format=json&source_lang=jbo&positive_scores_only=false"
lensisku_created_at = "2026-10-04T03:07:08Z"
sha256 = "12e826751c3e50c10dd8df1adafb5f9eb32cdf542b22c4244f284b59d3824dde"
definition_count = 33526
entry_count = 31248
```

CDC actually received the entire compact receipt/metadata/dataREADME, build.rs238–304, lib.rs1–82 and THIRD-PARTY-NOTICES238–255. Full30KB/46KB code or337KB notice reading/copy is not claimed. The supplier docs/build distinguish raw33526 definition rows from31248 selected embedded entries: compiled view discards undefined/nonword rows, chooses highest-score/lowest-ID definitions, strips predictable4-letter forms and merges extracted rafsi. The raw vendored JSON is documented as a verbatim unfiltered export including repeats and wiki articles. S02 must acquire that pinned payload and validate independently; english() is not the lossless input. Unknown classes/raw articles require retention plus explicit lexical eligibility/diagnostics and balanced row accounting.

Default planned acquisition is the public pinned jbotci vendored file+metadata, without credentials. Upstream documents an authenticated vendor-dictionary refresh, offline --check and authenticated --check-upstream; none was executed or observed against live Lensisku here. Direct live export is optional future scope only with explicitly supplied credentials, kept out of process arguments/receipts. Automated means an explicit reproducible pipeline with replay, check and update stages: replay transforms pinned preserved inputs; check verifies hashes/schema/count/accounting without rewriting releases; update deliberately acquires and pins a new snapshot/release while preserving the old one. Changing upstream content must never silently change released data or an ordinary build. Ordinary corpus package builds/loads perform no network access. No scheduled automation was created or authorized. Optional extracted/enriched forms retain tool/method/run/source attribution; do not relabel them upstream assignments.

The supplier notice at THIRD-PARTY-NOTICES.md242–252 asserts jbovlaste/Lensisku data are public domain. This is supplier evidence whose upstream coverage must be traced before public redistribution of a release. Code MIT does not establish data rights; no projectwide license decision is made. Internal planning/evaluation remains in scope.

## Established SQLite patterns (primary documentation inspected2026-10-23)

- [ATTACH](https://www.sqlite.org/lang_attach.html), especially details68–77: connection-scoped additional files, qualified names and cross-file commit conditions. WAL or an in-memory main database does not provide crash atomicity across a multi-file write set. This is documentation, not an executed failure test.
- [SQLite limits](https://www.sqlite.org/limits.html), section11: default10 attached files per connection, documented upper125; observe actual compiled/runtime limits before choosing attachment strategy.
- [URI filenames](https://www.sqlite.org/uri.html), activation and immutable parameter: enable URI handling deliberately; immutable=1 skips locking/change detection and is only valid for sealed never-modified files. The editable owned store is writable/locking-aware, not immutable.
- [Datasette cross-database queries](https://docs.datasette.io/en/stable/sql_queries.html#cross-database-queries): --crossdb illustrates attached SQL queries over named files. This is a working-project documentation precedent, not a tested local dependency or a decision to adopt its topology.

S04 compares unified derived SQLite, ATTACH and app fanout under one complete grouped query/link/count/page contract. Data-crate independence does not dictate file topology; local FTS rank statistics are not a shared cross-store relevance scale. Preserve active-pack/owned-revision query snapshots, explicit unavailable states and complete scan references. No topology/benchmark result is asserted today.

## Planning intake

Current project-management SKILL/README and focused01/02/03 guides were fully received for this new planning operation; a truncated combined view of02 was recovered with bounded reads. Reuse prior genuinely complete unchanged collaboration/verification/scientific/lab/Rust/domain intake from the accepted arc; no new code authoring/assignment is performed. Existing source/P01/accepted raw data remain unchanged. This note serves the concrete importer/topology planning decisions; further corpus refresh/research is not automatically authorized.
