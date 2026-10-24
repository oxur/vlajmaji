# Versioned dictionary store schema 1

`vlajmaji_store::DictionaryStore` persists a validated
`vlajmaji_lexicon::Dictionary` in a fresh directory containing
`dictionary.sqlite3`. Schema 1 is an explicit relational projection. The
`canonical_document` JSON BLOB and optional exact `legacy_archive` BLOB are
rebuild inputs, not a substitute for querying the typed tables. Opening a
store compares every projected SQLite value with a freshly validated model.

## Field map

All lexical text is stored as SQLite `TEXT COLLATE BINARY`; source bytes and
canonical JSON use `BLOB`. The projection retains vector order in `ordinal`
columns and preserves `Missing`, `Null`, and `Value` as a state plus nullable
value. Present empty text remains different from absent text. Unsigned values
that can exceed SQLite's signed integer range use canonical decimal text.

| Dictionary area | Schema 1 tables and retained fields |
| --- | --- |
| Registry | `sources` retains stable ID, display name, exact abbreviation/kind/evidence and origin; `contributors`, `distributors`, `source_aliases`, `source_lineage` preserve their own IDs, ordered history, status and unresolved targets. `snapshot_bindings` binds source/distributor to a capture identity. `occurrence_attributions` keeps optional contributor/native entry/native assertion IDs, including repeated IDs and empty supplied strings. |
| Capture and raw records | `capture_refs` is the union of known identities. `raw_captures` stores exact source bytes, source metadata, revision/time/notice presence. `occurrence_refs` stores the existing full length-prefixed tuple encoding as lowercase hex (not a hash), including known absent locators; `raw_occurrences` and `native_fields` contain loaded rows and ordered exact raw values only. |
| Catalogue | `collections`, `components`, `component_languages`, `releases`, `release_captures`, `release_assertions`, `release_diagnostics`, `content_notices`, `redistribution_evidence`, and `code_notices` retain source ownership, names, language order, release revisions, normalizer, accounting, availability, notices and rights statements. Code notices remain separate from content notices; no permission is inferred. |
| Entries and assertions | `entries` stores spelling/language/script/native class/selma'o presence independently from optional explicit POS mapping and rule. `assertions` stores entry and occurrence identity, native sense ID, language, definition text/language/markup, raw score, eligibility and exclusion reason. `assertion_timestamps` retains each named raw timestamp and precision. Equal spellings, native IDs and definitions do not merge rows. |
| Context and frames | `contexts` retains all eight context kinds, nested text presence, translation link, and required evidence. `argument_frames` and ordered `argument_slots` retain supplied structured frame labels, slot order, descriptions, grammatical role and source evidence; prose is not parsed into a frame. |
| Forms | `forms` retains exact spelling, language, script, pronunciation, one of seven explicit kinds, standing, optional evidence and computation link. `form_components` preserves ordered spelling and optional target for Entry, Assertion, Form, or EditorialSense. Assigned rafsi, source-reported forms and computed candidates remain separate claims. |
| Etymology | `etymologies` retains assertion, statement presence, native kind and required evidence. `etymology_components` retains ordered components and all four optional `LocalRef` roles. It does not infer morphology or fill missing source statements. |
| Relations | `relations` records typed origin, relation kind/direction/scope, and one separate target shape: Local, Catalog, Unknown, or Unresolved. Catalog targets retain source/release/native IDs and optional occurrence ref; unresolved text remains explicit. Evidence and editorial revision remain distinct. |
| Owned history | `owned_revisions`, `revision_assertions`, `revision_tombstones`, and `owned_heads` preserve source, parent, editor, timestamp/rationale presence, ordered memberships and tombstones. `Dictionary` validation checks same-source ancestry and current-head semantics; SQL FKs ensure referenced rows exist. |
| Editorial | `editorial_senses` and ordered `editorial_members` preserve explicitly grouped assertion IDs, editor, revision and rationale. No grouping is inferred from spelling or source. |
| Computation | `computations`, `computation_inputs`, `annotations`, and `extensions` keep tool/version/policy/profile/explanation, occurrence/entity/literal input variants, typed target, raw JSON values and optional evidence separately. Raw JSON is retained as its exact validated lexical string. |

Every model entity has a role row in `entity_ids`; role-table rows reference the
matching `(id, role)` pair. Ordered child tables have parent/ordinal keys and
the appropriate direct foreign keys. Four indexes cover exact spelling,
source/spelling, assertion entry, and reverse release assertion membership.
The schema has no generic entity-value table. SQL enforces declared CHECK,
UNIQUE and foreign-key constraints; the validated `Dictionary` enforces richer
cross-row rules such as capture hash identity, source membership, evidence
ranges, owned ancestry, release accounting and availability. Reopen additionally
checks the exact schema catalog and the complete projection, catching valid
shape changes that satisfy SQL constraints.

