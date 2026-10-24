# Arc02 — Reproducible full jbotci import

| Field | Value |
| --- | --- |
| status | ACTIVE; S01 candidate01 repair required/iteration02; no child accepted |
| version/date | 1.0 / 2026-10-24 |
| depends-on | A01 fixture contracts originally accepted efef9c4; current implementation baseline d1e1c928 including CLI relocation/umbrella0.3.0 |
| blocks | A03 jbotci optional package and later complete-corpus integration |
| authority | ../cdc-directive02.md; renewed Autonomous/Expedited, CDC SolHigh / CC LunaHigh, stop before A03 |
| source | feature/project01-full-jbotci-import at ../project01-arc02/vlajmaji (absolute paths below) |

Deliver a public Rust importer and useful CLI, explicit pinned acquisition,
offline replay/check and deliberate update operations for the complete raw
jbotci-vendored Lensisku English export plus metadata. Every original row,
field/lexeme, alternative and occurrence survives into the versioned normalized
Dictionary, including excluded/unknown material. Release JSON works without
SQLite. Fresh full persistence/reopen and existing source-grouped exact APIs
compose from those bytes. Supplier counts are independently reconciled rather
than forced; terms/provenance/version distinctions remain visible.

## Source and input baseline

Source `/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc02/vlajmaji`,
branch `feature/project01-full-jbotci-import`, clean d1e1c928 at opening. Main and
old A01feature are read-only. Bauske sibling symlink resolves clean c5801ae; no
mutation. Five crates under `crates/*`, CLI `crates/cli`,0.3.0/Rust1.98/edition2024.
Actual lib/store/search contracts retain original A01 implementation; baseline
changes are CLI relocation/include path, dependency/package versions, umbrella,
docs/branding and CI. No old receipt is relabelled as new acceptance.

Pinned source `https://github.com/int19h/jbotci` revision
`3d9fd41d9ed32de26311ee27ec86b6b165985aa1`, data paths
`crates/jbotci-dictionary-data/data/dictionary-en.{json,metadata.toml}`.
Exact bytes verified from clean cached checkout `/tmp/p02a02-jbotci` and copied
locally to `/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc02/data/jbotci/3d9fd41d9ed32de26311ee27ec86b6b165985aa1/`.
That data home is outside the public source and planning tree. Raw JSON17163482B,
SHA12e826751c3e50c10dd8df1adafb5f9eb32cdf542b22c4244f284b59d3824dde;
metadata514B/SHAfe101eead51e0be1b885f3ee1b4555136e333e839baa250ddbd7b84c9af5fc98.
Complete independent raw census is `artifacts/raw-census.json`; reconstruction
URLs and actual methods are in `artifacts/readiness.md`.

Crucial observed boundary: full export uses `word`, `word_type`, `definition_id`,
not legacy API capture's `valsiword`, `type_name`, `definitionid`. Add an explicit
export importer without weakening existing La/API/LLG parsers or v1/v2 archives.
The complete list has33526objectrows/278254top-level field occurrences/14field
names,17nativeclasses,238usernames;9empty definitions and1wiki. Our declared
lookup policy yields33516eligible occurrences/31248exact spellings. All33526
assertions remain normalized. Supplier31248selected entries is a separate
projection expectation, not permission to discard2278raw alternatives/other rows.
No identical full rows/nativeIDs repeat in this pin; synthetic duplicates still
must remain distinct. No upstream word-entry ID or sense ID is supplied.

## Usable slices and sizing

| Slice | Delivered capability/home | Dependencies |
| --- | --- | --- |
| S01 — Lossless export normalization | Rust `lexicon::import_jbotci_export`, report/typed errors and CLI jbotci-import/jbotci-check; complete raw-preserving release JSON, documented field map and full actual population proof without SQLite. | Inspected model/registry/source bytes and supplier subset metadata format. |
| S02 — Pinned pipeline and persisted lookup | Bash3.2/Make acquire/replay/check/update, explicit pins/local history; useful fullraw→release→freshstore/reopen→groupedRust/CLI recipe and actual fullpopulation controls. | Accepted S01release/import API; current DictionaryStore/SearchEngine. |

