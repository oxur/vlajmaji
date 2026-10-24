# Arc02 S01 — Lossless export normalization

| Field | Value |
| --- | --- |
| status/version | OPEN/repair required after frozen candidate01 independent review /1.2,2026-10-24 |
| source | candidate01 23d057bf (initial d1e1c928); feature/project01-full-jbotci-import; isolated Arc02checkout |
| depends-on | A01Dictionary/registry, current baseline and full pinned raw+metadata |
| delivery | Public Rust importer + local CLI import/check + complete normalized JSON usable before SQLite |
| current assignment | cc-prompt-iteration02.md (first executed refinement of5; candidate01 not accepted) |
| artifact home | artifacts/; CC writes only artifacts/cc-execution.md; CDC owns canonical planning |

Provide lossless fullrawexport normalization and useful Rust/CLI entrypoints.
No acquisition scripting until S02. Full actual33526rows is exercised now; no
SQLite dependency in lexicon. All inherited model/legacy/query/version/terms
constraints are in parent arcplan/directive02 and remain binding.

## Binding API, identities and failure rules

Add private `crates/lexicon/src/jbotci.rs`, explicit root exports:
`JbotciImportOptions`, `JbotciImport`, `JbotciImportReport`, `JbotciImportError`,
`import_jbotci_export`, and constants `JBOTCI_PINNED_REVISION`, `JBOTCI_PINNED_RAW_SHA256`,
`JBOTCI_PINNED_METADATA_SHA256`. Options implements Default for this pinned triple.
Options is passive draft with revision, expected_raw_sha256,
expected_metadata_sha256 Strings; validate40lowerhex revision/64lowerhex digests
on import. `import_jbotci_export(raw: &[u8], metadata: &[u8], options:
&JbotciImportOptions) -> Result<JbotciImport,JbotciImportError>` is pure/noI/O.
Result private fields, dictionary()/report() borrows and into_dictionary() ownership.
Report is a public serializable passive record with exact fields: `version: u32` (=1),
`revision`, `raw_sha256`, `metadata_sha256`: String; `raw_bytes`, `metadata_bytes`,
`input_rows`, `native_fields`, `normalized_assertions`, `lexical_occurrences`,
`excluded_occurrences`, `eligible_spellings`, `all_spellings`, `contributors`,
`supplier_definition_count`, `supplier_entry_count`: u64;
`word_types`, `field_counts`, `diagnostics`: BTreeMap<String,u64>;
`definition_count_matches`, `entry_count_matches`: bool. Report diagnostics counts
are counts per code, distinct from excluded-row counts. Raw valid spellings for
all_spellings count every stringword including empty; eligible_spellings uses
eligibility. Malformed word_type uses no fabricated class key. Metadata fields
remain separately captured. `JbotciImportError` variants are `InvalidOptions(String)`,
`HashMismatch { input: String, expected: String, actual: String }`,
`InvalidUtf8 { input: String }`, `InvalidJson(String)`, `InvalidMetadata(String)`,
`UnsupportedMetadata(String)`, `Registry(RegistryError)`, `Dictionary(DictionaryError)`.
Use input labels raw/metadata; unsupported metadata language/format is separate
from malformed/duplicate/missing-key metadata. Syntax/root errors are InvalidJson. No public unchecked Deserialize for invarianttypes.
Typed nonexhaustive errors distinguish invalidoptions, raw/meta hash mismatch,
UTF8/JSONroot/syntax, invalid/unsupported metadata and registry/model validation;
implement Error/Display/source where applicable. Do not add thiserror just for derive.
On input error, no callerstate/files mutate and no panic. Unsupported valid rawrow
shapes produce retained excluded assertions/diagnostics, not whole-import failure.

Metadata is the pinned supplier's12key simple assignment format, not a general
TOMLAPI. Implement a documented deliberately bounded parser: blank/commentlines,
unique barekeys, values JSON-compatible doublequotedstrings/booleans/nonnegative
u64integers. Required keys are all12observedkeys; forbid duplicatekeys, malformed
value/trailingjunk and unsupported required shapes. Unknown simplekeys remain
captured and diagnosed, never silently overwritten. No section/multiline/general
TOMLclaim; a future unsupportedsource change yields named metadataerror. Require
language_tag=en,source_language_tag=jbo,format=json,filename=dictionary-en.json,
positive_scores_only=false. Verify metadata.sha256 matches actualraw AND options
expectedraw; metadata bytes match expectedmetadataSHA. Preserve all metadata bytes
in a second RawCapture with zerooccurrences, bound to same source/distributor and
listed by release; release accounting counts rawrowcapture only. Input timestamp
is unknown unless explicitly supplied; do not misuse lensisku_created_at as capture
or assertion date. Preserve suppliercreatedat/retrievalURLs as metadata facts.

