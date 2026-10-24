# Source registry and occurrence attribution

`vlajmaji-lexicon` can attach explicit source, contributor, and distributor
identity to the immutable captures it already imports. The registry is a
versioned sidecar; it does not rewrite `SourceMetadata`, source bytes, archive
versions, `SnapshotIdentity`, `RecordIdentity`, or the store's encoded record
IDs. One source may have multiple distributor snapshots, and matching text or
source-native IDs never collapse separate captured occurrences.

## Registering sources

`SourceId`, `ContributorId`, and `DistributorId` are distinct validated string
newtypes. Construct them with `new` or `TryFrom<String>`. The accepted spelling
is one or more lowercase ASCII letters or digits followed by zero or more
lowercase letters, digits, `.`, `_`, `:`, or `-`; no trimming or normalization
occurs. The same literal may occur in different ID roles without making those
roles interchangeable.

A `RegistryDocument` is the unvalidated input shape. `SourceRegistry::from_document`
validates the complete candidate before constructing state. `SourceRecord`
separates stable ID, display name, abbreviation class, evidence, and source
origin. Display names never act as selectors. Use `ProjectAssigned` for a local
abbreviation; `Conventional` requires nonempty evidence. `OwnedDraft` describes
an owned draft source and does not make its proposals upstream captures.

Selectors are exact and case-sensitive: source IDs, current abbreviations, and
explicit aliases. Names, case-folded variants, and punctuation-normalized forms
do not resolve. Active aliases cannot collide across sources or shadow another
source ID. Retired aliases remain in history and may be reused by a different
source. `resolve_source(..., ActiveOnly)` excludes retired aliases;
`IncludeRetired` includes them and returns `AmbiguousAlias` when reuse makes the
old spelling ambiguous.

A source rename changes its display name freely. Changing its abbreviation
requires retaining the prior spelling as an alias. `apply_document` accepts a
complete replacement only after validation and history checks; IDs, prior alias
rows, lineage rows, snapshot bindings, and occurrence bindings cannot be removed
or rewritten. Retired aliases cannot be reactivated. Failure leaves the
serialized registry unchanged. This is an in-memory update contract, not a
cryptographic audit log.

## Capture attribution

`SnapshotAttribution` binds one exact `SnapshotIdentity` to an asserting source
and optional distributor. `OccurrenceAttribution` optionally adds a contributor
and source-native entry/assertion IDs for one exact `RecordIdentity`. `None`
means unknown; `Some("")` preserves a supplied empty native ID. If an occurrence
has no attribution row, its optional occurrence metadata remains unknown.

`LineageTarget::Known` must reference a registered source and carry evidence.
`Unresolved` stores a locator or description and never resolves by matching its
text to a name. Lineage is descriptive; it does not merge identities. Self-edges
are invalid.

Call `validate_lexicon` to verify both directions: every captured snapshot has
one registry binding and each registered snapshot or occurrence exists in the
supplied `Lexicon`. `attribute` returns one borrowed view for a record and
requires an explicit binding for that record's snapshot. Its record-presence
check is guaranteed when the caller obtains the record from that `Lexicon` or
has validated the registry against it. `attributed_records` validates the full
relationship and walks the original record order, retaining duplicate
observations and alternatives.

The legacy `SourceMetadata::new` API remains infallible and continues to retain
empty values. A lexicon imported with empty metadata is still valid legacy data;
it needs an explicit registry binding before attributed views can be produced.

## Serialization and validation

`RegistryDocument::default()` uses registry version 1. `to_json_bytes` emits a
deterministic representation for its insertion-ordered rows.
`from_json_bytes` rejects malformed UTF-8/JSON and duplicate JSON object keys,
then validates IDs, roles, aliases, lineage and references. Serde deserialization
of `SourceRegistry` also validates its invariants; draft `RegistryDocument` is
not itself validated until passed to `from_document` or `apply_document`.
Unknown fields on registry objects are rejected rather than silently discarded.
This check applies only to registry JSON: original source JSON fields, including
unknown nested fields and their raw bytes, remain in the lexicon's capture.

Validation reports a `RegistryError` rather than mutating partial state. Common
failures include invalid IDs or values, unsupported registry versions, duplicate
entities/bindings/aliases, missing referenced entities, active alias collisions,
ambiguous retired alias resolution, dangling occurrence metadata, and protected
history changes.

## Runnable example

The shipped example builds five `farvi` occurrences: four from source A across
two distributors and one competing definition from source B. The repeated A/D1
rows retain distinct locators even though their text is identical.

```sh
cargo +1.98.0 run --locked -p vlajmaji-lexicon --example source_registry
```

The small example uses synthetic labels only. It does not assert real source
lineage or conventional abbreviations. Project persistence and grouped query
behavior belong to later slices; this API has no SQLite schema or pagination
contract.