## Lifecycle and trust boundary

`DictionaryStore::create` validates the model before reserving a new leaf under
an existing parent. It refuses files, directories and symlinks already at that
leaf and never replaces caller data. It writes a transaction with an incomplete
marker, commits the exact canonical document and all typed rows, then closes
the writer and returns only after strict reopen succeeds. A failed create can
leave a caller-owned failed directory; inspect it and choose whether to remove
it. Open never repairs, migrates or creates files.

`DictionaryStore::open` opens SQLite read-only and pins a deferred read snapshot
before reading metadata. It checks format/version/completion, exact DDL,
canonical bytes/hash/model, foreign keys, and all rows and cells in every table.
It returns no partial model on failure. The successful store retains that
read-only connection/snapshot until drop. This lifecycle is an integrity check
for accidental or invalid changes, not an adversarial-file security boundary.

To persist a model loaded from the accepted versioned JSON:

```rust,no_run
use std::path::Path;
use vlajmaji_lexicon::Dictionary;
use vlajmaji_store::DictionaryStore;

let dictionary = Dictionary::from_json_bytes(&std::fs::read("dictionary.json")?)?;
let store = DictionaryStore::create(Path::new("data/dictionary"), &dictionary)?;
drop(store);
let reopened = DictionaryStore::open("data/dictionary")?;
assert_eq!(reopened.dictionary().document(), dictionary.document());
# Ok::<(), Box<dyn std::error::Error>>(())
```

The `dictionary_store` example performs the same create/drop/reopen cycle from
an input JSON document. To rebuild from a legacy schema-1 SQLite index, first
read its untouched `SqliteStore::archive_bytes()`, supply an explicit validated
`SourceRegistry` and `DictionaryMetadata` to
`Dictionary::from_legacy_archive`, then create a *different fresh directory*.
The adapter does not infer identities or metadata. Opaque rows not represented
in the accepted legacy archive remain new-model-only and must not be fabricated
as legacy input.


## Exact relational columns and constraints

This concrete inventory names the schema-1 columns and reference/order contracts. The DDL in `crates/store/src/dictionary_schema.rs` is authoritative; this inventory expands presence/reference notation and records the intended SQL constraints.

Notation: TEXT is exact BINARY UTF8; BLOB exact bytes; nullable columns preserve Option None versus Some(empty). All vector ordinals are checked nonnegative SQLite INTEGER, unique in their owning vector. u32 uses checked INTEGER 0..4294967295. Full u64/usize values use canonical unsigned decimal TEXT, nullable only for Option, never a lossy i64 cast. Check canonical digits/no leading zeros/range losslessly or rely on exact validated projection with a documented SQL constraint boundary; no float/coercion. Canonical projection must preserve embedded NUL as UTF8 bytes; SQL length(TEXT) is not byte-length (local reference SQLite3.39.4 gave NUL text0/blob1), so avoid invalid stronger emptiness checks or use BLOB byte length. Actual pinned rusqlite controls verify preservation.

`P(x)` expands `x_state TEXT NOT NULL` in missing/null/value and `x_value TEXT NULL`, requiring value non-NULL exactly at state=value, allowing empty string. For Presence<RawJson> the value is its exact raw lexical string. `TP(x)` for Presence<TextValue> expands outer x_state plus nullable x_text/x_language_state/x_language_value/x_markup_state/x_markup_value; all payload columns NULL outside outer value, within value text and both inner states required with their own P consistency. Nonoptional TextValue `TV(x)` has x_text NOT NULL plus P(x_language),P(x_markup). `PM(x)` is Presence<PosMapping>: state plus x_value/x_rule, both supplied exactly when value. No XML/rendering/linguistic inference.

`T(x)` for a LocalRef expands x_kind and nullable x_entry_id/x_assertion_id/x_form_id/x_editorial_sense_id, each direct FK to its role table; exactly the named role's column is present. Required T has non-NULL kind; optional T has all NULL when absent. All four roles allowed for BOTH form and etymology components. `E(x)` expands `x_id INTEGER` as a required/optional FK to source_evidence.evidence_id (thus E(evidence) is evidence_id). Use explicit CHECK predicates with IS NULL/IS NOT NULL; do not let SQL NULL's three-valued logic bypass role/presence constraints. Role tables' id columns FK `(id,entity_role)` to entity_ids with constant role CHECK, preventing a role-table row from masquerading under another entity role. All real FKs enforced and DEFERRABLE INITIALLY DEFERRED where forward references/cycles require; no disabling enforcement.

