# S03 intake execution — 2026-10-29

This record captures actual S03 intake only. The active issued prompt is `cc-prompt.md` (28,001 bytes, SHA-256 `db2b7877a95c1f0c604e894c5eb88523112bd929ebae04b6037c322956af90e4`), issued at planning commit `f51826d5a1c90df029bb4431e3cb20de4fbfc6a4`. I loaded the whole prompt and current `slice-plan.md` (§§1–8, including the full goldens, recipe, census and gate text) and `ledger.md`; current authority is S03 INITIAL / complete intake only, with all eight criteria OPEN. The S02 dependency is its accepted CDC verifier, not the earlier author proposal. S02 established the source API baseline and retained limitations; A05-03 remains partial until this S03 reimport work.

## Read coverage and current-source readback

I loaded the required current project/arc authority: source `AGENTS.md`, planning AGENTS/current Arc05 authority, project plan scope/roadmap, project-level `design.md` section “Arc05 owned authority and context adapter” (the intended `design.md`; no arc-local design file was required), arc plan sections named by the prompt and its complete current ledger, and project `cdc-directive04.md`. I read the complete S02 `cdc-verification.md`, source `AGENTS.md`, current lexicon/store/CLI manifests and relevant crate exports, plus `store/src/owned_schema.rs` in full. No dependency or manifest extension is authorized.

Source read, accepted S02 HEAD `854865a72c4ca4971e1b93fd25a2e69ee14b03a8`, tree `ef81ffbb10054bd4ceb935ac76f7912178a603e2`, parent `1b899ebb492d89ceb1561abb0befb17ff3ccc5a1`:

- `store/src/owned.rs`: public owned types/errors/categories and conversions (1–294); complete `OwnedJournal`/`OwnedJournalReader` implementation (295–939); section representation, validation and insertion (940–1284); rollback/header/summary/hash (1285–1504); curation persistence/load (1505–1711); summary/reconstruction/whole-record validation (1712–EOF). Existing reader owns the pinned SQLite connection; ordinary open/head/history use summary scopes. Existing `RollbackGuard`, observed rollback and commit-uncertainty behavior are real transaction seams; S03 must share one immediately armed guard across DDL and import row insertion.
- `lexicon/src/owned_curation.rs`: data/wire/errors/wrapper (1–468), inherited `materialize` behavior (469–672), validation (701–918), invoked base/head/shape helpers (919–1006), and serialization/hash helpers (1245–EOF). `OwnedCurationDocument::state_sha256` does not exist at this baseline; the prompt explicitly requires adding the single pure method. Existing materialize is an output projection, not what `curation_show` does.
- `lexicon/src/dictionary.rs`: `RawJson` (151–182), owned revision/head/editorial/extension types (796–902), Dictionary parser/strict byte path/accessors (903–1102), owned history and editorial validation (1179–2172), and all invoked validation helpers through EOF (2256). `Dictionary::from_json_bytes` is the original strict nested-byte parser; validation includes complete revision/active/tombstone/editor relationships. `validate_owned_base` is only a base structural check, not the complete journal genesis/adjacent policy.
- `lexicon/src/registry.rs`: declared identity/record types (1–332), document (333–352), registry construction/accessors (560–684), and full identity-prefix/document validation (685–902). `lexicon/src/lib.rs` duplicate-key RawValue visitor and recursive checker (1412–1488) is the strict whole-envelope precedent. `pack.rs` errors (97–201), seal types/load (223–386), and invoked hash helpers (707–EOF) were read as the unchanged existing seal precedent.
- `store/src/lib.rs` exports confirm the existing owned journal and error types are re-exported from private `owned`; S03’s new snapshot surface must be explicitly re-exported without adding a dependency. `owned_schema.rs` fixes the 24 ordered sections and schema1/2 DDL; S03 must not introduce schema3.
- `cli/src/main.rs`: `OwnedCommand` (196–258), input/error/id helpers (543–584), complete managed-output context mapping (684–708), owned dispatch (1123–1278), DictionaryBuild/Query/Detail routes, shared success JSON writer/flush/BrokenPipe/error handling, and terminal error mapping. Rootless commands must not synthesize a default ROOT; owned file reports use native `Path` handling while shared stdout follows the existing `QueryOutput::Json(Value)` + one-LF/error convention. Dictionary-build remains positional `INPUT STORE` with its inherited UTF-8 store response constraint. Existing full dictionary query output is direct; compact CLI output is the inner response, not the library enum wrapper.
- Complete existing owned journal, curation, CLI contract tests; both shipped owned examples; and `docs/owned-dictionary.md` were read. These establish public fixture, full typed equality, logical SQL snapshot, Busy/drop/retry, native path, output failure and query/detail conventions; they are inherited evidence, not proof of new S03 export/import.