S01 combines the coherent normalization/API/CLI contract with its tests and
full input. S02 is the externally usable acquisition/update/replay workflow and
its actual persistence integration, not evidence-only work. No final-integration
arc postpones first usable entrypoints. Each is sized with source/guide/query
intake and repair headroom; maximum5refinements per slice. Plan S02's exact open
set against S01 rather than inventing all detail now. All inherited obligations
remain in the two capability slices and arc composition, not hidden deferrals.

## Ownership, resources, checks and limits

CDC `/root/vlajmaji_p01_a02_cdc` owns planning/design/review; current requested
OpenAI gpt-6.1-sol/high. CC explicitly gpt-6-luna/high is assigned after readiness,
source mutation/index only during issued intervals. Root read-only. Fresh separate
SolHigh review context, authoring none of the arc, owns independent technical
composition and final canonical packet gate. Backend model attestation unknown;
no substituted defaults. No CRC selected.

One retained build target `/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc01/target`,
exclusively handed to Arc02 after exact abandoned process recovery. Launcher
`/tmp/vlajmaji-p01-a02-cargo` exports it plus CARGO_NET_OFFLINE=true; every Cargo
and Make invocation explicitly sets this CARGO_TARGET_DIR and uses that launcher
(CARGO=... for Make). Source/SQLite inputs frozen during scans. No cache deletion;
old source-local68MiB target remains untouched. Initialfree2.6GiB/current2.5GiB
after baseline check. Keep one full normalized release and one full derived store
per active caller; after consumed proof, only explicitly task-owned derived
outputs may be removed with receipt/hash retained when space requires. Avoid
duplicate native graph builds; reuse retained dependencies. Recheck free space
before large outputs and hold rather than erase unknown caches. Serialize builds,
imports, stores, package scans and source mutation. Unique external package AND
binary names per independent caller; source hashes/offline locks accompany proof.

Run source AGENTS gates: format→lint→check, focused tests, exact isolated search
no-default and all-features, affected umbrella features, public outside callers.
Full input must actually run; no ignored-test/script-existence substitution.
Independent raw lexeme/native accounting and query reference derive from raw input,
not importer output; negative controls have valid counterparts. Full JSON reload
is necessary but alone cannot prove field mapping. S02/arc composition must
create/drop/reopen full SQLite and consume grouped results, counts, duplicates,
source attribution/native fields and deterministic pages against raw references.
Retain existing Q01–Q06 and v1/v2 identities; no source parser rewrite.

Raw captureformat label, Dictionarydocument1, importer normalizer1 and store
schema1 are separately named. Preserve exact captures, provenance/time precision,
source/author/distributor roles, content notices and history. Public redistribution
stays Held even though pinned supplier notice asserts publicdomain. Do not turn
supplier evidence into independent legal conclusion. Apache2 code notice is
separate. No fullrawdata in publicpackage; only synthetic fixtures and smallpins.
All12metadata areas retain model homes; populate only supplied facts. No invented
senses/POS/etymology/frames, prose place parsing, extractedrafsi, ranking winner,
new phonology, optionalcorpuscrates/A03, topology/A04, editor/A05, finalA06,
publication, push/PR/merge/main integration, service/schedule or external messages.
Public pinned jbotci fetch/read is authorized; no authenticated Lensisku.

## Composition and close contract

All child rows and corrections independently dispositioned before arc eligibility.
Arc ledger requires a fresh separate full-path demonstration, source-field/native
accounting, actual offline caller, complete persist/reopen, source-grouped exact
counts/pages and acquisition/check/update history. Child successes do not prove
composition. CDC assembles closing report and retrospective, updates project
current partial contributions/carryovers, then the same independent review context
checks the final canonical packet. Only accepted gates permit mechanical close.
All14projectrows remain open at opening; no project acceptance claimed. Preserve
failed/retried setup/product attempts, original pins/receipts, issued prompts and
five-iteration history. No close/verifier/retro records precreated.

## Version history

-1.0 —2026-10-24: activated full A02under directive02; source/data reconnaissance
supports two usable slices, explicit export-vs-API boundary, measured complete
33526population and protected/exclusive resource plan. No scope reduction.