Rawsource namespace `captured:lensisku:jbotci:en`, metadata namespace same plus
`:metadata`. Capture format labels explicitly export-v1 and supplier-metadata-v1,
not legacy API. Derived IDs include fullrawdigest+rowindex and role prefix (e.g.
`jbotci-<64hex>-entry-13306` /assertion/form/context roles) so same input is stable,
new snapshot distinct, every duplicate separatelyaddressable. One EntryRecord
per rawrow intentionally avoids inventing nativewordidentity or merging conflicting
class/selmaho; one AssertionRecord perrawrow, locators `/0`...`/33525`.
Registry source `lensisku`, display Lensisku, abbreviation LSK ProjectAssigned;
distributor `jbotci`, separate source lineage unresolved jbovlaste URL with pinned
suppliernotice evidence. Snapshot bindings for both captures. Every occurrence
attributed; nativeentryIDNone, nativeassertionID exact definition_id lexical value
when scalar string/number (not a sense); native_sense_idMissing. Contributors use
exact source-scoped username, stable hash-ID, displayusername when registry-valid;
otherwise explicit fallback display/hash+diagnostic with rawname intact. Missing/
null user means contributorNone, never fabricatedname. Realnames preservedraw.
Use BTreeMap/sets once for membership/grouping, avoid repeatedfullcorpus scans.

## Raw and normalization rules

Parse valid rootarray into rawvalues and custom orderedobjectfields, retaining
originalfullcapturebytes and each field's exact rawJSONlexeme, including order,
unknownkeys/nestednull/largeintegers/repeatedkeys. Nonobjectrow gets one NativeField
`$row` containing exact JSONvalue and excluded assertion. Repeatedkey at anydepth
retains rawlexemes and explicit duplicate-key diagnostic, with no ambiguous typed
projection/lexical hit. Raw unknown-field extensions remain indexed evidence even
on duplicate-key object rows; suppressing typed semantics does not suppress these
raw extensions/diagnostics. Do not call legacy parse_object which rejects repeatedkeys.
Rawrootnonarray/syntax/UTF8 is typedfailure; do not reinterpret syntaxfailure as a
partial successful release. Parsing/metadata/registry/modelvalidation precede return.

Eligibility policy version1: recognized nonwiki class, nonempty string spelling
and non-whitespace definition string, no duplicatekey/invalidcriticalfield. Exact
supportedclasses are raw-census's16nonwiki classes (including nalvla/obsolete/
phrase); do not claim morphology validates them. Wiki, unknownclass, missing/null/
wrongtype/empty critical word/word_type/definition retained with Excludedreason
and perrowdiagnostic. Low/negative score alone never excludes. Other malformed
optionalfield types retainraw, get diagnostic, skip only that typedprojection;
whole row stays lexical when criticalfields valid. Duplicateknown data never
selects a winner. Input definition-count mismatch and expected selected-entry
mismatch are diagnostics/flags, not rowdrop or forcedoracle. Observed exacteligible
spellings is separately labelled as our comparator, not compiledwinner contents.

Known field map (preserve missing/null/empty via Presence where typedhome exists):

| Native input | Normalized home / boundary |
| --- | --- |
| word | Entry.spelling exactbytes, language source_language_tag=jbo; no script/POS inference |
| word_type /selmaho | native_word_class/selmaho exactpresence; unknownclass preserved excluded |
| definition | Assertion.definition TextValue language_tag=en, markupMissing (do not infer rendering); empty/null preserved while eligibility excludes |
| definition_id | registry nativeassertionID; rawlexeme authoritative; not native_sense_id |
| score | Assertion.source_score RawJson exactlexeme/null/missing; no winner |
| notes | ContextKind::Note if fieldpresent (null/empty retained), fieldindex evidence; no prose-derived relations required here |
| jargon | ContextKind::Domain if present with exactnativevalue, no controlled vocabulary claim |
| gloss_keywords | each validobject word→Gloss Context preservingarrayorder, duplicates and emptyword; wholeexact object as ExtensionRecord on that context, including meaningmissing/null/empty/unknown; invalidobjects diagnosed+wholefieldraw retained |
| place_keywords | assertion ExtensionRecord `lensisku.place_keywords` containing exactwholefield; keywords are not structuredargumentframes; no parsedplaces |
| rafsi /experimental_rafsi | split string whitespace or supplied stringarray into ordered SourceReportedForm rows; retain all4letterforms/duplicates; standing is exact sourcecolumnname, not LLGAssignedRafsi; evidence exactfieldindex; sourcefieldraw keeps whitespace/empties |
| etymology | EtymologyRecord statement if present inclnull/empty, source evidence; no components inferred |
| user | Contributor attribution as above; exactnesteduser remains raw; no realnamecollapse |
| unknownfields /unsupportedknownshape | exactnativefields always; unknownfields also assertionextension with exactfieldindex/rawvalue + diagnostic; optionalunsupported shape diagnostic names originalfield |