| Concrete table | Columns and key/reference/order contract |
| --- | --- |
| metadata | key TEXT PK,value TEXT NOT NULL; exact keys format/schema_version/complete/canonical_sha256/dictionary_version/registry_version/catalogue_revision. Dictionary/registry versions decimal u32; catalogue_revision full u64 decimal. |
| canonical_document | id INTEGER PK CHECK1,json BLOB NOT NULL,legacy_archive BLOB NULL; exactly one row; legacy BLOB byte-equals doc Option. JSON is rebuild source, not typed-schema substitute. |
| entity_ids | id TEXT PK,role TEXT NOT NULL,ordinal INTEGER NOT NULL; UNIQUE(id,role),UNIQUE(role,ordinal). Roles collection/component/release/entry/assertion/context/frame/form/etymology/relation/owned_revision/editorial_sense/computation/annotation. Registry entities remain separate typed domains. |
| sources | id PK,ordinal UNIQUE,display_name,abbreviation UNIQUE,abbreviation_kind,abbreviation_evidence NULL,origin. Exact accepted Conventional/ProjectAssigned and Upstream/OwnedDraft vocabularies. |
| contributors; distributors | each id PK,ordinal UNIQUE,display_name. Native IDs never become these project IDs automatically. |
| source_aliases | ordinal PK,source_id FK,text,status Active/Retired,evidence NULL. No global alias-text UNIQUE: historical alias reuse/ambiguity is intentional. |
| source_lineage | ordinal PK,source_id FK,target_kind Known/Unresolved,upstream_source_id FK NULL,unresolved_text NULL,evidence NULL; exclusive target shape. |
| capture_refs | namespace TEXT,sha256 TEXT,PK(namespace,sha256); union of loaded/registry/release/known reference identities, not fabricated payload rows. Namespace may be legacy empty. |
| snapshot_bindings | namespace,sha256,ordinal UNIQUE,source_id FK,distributor_id FK NULL; PK tuple and FK capture_refs. Bindings distinct from expected capture refs. |
| occurrence_refs | id TEXT PK,namespace,sha256,locator TEXT; UNIQUE(namespace,sha256,locator),UNIQUE(id,namespace,sha256),FK capture_refs. id is existing record_id exact length-prefixed tuple-hex, NOT a hash. Includes known absent locator refs. |
| occurrence_attributions | occurrence_id PK/FK,ordinal UNIQUE,contributor_id FK NULL,native_entry_id TEXT NULL,native_assertion_id TEXT NULL. Some(empty) stays empty; no text/native-ID deduplication. |
| raw_captures | namespace,sha256,ordinal UNIQUE,bytes BLOB,source_name,format_label,locator,P(revision),P(captured_at),time_precision,P(notice_locator); PK tuple/FK snapshot_bindings, preserving empty metadata. |
| raw_occurrences | occurrence_id PK,namespace,sha256,ordinal,raw_line TEXT NULL; FK(occurrence_id,namespace,sha256) to occurrence_refs and FK tuple to raw_captures; UNIQUE(namespace,sha256,ordinal). Loaded subset only. |
| native_fields | occurrence_id FK raw_occurrences,ordinal,name,encoding Json/Text,raw_value TEXT; PK(parent,ordinal). Repeated names and exact lexemes/lines retain order. |
| collections | id+constant entity_role collection,source_id FK,display_name,TP(description). |
| components | id+role component,collection_id FK,native_name. |
| component_languages | component_id FK,ordinal,language TEXT; PK(parent,ordinal), preserve duplicates/order. |
| releases | id+role release,component_id FK,distributor_id FK NULL,P(upstream_revision),P(distributor_revision),P(captured_at),time_precision,schema_version u32,normalizer,normalizer_version,accounting_input_occurrences/accounting_normalized_occurrences/accounting_query_excluded_occurrences nullable unsigned TEXT,accounting_unit,selected INTEGER0/1,availability Available/Disabled/Missing/Removed,redistribution Unknown/Held/Permitted. |
| release_captures | release_id FK,ordinal,namespace,sha256 FK capture_refs; PK(parent,ordinal),UNIQUE(release_id,namespace,sha256). Shared capture across releases allowed. |
| release_assertions | release_id FK,ordinal,assertion_id FK; PK(parent,ordinal),UNIQUE(release_id,assertion_id). |
| release_diagnostics | release_id FK,ordinal,occurrence_id FK NULL,code,message; PK(parent,ordinal). |
| code_notices | ordinal PK,text TEXT NOT NULL,P(locator),P(license), separate from content rights. |
| content_notices | release_id FK,ordinal,P(text),P(locator),P(license); PK(parent,ordinal). |
| redistribution_evidence | release_id FK,ordinal,evidence TEXT; PK(parent,ordinal). No inferred permission. |
| entries | id+role entry,source_id FK,P(spelling),P(language),P(script),P(native_word_class),P(selmaho),PM(mapped_pos). No UNIQUE(source,spelling) or inferred POS. |
| assertions | id+role assertion,entry_id FK,occurrence_id UNIQUE/FK,P(native_sense_id),P(language),TP(definition),P(source_score),eligibility Lexical/Excluded,exclusion_reason TEXT NULL. Definition/native assertion/native sense remain distinct. |
| assertion_timestamps | assertion_id FK,ordinal,field,value TEXT exact RawJson,precision; PK(parent,ordinal). Keep epoch/created_at separately. |
| source_evidence | evidence_id INTEGER PK,occurrence_id FK,field_index nullable unsigned TEXT,P(citation),offset_start/offset_end nullable unsigned TEXT (both or neither). Deterministic projection-local evidence IDs, no invented lexical identity. |
| contexts | id+role context,assertion_id FK,kind Gloss/Note/Label/Register/Domain/Example/Translation/Citation,TP(value),translated_from FK contexts NULL,E(evidence) required. |
| argument_frames | id+role frame,assertion_id FK,E(evidence) required. |
| argument_slots | frame_id FK,ordinal,slot_order u32,native_label,TP(description),P(grammatical_role); PK(parent,ordinal),UNIQUE(frame_id,slot_order). Vector and supplied order separate. |
| forms | id+role form,entry_id FK,spelling,P(language),P(script),TP(pronunciation),kind exact7 FormKind values,P(standing),E(evidence) optional,computation_id FK NULL. Source kinds require evidence/no computation; computed kinds inverse. |
| form_components | form_id FK,ordinal,component_order u32,spelling,T(target) optional; PK(parent,ordinal),UNIQUE(parent,component_order). |
| etymologies | id+role etymology,assertion_id FK,TP(statement),P(native_kind),E(evidence) required; explicit null/empty/source statement separate from inference. |
| etymology_components | etymology_id FK,ordinal,component_order u32,spelling,T(target) optional (ALL4 roles); PK(parent,ordinal),UNIQUE(parent,component_order). |
| relations | id+role relation,T(origin) required,kind SourceSeeAlso/SourceEtymology/EditorialLink/Other,other_kind TEXT NULL,direction Forward/Reverse/Bidirectional/Unspecified,scope Entry/Assertion/Form/Editorial,target_kind Local/Catalog/Unknown/Unresolved,T(local_target) optional,catalog_source_id FK NULL,catalog_release_id FK NULL,catalog_native_entry_id/catalog_native_assertion_id TEXT NULL,catalog_occurrence_id FK NULL,unresolved_text NULL,E(evidence) optional,editorial_revision_id FK NULL. Enforce mutually exclusive target/provenance shape; Other may have neither but not both evidence/revision. Source kinds require evidence; EditorialLink revision. |
| owned_revisions | id+role owned_revision,source_id FK,parent_id FK self NULL,editor_id FK contributors,P(timestamp),time_precision,TP(rationale). |
| revision_assertions; revision_tombstones | each revision_id FK,ordinal,assertion_id FK; PK(parent,ordinal),UNIQUE(parent,assertion_id). |
| owned_heads | source_id PK/FK,ordinal UNIQUE,revision_id FK. One head per source; model checks same-source/current history. |
| editorial_senses | id+role editorial_sense,editor_id FK,revision_id FK,TV(rationale). |
| editorial_members | editorial_sense_id FK,ordinal,assertion_id FK; PK(parent,ordinal). Retain vector, no invented unique membership rule. |
| computations | id+role computation,tool,tool_version,policy,P(profile),TP(explanation). |
| computation_inputs | computation_id FK,ordinal,input_kind Occurrence/Entity/Literal,occurrence_id FK NULL,T(entity) optional,literal TEXT NULL; PK(parent,ordinal), exclusive shape. |
| annotations | id+role annotation,computation_id FK,T(target) required,kind,value TEXT exact RawJson,TP(explanation). |
| extensions | ordinal PK,owner_id FK entity_ids,name,value TEXT exact RawJson,E(evidence) optional. Owner can be any metadata entity, not just LocalRef roles. |