The F01 distinction is explicit: `curation_show`/`curation-show` returns the retained `OwnedCurationDocument` without resolving contexts. ZERO-context `materialize` on a separate fresh output yields the documented `Unavailable`/`MissingContext` state where applicable; exact supplied context yields the resolved projection. Missing context can coexist with other resolved owned records; a Link whose declared Sense dependency is unavailable retains `DependencyUnavailable` rather than being rewritten to `MissingContext`.

The pure codec contract is a separate versioned snapshot/seal, with strict whole-envelope duplicate/UTF-8/syntax checks, nested original Dictionary bytes, manual Deserialize→`from_data`, exact 24-section ordered prefix rosters, actual historical state/curation cutoffs and shared curation hash. Bare typed JSON accepts valid whitespace/field ordering after semantic validation; sealed loading additionally requires canonical payload bytes after header, byte length/hash, typed history and seal-inventory checks. Schema 1 has no curation; schema 2 preserves even empty migrated curation. Import is a one-transaction fresh-root operation under one rollback owner; exact replay compares whole verified history on the same connection and makes no logical changes. Equal final head/hash alone cannot establish replay equality; the schema1 same-final-head/different-genesis-cutoff input must load as a valid alternate authority and conflict on import. Pre-COMMIT observed rollback is distinct from post-COMMIT outcome uncertainty; a pinned DELETE-mode reader yields Busy until caller drops it and deliberately retries.

## Structured inputs loaded

I read all of `history-golden-oracles.json`, both schema1/schema2 golden snapshot+seal pairs, all three schema1 same-head alternate inputs, and the complete pure generator. These are pinned expected inputs, not S03 product output: schema1 payload/seal `8040B/a16d6f1c…090a05` and `319B/9e87f3df…090d21`; schema2 `15705B/2d4925e6…c3d8a` and `353B/412de31a…82573e`; same-final-head alternate `8040B/a85fd034…f236f7` and `319B/88e4fd5c…48cf6f5`. The oracle includes all 24 roster lengths, five base and curation state hashes, and all curation cutoffs. Expected Rust tests/examples must embed or independently construct inputs in allowed files; they must not load absolute planning files or derive expectations from candidate exports.

I read the complete Makefile routes relevant to format/lint/check/default/no-default/all-features/package, `.github/workflows/ci.yml` synthetic fixture recipes, and the specified Rust guidance sections (AP18/19/33/67, API02/03/04, EH04/13/14) plus the prompt-authoring “Required reading/CC intake” section. Resource boundary: one retained A01 target; Rust 1.98, locked/offline and frozen guard; capture the actual parent PATH/toolhomes before setting owned HOME/XDG/TMPDIR. Synthetic all-features belongs only in a physically asserted disjoint archive, never on retained ignored Full data; no Full/default/benchmark/network/Arc06/external operation is authorized. Preserve every gate failure and distinguish CC proposal from CDC verifier/acceptance.

## Baseline check and status

Read-only source check immediately before closing intake: clean HEAD `854865a72c4ca4971e1b93fd25a2e69ee14b03a8`, tree `ef81ffbb10054bd4ceb935ac76f7912178a603e2`, parent `1b899ebb492d89ceb1561abb0befb17ff3ccc5a1`, branch `codex/project01-owned-dictionary`; `Cargo.lock` SHA-256 `65cd7f39a88bdd2a3dab1fcd14e9055ff7b77cb22ec3af8c309ce49dbea78700`; guard SHA-256 `426d1e49863551fe37252cecb8d36f8fb58410670fc550c6deedc89ddb92f53d`. No source/index/target/Cargo/fixture command or product operation was run. No consequential source contradiction was identified; expected S03 APIs (`owned_snapshot`, `OwnedJournalReader::snapshot`, `OwnedJournal::import_snapshot`, and pure `state_sha256`) are absent because the issued slice is not implemented. This completes only the actual intake/readback; it is not implementation, gate evidence, row acceptance, source transfer, or a request for a new pass.

