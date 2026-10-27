# S05 iteration03 — readable Exact/reference/detail and strict profiling carryover

Status: DRAFT — engineering spine for root review; NOT ISSUED; no source transfer.

## Outcome and boundary

Deliver the useful reusable source-grouped readable Exact AND Resolve/detail increment on frozen7399563f98b41d26bd5099d04873e1a8cc7e23fe/treee1840807bba597e779a1b28d02937ceff9ffc4ab, branch codex/project01-readable-compact-output. This is pass4ofFIVE TOTAL, not a reset. Initial/pass1 and corrective01/pass2,02/pass3 remain immutable. Semantic/API/finiteTiny contribution passes at739, but strictF03 has ONE OPEN carryover; INITIAL is not unconditionally accepted and no Full metric trial is eligible. All six ledger rows stay OPEN. Same Sol6.1/high CDC and retained Luna/high CC, Autonomous THROUGH ARC04 and explicit next-slice authority. No new human gate for routine design/intake. Root must review complete concrete packet before STATUS-ONLY issue, full CC intake and explicit ownership transfer.

FIRST mandatory source change: compact_hit currently makes occurrence_id=occurrence_key.clone() outside projection timing. After the final occurrence-key lookup MOVE that String into returned occurrence_id/remove the unnecessary clone. The final lookup is `self.indices.occurrence_attributions.get(&occurrence_key)`; immediately after it move with `let occurrence_id = occurrence_key;`, because no later key use remains. Preserve exact tuple/key/output bytes. No silent edit of739; next frozen source must close this carryover BEFORE metrics. Do not reopen other engine kernels, change selection/state/cursors/keys or optimize inherited full eager behavior.

No RETAINED FULL/G4 read/model/import/create/reverification/All or numeric campaign, new target/cache/worktree/dependency/schema/lock/model/importer/workflow change, actual-default access, network/push/PR/merge/publication. The explicitly approved owned Tiny fixture preparation/CLI install/remove/reinstall and their inherent existing verification (including Tiny internal All) remain allowed under the exact §14.3 recipe. Current readable implementation and finite Tiny proof precede the separately reviewed approved final read-only G4/numeric/Linux/fresh-ARC stages; none are waived or triggered by this prompt.

## Exact source subset (ELEVEN paths, within original21 union)

1. crates/search/src/dictionary/projection.rs — ONLY mandatory occurrence String move above; no other kernel/representation change.
2. NEW crates/search/src/presentation.rs — four reusable pure typed renderers/private formatting helpers, no engine/store/IO.
3. crates/search/src/lib.rs — export presentation module and four renderer functions only.
4. NEW crates/search/tests/presentation_contract.rs — literal golden/typed/Unicode/multiplicity/status/detail counterparts.
5. crates/cli/src/main.rs — typed format routing before provider, DetailFormat, readable transport/context and actual common write/flush handling.
6. crates/cli/src/query_output.rs — private Json/Readable transport enum; no business projection.
7. crates/cli/tests/query_output_contract.rs — actual formats/errors/native/IO/W1–W5 journey using existing fixtures/dependencies.
8. README.md; 9. crates/search/README.md; 10. crates/cli/README.md; 11. docs/collections.md — actual useful formats, omissions and executable workflow guide.

No other source/test/manifest path, public response/error/schema field, new test accessor or data fixture file. Inline bounded literal helpers may be copied from already-issued original fixtures; no planning include paths in product tests. CC appends only existing artifacts/execution.md; CDC owns other planning. One writer/source index/retained Arc01 target, not transferred at draft.

## Required intake (complete readback before edits)

Read FULL current prompt, slice-plan §§1–14 including original§§1–2/6 and all dated corrections, ledger, review-iteration02.md, planning AGENTS at /Users/oubiwann/lab/oxur/vlajmaji/.worktrees/planning/AGENTS.md and source AGENTS; prior omission/receipt interpretation mistakes stay history. Read parent arc binding useful output/W1–W5/approved numeric constraints and original preserved initial+01+02 prompts as history, not new assignments. Verify current cdc-iteration02 manifest832b9a7b…ce4f5,114payloads1110977B, current739/treee184, lock65cd…8700/guard20fe…de8/both oldS04refs432 unchanged. Read current complete public key/error/profile/response types in dictionary/projection.rs50–443, managed projection.rs28–96, ReferenceStatus/DictionaryHit, CLI QueryFormat/detail/execute four branches/output-context/main/emitter/unit helper and query_output.rs; current search/lib exports, owning READMEs/guide and existing query_output_contract fixture/tests. Exact final line spans are checked at issue; no assumption a historical module read covers changed bodies.