Do not populate frames, sensegroupings, ownedrevisions, computations/annotations,
newrelations or dates absent fromsource. All12modelhomes stay available. CodeNotice
Apache2 describes Vlajmajicode separately. ContentNotice explicitly quotes supplier
publicdomain claim as supplierassertion with pinnedTHIRD-PARTY-NOTICES locator
`https://github.com/int19h/jbotci/blob/3d9fd41d9ed32de26311ee27ec86b6b165985aa1/THIRD-PARTY-NOTICES.md#L242-L252`;
code notice locator is actual `LICENSE`. The supplier witness remains that known
revision even for a provenance-only options.revision change;
license field must say supplierclaimed, redistributionHeld with explanation pending
publicauthorization/sourcecoverage. Release sourceLensisku/distributorjbotci,
collection/component stableIDs; releaseID revision+fullraw+metadata digest (distinctmetadata
revision can change identity), distributor_revisionoptions.revision,
upstream_revisionMissing, captured_atMissing/precisionUnknown, normalizer
`vlajmaji-jbotci-export`,normalizer_version1, schema_version1. Bothrawcaptures listed,
allassertion IDs, exactobserved input/normalized/excludedcounts, diagnostics,
selected=true/Available. Raw/export, normalizeddocument and storedschema versions
remain separate. Add releaseextension retaining exact suppliermetadata string or
explicitmetadata capture reference (both rawbytes already preserved); do not infer
the supplier selectedcount as actual assertion count.

## CLI and user delivery

Add `vlajmaji jbotci-import RAW METADATA OUTPUT` and `jbotci-check RAW METADATA`
with optional `--release PATH` only forcheck. Both have `--revision`,
`--raw-sha256`, `--metadata-sha256` explicit optional defaults=pinnedconstants.
Use clapderive; add direct lexiconworkspace dependency to CLI, remove duplicate
dev-dep only if made ordinarydependency; lock graph should not need newpackage.
Import validates/serializes fully, then create_new OUTPUT and write/flush;
existingfile/symlink/directory neverreplaced. A midwriteIO failure can leave new
partialoutput, explicitly documented/errored; do not delete caller-ownedpaths.
Check reimports verifiedbytes, validates actualaccounting and suppliercomparison;
if --release supplied, loadandcompare complete Dictionary to regenerated model.
No writes oncheck. Countmismatch imports a truthfulreport/release but check returns
structurederror `jbotci_accounting`; changedrelease gives`jbotci_release_mismatch`.
Other runtime import errors map respectively to `jbotci_options`,
`jbotci_hash_mismatch`, `jbotci_utf8`, `jbotci_json`, `jbotci_metadata` (both
metadata variants), `jbotci_registry`, `jbotci_dictionary`; filesystem uses `io`.
These use existing `{error:{code,message,sources:[]}}` stderr envelope,
nonzero/empty stdout. Success stdout JSONoperation/version/report/output where
applicable; no rawcorpusdump. Existing CLIwire behavior retained.
Docs fieldmap/limitations/replay/APIexample and README/current0.3.0changelog.
Do not bundle fullsource. Public example importer accepts paths and freshoutput,
no network, uses publicAPI; its symbol names/argumentlayout localdiscretion.

## Verification, artifacts and exit