## Intake addendum — golden recomputation and PackSeal definitions

I read the actual `crates/lexicon/src/pack.rs` definitions at lines 62–96: `PackSeal` is the strict typed release inventory and `SealedRelease` is the canonical release/seal byte pair. The earlier read of `pack.rs` lines 223–386 covered the loader, not those declarations; this is the corrected source citation. PackSeal remains an unchanged precedent, not an S03 API.

I ran a source-only Python query from this slice directory over exactly the seven named files under `artifacts/{schema1-golden,schema2-golden,schema1-same-head-alternate}` (both payload/seal pairs and alternate genesis) plus `history-golden-oracles.json`. It did not invoke the generator, a product binary, SQLite, Cargo, or a target, and wrote no files. The complete command was:

```python
import hashlib, json, pathlib
root = pathlib.Path('artifacts')
paths = (
    'schema1-golden/snapshot.json', 'schema1-golden/seal.json',
    'schema2-golden/snapshot.json', 'schema2-golden/seal.json',
    'schema1-same-head-alternate/genesis.json',
    'schema1-same-head-alternate/snapshot.json',
    'schema1-same-head-alternate/seal.json',
)
raw = {path: (root / path).read_bytes() for path in paths}
for path, data in raw.items():
    print('FILE', path, 'bytes', len(data), 'sha256', hashlib.sha256(data).hexdigest(), 'noLF', not data.endswith(bytes([10])))
oracle = json.loads((root/'history-golden-oracles.json').read_text())
for path in paths:
    expected = next((entry for entry in oracle['outputs'] if entry['path'] == path), None)
    if expected is not None:
        actual = raw[path]
        assert len(actual) == expected['bytes'] and hashlib.sha256(actual).hexdigest() == expected['sha256']
        print('ORACLE_MATCH', path)
for path in ('schema1-golden/snapshot.json', 'schema2-golden/snapshot.json'):
    doc = json.loads(raw[path])
    print('HISTORY', path, 'schema', doc['journal_schema'], 'source', doc['source'], 'head', doc['head'])
    for commit in doc['commits']:
        print('ROSTER', commit['revision'], len(commit['prefixes']), ','.join(str(row['length']) for row in commit['prefixes']))
    if doc['curation'] is not None:
        c = doc['curation']
        print('CURATION_CUTOFFS', ','.join(f"{row['revision']}:{row['contexts_length']}/{row['records_length']}" for row in c['prefixes']))
original = json.loads(raw['schema1-golden/snapshot.json'])
alternate = json.loads(raw['schema1-same-head-alternate/snapshot.json'])
sections = [row['section'] for row in original['commits'][0]['prefixes']]
entry_index = sections.index('metadata.entries')
print('ALT_EQUAL_FINAL_DICTIONARY', original['dictionary'] == alternate['dictionary'])
print('ALT_EQUAL_HEAD', original['head'] == alternate['head'])
print('ALT_EQUAL_LAST_COMMIT', original['commits'][-1] == alternate['commits'][-1])
print('ALT_GENESIS_METADATA_ENTRIES', original['commits'][0]['prefixes'][entry_index]['length'], alternate['commits'][0]['prefixes'][entry_index]['length'])
print('ALT_OLD_CUTOFF_AND_DECLARED_ALTERNATE', oracle['same_head_alternate']['old_cutoff'], oracle['same_head_alternate']['alternate_cutoff'])
```

The complete query exited 0. It recomputed every named file's byte length, SHA-256, and no-LF condition: schema1 payload `8040 / a16d6f1c…090a05`, seal `319 / 9e87f3df…090d21`; schema2 payload `15705 / 2d4925e6…c3d8a`, seal `353 / 412de31a…82573e`; alternate genesis `2199 / e674cb41…a1a47`, payload `8040 / a85fd034…f236f7`, seal `319 / 88e4fd5c…48cf6f5`. All five files present in the oracle `outputs` section matched its declared byte count and SHA; the original schema1 pair is not listed in that section, so its recomputation is recorded without claiming an oracle-output match.

The schema2 history supplied all five ordered 24-element rosters. For `owned-r0` through `owned-r4`, the `metadata.owned_revisions` lengths are respectively 1, 2, 3, 4, 5; other roster values match the printed per-state vectors in the command output. Its curation cutoffs are `owned-r0 0/0`, `owned-r1 0/0`, and `owned-r2`, `owned-r3`, `owned-r4` each `1/2`. Schema1 original and alternate each have two ordered 24-entry rosters. Direct comparison confirms alternate and original have equal whole final Dictionary, head, and final commit; only the genesis `metadata.entries` cutoff differs, 0 versus 1, matching the pinned old/alternate cutoffs.