Load required Rust PF12/13 (no full→trim or unnecessary body clones), EH04 (existing typed errors), OB16 (explicit engine/model scope), engineering prompt/intake and PM iteration guides. Fixture imports: crates/lexicon/examples/support/dictionary_fixture.rs::complete_fixture; accepted original four-source models()/inventory() at planning S03 artifacts/cdc-review02/public-caller01/src/helpers.rs (NOT readiness01 confyg probe). Literal source data live in S03 artifacts/readiness01 and current accepted source tests. Three originals: lensisku.json used JB+La, llg-valid-row.txt, sutysisku.json. Name6raw/5lexical/3sources and source2/1/2 versus distinct Full5/3 population. Read manifests/current independent raw/public/edge/profile closure; do not scan any Full DB/model.

## Pure public API and typed serde contract

Add/reexport EXACT functions:

- render_dictionary_compact(&DictionaryCompactQueryResult) -> String
- render_collection_compact(&CollectionCompactQueryResponse) -> String
- render_dictionary_detail(&DictionaryHitDetailResponse) -> String
- render_collection_detail(&CollectionHitDetailResponse) -> String

No format parameter/public presentation errors or new public response. Input records are projection data, not validated immutable models. Functions only borrow input, have no provider/config/store/default/clock/query/detail/hydration access, do not mutate/drop/deduplicate occurrences. Never obtain full responses to trim. Do not reconstruct full DictionaryHit; complete detail already supplies it.

J(x) below means DIRECT serde_json::to_string(&typed_x), no Value roundtrip. PrettyHit means DIRECT serde_json::to_string_pretty(&DictionaryHit), preserving typed struct order/RawJson/Presence and duplicate Vec order. No flattening TextValue, no replacing Missing/Null with empty, no JSON-shape dispatch. Existing JSON-safe closed input shapes permit a documented invariant expect in one private serialization helper; do not swallow failure into null/empty. If actual source contradicts JSON safety, stop affected design rather than silently alter the four String signatures.

A(s), for ALL unquoted header atoms (source abbreviation/ID, assertion/occurrence ID, fallback metadata), is JSON STRING CONTENT: serde_json::to_string(s) with only its enclosing quote bytes removed. This preserves visible Unicode and exact escaped backslash/quote/ESC/CR/tab/LF; no normalization/trimming/ANSI/width probing/wrap/truncation/colour. Labelled values remain complete J(x), including quotes. No raw corpus string is interpolated into terminal text. Numbers use decimal usize; literal grammar words never singularize. Fixed structural newlines are LF. Every renderer result ends exactly one structural LF, no added blank footer.

## Literal readable v1 grammar

### Exact (standalone)

Line1: Exact <J(spelling)>: <totals.occurrences> occurrences, <totals.distinct_spellings> spellings, <totals.source_groups> sources
For every supplied group, in GIVEN group order, retain every hit in GIVEN order:
<A(source.abbreviation)> [<A(source.id)>] <returned_count>/<full_count><literal ' (partial)' iff group.partial>
  <A(assertion.id)> @ <A(occurrence_id)>
    spelling: <J(entry.spelling)>
    definition: <J(assertion.definition)>
    detail: <J(detail_ref)>
Final line ALWAYS: next_cursor: <J(next_cursor)>

Managed Exact is the SAME inner grammar, but inserts exactly these two lines AFTER groups and BEFORE final next_cursor:
selection: <J(outer.snapshot)>
collections: <J(outer.collections)>
No parsing/rejoining an already-rendered String: one private typed exact renderer gets optional managed metadata. Declared counts/partial flag are rendered, not recomputed from page size. Page0 prints totals/cursor and no hit/source groups; normal engine selected group order stays unchanged. Readable intentionally omits compact availability/code-notice/inventory bags and raw bodies; --format compact exposes the complete projected schema, detail obtains complete hit. It does not invent availability or change eligibility.

Literal standalone zero golden (TWO content lines, each ending LF; no blank footer):
Exact "farvi": 0 occurrences, 0 spellings, 0 sources
next_cursor: null

For nonempty golden freeze the same literal labels/indent above with synthetic typed fixture whose key is a PREDECLARED constant K (all fields in existing DictionaryHitDetailRef order); spelling Missing prints "Missing", Null prints "Null", Value uses exact externally tagged serde shape. Goldens MUST contain complete expected key JSON, not ellipses or renderer-derived expected strings. The COMPLETE literal K/nonempty Exact/Resolve/metadata-only/null-detail/pretty-hit goldens are already frozen in slice-plan §14.2; copy those exact declared fixtures/expected bytes inline in new presentation_contract.rs. Labels/order/escaping here and that appendix are binding, not a future CC output-policy decision.

### Resolve (standalone and managed)