Ledger7rows includes directfullpopulation fields/native map, not only roundtrip.
Pinned actual import33526assertions/33516eligible/10excluded/31248spellings,
rawfield278254,238contributors whenallusernamesvalid; normalize33526entries under
one-per-rowpolicy. Preserve all39721glossobject contexts,28280notes,8356jargon,
2085etymologies; placekeywords5193objects remain separately retained. Rawscore,
farvi2/conflicting4/5prose, cpana cpan, si'eMOI, ma's88840experimentalmaz despite
losing supplierselection all checked fromactualsource, not inventedfixtures.
Both raw+metadata byteequality after Dictionaryreload; eachrawfieldlexeme/order
compared independently across fullpopulation. Typedmapping derivedsource separate
fromwriter code. FullpureRustoutsideconsumer unique package/bin plus actualCLI
import/check; noSQLite prerequisite. FullJSON retainedonly localtaskdata; artifact
logs/receipts small inthisslice. Partial/gap/unrunresults stayexplicit.

Syntheticpositivefixtures cover duplicatecompleterows/nativeIDs, conflictingdefs,
unknownfields/nestedbignumber/null/empty/missing, malformedoptional vs critical,
wikiwithrafsi/selmaho retained/excluded, unknownclass +validcounterpart,
nonobjectrow, repeatedkeysatdepth, all16classes/negative score, rafsi4letter and
crosscolumnsameform retaineddistinct. NegativeAPI/CLImetadata/UTF8/root/hash/options/
duplicatekeymetadata/countmismatch/releasetamper/occupiedoutput all have named
expectedbranches and no unintendedmutation; countmismatch remains completeimport
before checkreject. Existingv1/v2 identity/regressions and realformat→lint→check,
isolatedno-default/allfeatures required. CC records exactselectedtestdenominators.
Artifacts: CC only `artifacts/cc-execution.md` plus task-owned /tmp
`vlajmaji-p01-a02-s01-cc-*`; no canonicalplanningwrites/index. Return perrowproposals,
commands/failures/dirty/hashstate/intake/readback, sourcecommit andfreeze; CDC
independently reviewsactualdiff/controls. No source edits by CDC. Sourceallowed
list inprompt; no deps/framework/statuswaivers.

## Author readiness and assignment history

CDC fully inspected raw/model/CLI/registrysource and requiredguidebodies;
parent artifacts/readiness.md records actualextents/querycoverage. API/private
validation/explicitexportformat, conservative typedmap, duplicate-retaining
eligibility/error order and noI/O/fullmetadatahash choices are resolved above.
Applied API02/04/23/27/39, EH01/04/07–14, TD05–07, PS02/06/11/13/25–26,
CLI05/06/10/11/16–20/27/55/56 and AP09/18/25/33/54. Existingalwaysserde is a
justified API18SHOULD exception: serializedDictionary is the core existingcontract,
no newserdefeature migration. No thiserror dependency needed; handwrittenerrors
match source. SansIOpurebytes letscallers reuse beforeSQLite. Field/raw map
and full33526source-derived oracles reject selecteddictionary and wrongAPIkey
implementations; duplicatecounterpart avoids vacuous no-duplicatespin proof.
Prompt carries exactmanifest/intake. Packet sizing: relevantfullsourcecore22KB+
currentmodel/registrysections and boundedguidebodies plus clearrecipe; retain
headroom by not requiring unrelatedA01history or wholeupstream337KBnotice.
Consequentialdecisions not delegated toCC; localhelpermechanics remain local.

| Assignment | Date | Predecessor/reason | Disposition |
| --- | --- | --- | --- |
| cc-prompt.md |2026-10-24 | Initial useful full export normalizer/API/CLI from verified d1e1c928/84c1694 | superseded before dependent edits; intake retained |
| cc-prompt-iteration01.md |2026-10-24 | Pre-edit clarification of existing revision+raw+metadata release identity; root surfaced provenance-only update interaction | superseded by iteration02 after candidate01 review; original intake/source evidence retained |
| cc-prompt-iteration02.md |2026-10-24 | Frozen23d057bf independently reviewed: optional diagnostics, malformed metadata, duplicate-row raw extension, truthful pinned notices, useful example, CLI regression and filename corrections | current/issued; first executed refinement of5, all7rowsOPEN |

## Version history

-1.0 —2026-10-24: source-grounded full normalization API/CLI open set issued.
-1.1 —2026-10-24: root review highlighted explicit provenance-only update oracle; preserved iteration01 supersedes unstartedinitial assignment. Existing revision-aware release identity retained; exact raw tuples/row IDs stay stable for byte-identical distribution updates. No product defect or source change, no acceptance/scope reduction.

-1.2 —2026-10-24: candidate01 proposed-done rejected at independent controls; actual passing915570 full-field checks/30CLI controls and failed cases preserved in artifacts/cdc-review-attempt01.md. Iteration02 binds repairs without reducing scope. Full raw input unchanged, no S02 implementation issued.