A preceding exploratory Python print had a syntax error in its f-string (`backslash` in the expression); it exited 1 before printing results and made no writes. The later complete query above is the successful recorded recomputation. The initial source `sed` was run from the planning worktree and therefore found no relative `crates/lexicon/src/pack.rs`; the corrected absolute source read then loaded lines 62–105. Neither method failure accessed or changed product/target state.

For durable review, the five schema2 rosters in exact section order are:

- `owned-r0`: `[1,1,0,0,0,0,0,0,0,1,1,1,0,0,0,0,0,0,0,1,0,0,0,0]`
- `owned-r1`: `[1,1,0,0,0,1,2,1,0,1,1,2,1,2,0,0,0,0,0,2,0,0,0,0]`
- `owned-r2`: `[1,1,0,0,0,1,2,1,0,1,1,2,1,2,0,0,0,0,0,3,0,0,0,0]`
- `owned-r3`: `[1,1,0,0,0,1,2,1,0,1,1,2,1,2,0,0,0,0,0,4,0,0,0,0]`
- `owned-r4`: `[1,1,0,0,0,1,2,1,0,1,1,2,1,2,0,0,0,0,0,5,0,0,0,0]`

Each has 24 lengths; the schema1 original and alternate rosters are the `owned-r0`/`owned-r1` values printed by the same query. The query's section-name lookup confirms the original and alternate differ at `metadata.entries`, not at the neighboring component field.


## CDC actual intake acceptance and conditional initial transfer —2026-10-29 CDT

CDC read the WHOLE original9119B/dd763b02 intake plus the complete addendum to14679B/266ba3b8. Required current text coverage and source-cited readback are coherent: exact24 historical prefixes plus schema2 cutoffs/creation selections, pure byte versus sealed canonical policy, one immediately armed transaction owner/same-connection replay, old/private hash parity vs new shared pure method, portable inputs/native rootless output/errors and actual gates/endpoint. The original required-data result omission was completed by the recorded source-only query (exit0); the earlier f-string syntax error and wrong relative source read remain method failures, not product failures. Correct PackSeal definitions62–96 resolve CDC's imprecise original range label; issued prompt bytes remain unchanged. This is intake/record acceptance, NOT product correctness or row closure. No additional broad intake or root approval gate is imposed.

Fresh CDC check2026-10-30T00:32:35.522297+00:00 (2026-10-29CDT) PASS: source clean854865a72c4ca4971e1b93fd25a2e69ee14b03a8/treeef81ffbb10054bd4ceb935ac76f7912178a603e2/parent1b899ebb492d89ceb1561abb0befb17ff3ccc5a1 on codex/project01-owned-dictionary. Lock32156B/SHA65cd7f39a88bdd2a3dab1fcd14e9055ff7b77cb22ec3af8c309ce49dbea78700 and guard330B/SHA426d1e49863551fe37252cecb8d36f8fb58410670fc550c6deedc89ddb92f53d unchanged. Actual local+origin-tracking oldS04refs432d6321, oldArc04refs9026c517 and origin/maina55e1cc7 match protected identities; no fetch/network. Checked source metadata/named files only, no target/product/DB/default operation.

AFTER this named-path checkpoint commit AND the explicit CDC followup dispatch, retained Luna/high CC exclusively owns the EXACT FIFTEEN source paths in issued cc-prompt.md, source index and ONE retained target for S03 INITIAL/pass1. Five total S03 assignments including initial, no automatic sixth; no source spill/failed-hardgate automatic replay. CC may append artifacts/execution.md/raw logs; CDC owns governing planning. Root/CDC then source/index/target READ-ONLY until exact frozen/QUIET handback. Ownership does not extend source manifests/lock/PackId/pack/search/provider/data/workflow/Bauske or S01/S02 tests. Scope/exclusions/current gates/failure preservation and literal parent PATH/toolhome binding are the issued contract. No Full/default/network/external/Arc06. All8 S03 criteria OPEN. Actual transfer commit identity and dispatch are reported in parent/thread, avoiding a self-referential commit-hash rewrite.