Standalone begins exactly:
Reference <J(target)>: <PascalCase ReferenceStatus>
entity: <J(entity)>
confirmed: <confirmed.len()>
Then confirmed groups/rows below; afterwards unverified: <unverified.len()> and its groups/rows; final requested_release: <J(requested_release)>.

Group each classification INDEPENDENTLY by entry.source using ascending SourceId Ord; retain INPUT relative order and multiplicity within each source. Known abbreviation comes from FIRST exact source-ID match in that result's availability. If missing, literal ? is the label; never omit a view or invent a SourceRecord. Known empty abbreviation stays empty. Group line:
<A(abbreviation-or-?)> [<A(source_id)>] <N> confirmed
(or N unverified). For every row:
  <A(assertion.id)> @ <A(occurrence_id)>
    spelling: <J(entry.spelling)>
    definition: <J(assertion.definition)>
    eligibility: <J(assertion.eligibility)>
    payload_present: <J(bool)>
    current_eligible: <J(bool)>
    detail: <J(optional detail_ref)>

Never filter nonlexical/excluded/current_eligible=false rows. Always print both classification counts, even0; no fake Exact header for Unknown/Unresolved/NoMatch/CatalogueOnly.

Managed uses OUTER target/status header, then reason: <J(operational_reason)> and payload: true|false. Entity/groups come ONLY from payload when Some; None prints entity:null and confirmed:0/unverified:0. Requested_release is OUTER authoritative field, no fallback/preferred release. Append always:
catalogue_matches: <J(outer.catalogue_matches)>
selection: <J(outer.snapshot)>
collections: <J(outer.collections)>
The redundant nested payload target/status/requested_release are omitted, without modifying compact JSON. Retained removed/disabled/not_selected/registered-only reasons remain exactly provided.

Literal standalone Unknown golden:
Reference "Unknown": Unknown
entity: null
confirmed: 0
unverified: 0
requested_release: null

Literal confirmed/unverified/metadata-only goldens must cover source-label fallback, both classifications, excluded row and exact labelled values above. Complete typed entity/matches/requested release serialize directly; no omitted key or guessed selected release. Capture/role/release body semantics stay inherited.

### Complete detail

Both begin:
Detail <A(detail_ref.assertion_id)>: <PascalCase status>
snapshot: <J(response.snapshot_sha256)>
reason: <J(response.operational_reason)>
current_eligible: <J(response.current_eligible)>
reference: <J(response.detail_ref)>
For managed ONLY then selection:<J(outer.snapshot)>, collections:<J(outer.collections)>, catalogue_matches:<J(outer.catalogue_matches)> (one space after colon).
Final hit: null if None; otherwise hit: <PrettyHit> with first '{' on SAME line after 'hit: ', all remaining pretty JSON lines emitted unchanged and final structural LF. Complete hit fields are NEVER selected/flattened/truncated; no five-hit byte cap applies to detail. No hidden detail query to construct output. State digest/redundant version-policy metadata omitted from text; JSON routes unchanged. Snapshot-free stable key and optional observed-snapshot stale rules remain core API, not renderer decisions.

## Concrete CLI dispatch/transport recipe

QueryFormat becomes private Full|Compact|Readable, FULL default for TTY and pipes. Add separate DetailFormat Full|Readable (defaultFull); --format compact on either detail command is real Clap code2, not silent fallback. Existing strict DictionaryRequest/key/snapshot/pack/profile/error precedence unchanged, including static detail malformed material→null contexts BEFORE paths/config.

Current execute() ends by wrapping every Value as QueryOutput::Json; do not force readable through that boundary. Recommended narrow route: four DictionaryQuery/Lookup/DictionaryDetail/LookupDetail branches parse once as currently, choose constructor before setup, and return early Ok((typed QueryOutput, captured output_context)). Full existing branch computes EXACT old Value; compact JSON serializes exact current inner standalone/nested managed wire; Compact|Readable both use compact constructor/execution ONCE, then either serialize typed result or call the matching public renderer. Never inspect Value keys, never instantiate full engine for readable. Other legacy branches keep current final Json wrapper unchanged.

query_output.rs adds private Readable(String), removes as_json-only assumption. main calls emit_success_response(&QueryOutput,..). Change that existing private helper and its same-file tests to match typed output: Json writes existing serde_json::to_writer(Value), LF; Readable writes String.as_bytes EXACTLY (no extraLF); BOTH then flush through SAME existing ResponseWriteFailure/BrokenPipe/nonBroken handler. No JSON clone/new full buffering, no second independent IO/error implementation. BrokenPipe on write/newline/flush remains success; other faults fail with truthful existing contexts.

Managed Lookup/LookupDetail capture native config/ROOT/profile/knownpacks/snapshot/collections from typed response BEFORE rendering String, readonly outcome not_applicable, writer:null. No JSON-shape or postcommit/current_dir inference. Standalone detail output retains current 3-key DictionaryErrorReport mapping. For NEW readable standalone dictionary-query output failure, capture private dictionary_query origin and use same 3-key response_output_io report; existing full/compact standalone output/context compatibility is unchanged. Extend private report guard only for this additive origin, not a new public envelope. New readable does not write a mutation receipt. Keep original IO detail, stderr framing {error:report}+LF and no unrelated warning stdout.

## Finite original tests / gates / workflow

New pure library tests: exact empty/nonempty/partial/page0/cursor; literal Missing/Null/Value plus TextValue and RawJson; duplicate occurrences retained; source grouping/fallback and both confirmed/unverified/excluded/Unknown/Unresolved/NoMatch/CatalogueOnly; complete detail full serdeHit vs hit:null. Assert exact LF/indent/key bytes/deterministic repeats, Unicode, ESC/CR/tab/LF/backslash/quote in header IDs/abbreviations AND field strings; no terminal ESC bytes. Goldens are literal predeclared expected strings, never call renderer for expected. Include nonempty accepted complete_fixture Forms/editorial/contexts, not empty surrogate. Serialization directly typed, no Value-roundtrip normalization.

Actual CLI tests (existing query_output_contract path): default/explicitFull byte-identical, compact existing wire unchanged; readable Exact AND local/catalog Resolve plus both detail commands; request/key/static/context/error-before-path counterparts; DetailCompact Clap2; stdout BrokenPipe success and same real writer/flush OtherError helper + Linux write-open /dev/full for readable read-only13-key managed and3-key standalone detail/query. Existing JSON IO/committed lifecycle commands remain unchanged. Forced Linux native CONFIG+ROOT positives and all-absolute deleted-cwd use owned paths; Mac equivalent-IO qualification is not Linux proof. No actual default stat/read; isolated HOME/XDG for every product child, including negative paths.

One nonempty named four-literal Tiny W1–W5 journey uses pre-input25family/6raw5lexical/source2/1/2 and perpack1/1/1/2, then actual CLI sealed-manifest installall/configure/currentdefault lookup full/compact/readable. Exercise real typed reference+detail readable; disableLLG then fresh4hits/2sources and priorcursor stale; reenable5hits; removeLLG→metadata-only reference/detail hit:null with exact reason, then explicit release-bound reinstall→stablekey complete detail. Retain old derived generation/inputs; no implicit cleanup/fallback. Use EXACT argv/pre-input symbols/revision table already frozen in slice-plan §14.3 (catalogue/profile sequence1/absent→1/1→1/2→1/3→2/3→3/3); assert those preconditions BEFORE each dependent mutation, never derive expected revision from candidate output; expected inputs/IDs from models BEFORE candidate, no candidate-picked oracle or stock6hit fixture substitution. Doc guide shows these actual command routes, defaults/explicit STORE precedence, omissions and detail/reference states. Full-vsTiny populations distinct; W1–W5 claims only after actual process journey.

FIRST fix carryover and prove output/key bytes unchanged via existing original fivehit/page/detail counterparts and source inspection. Original indices/profile/None/full eager semantics stay unchanged; run relevant existing projection/reference/full-wire tests, not a new engine audit. Preserve all current739API/general/edge/parity/CLI-at-c364 evidence with exact delta applicability; no relabelled descendant results.

Formatting: ONLY pinned stdin→rustfmt --edition2024 --emit stdout→exclusive scratch→copy exact allowedpath; no filepath recursion/unstable skip_children/workspace mutating makeformat (prior approval rejection retained). Read-only pinned make fmt-check, make lint BEFORE behavior. Final stable fmt-check→lint→make check, isolated search no-default, Syntheticallfeatures and search/CLI package --list --allow-dirty --locked --offline (no verify/publish). Explicit RUSTUP_TOOLCHAIN1.98.0/MakeRUST_TOOLCHAIN1.98.0/directguard+1.98.0, unchanged CARGO_HOME/one target. Unique s05-i03 labels, fresh verified-absent owned roots; retain every failure/zero-match/method/setup/approval attempt. No broad repeats at stable same-source absent change/failure.

Return clean actual commit/parent739/tree/branch, exact11subset/cumulative21allowed boundary, unchanged lock/guard/protectedrefs, actual adjacent2trailers, timer carryover closure and fourrenderer/typedtransport source inventory, current gate/raw population/coverage implemented-vs-executed-vs-pending. All sessions terminal then explicit source/index/target handback. CC appends existing execution.md only; no closing/verifier/retro placeholders. CDC performs original independent Tiny/library/realCLI proof; root reviews frozen contribution. Then separate exact final read-only G4/numeric/platform/freshARC prerequisites, not automatically launched by this pass. No automatic fifth repair or external delivery.
