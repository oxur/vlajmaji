# Grouped exact dictionary lookup

`vlajmaji-search` exposes a grouped lookup API over a validated, persisted
`vlajmaji-lexicon::Dictionary`. The dictionary JSON remains the canonical
source record; SQLite is a rebuildable candidate index and checked projection.
The lookup returns source assertions and their provenance without merging
competing definitions or interpreting frame prose.

## Build and query from Rust

```rust,no_run
use vlajmaji_search::{DictionaryExactOptions, DictionarySearchEngine};

let input = std::fs::read("dictionary.json")?;
let built = DictionarySearchEngine::build("data/dictionary", &input)?;
println!("state {}", built.state_sha256());
drop(built);

let engine = DictionarySearchEngine::open("data/dictionary")?;
let result = engine.exact("farvi", DictionaryExactOptions::default())?;
println!("{} selected from {} lexical candidates",
    result.totals.occurrences, result.native_candidate_occurrences);
# Ok::<(), Box<dyn std::error::Error>>(())
```

The query is an exact UTF-8 spelling match. The store uses the persisted
spelling index, then the engine selects assertions in currently selected,
available releases and the explicit current head for owned sources. It does
not normalize spelling, generate forms, or infer semantics. The response
separates SQL candidate count from selected results and includes full totals,
all source availability, and occurrence-level groups. Missing capture payloads
remain catalogue state rather than fabricated hits.

`DictionaryExactOptions` can filter by source ID, current abbreviation, or
eligible alias. Stable IDs take precedence; the alias scope can include retired
names when resolving a selector. Empty source filters select every selectable
source. An available source with no matching spelling still appears in
`resolved_sources`; availability for all registered sources is returned even
when a filter or page is used.

## Pages and cursors

With no page size, `exact` returns every selected occurrence. A page size of
zero returns no groups but retains the real complete totals. Positive page
sizes count occurrences, not source groups. The opaque cursor binds the exact
spelling, resolved source set, alias policy, schema, and canonical store state.
It is a corruption-detection token, not an authentication credential. Malformed
encoding/checksum, a cursor for another query or store state, and a valid cursor
position that is not a result produce distinct typed errors. Page size may
change while continuing a traversal.

## Stable references

`DictionarySearchEngine::resolve` checks a `RelationTarget` against the
validated registry and typed graph. Local `Entry`, `Assertion`, `Form`, and
`EditorialSense` targets return only the corresponding entity type. Catalogue
targets keep their requested source/release/native identities and report
confirmed versus unverified assertion candidates; unavailable or missing
payload state remains visible in availability and reference views. No network
lookup or inferred target substitution occurs.

```rust,no_run
use vlajmaji_lexicon::{EntityId, LocalRef, RelationTarget};
use vlajmaji_search::DictionarySearchEngine;
let input = std::fs::read("dictionary.json")?;
let engine = DictionarySearchEngine::build("data/dictionary", &input)?;
let request = RelationTarget::Local(LocalRef::Assertion(
    EntityId::new("assertion-0")?,
));
let resolution = engine.resolve(&request)?;
println!("{:?}", resolution.status);
# Ok::<(), Box<dyn std::error::Error>>(())
```

For the complete response fields, source-presence distinctions, selection
rules, and error codes, see the public Rust API docs. The
[`dictionary_lookup` example](../crates/search/examples/dictionary_lookup.rs)
builds a fresh store, reopens it, pages results, and resolves a local reference.

## CLI

Build a fresh store from versioned dictionary JSON and query it with JSON on
stdin or via `--request`:

```sh
vlajmaji dictionary-build dictionary.json data/dictionary
printf '%s\n' '{"operation":"exact","spelling":"farvi","options":{"page_size":2}}' \
  | vlajmaji dictionary-query data/dictionary
vlajmaji dictionary-query data/dictionary \
  --request '{"operation":"exact","spelling":"farvi"}'
```

Successful build emits the dictionary state hash and capture, occurrence,
entry, and assertion counts. Query emits the exact/reference response directly.
New-command runtime errors exit nonzero, leave stdout empty, and write one JSON
object shaped as `{"error":{"code":"...","message":"...","sources":[]}}`
to stderr. Existing `build`, `search`, and `info` commands retain their legacy
wire behavior.

The source model and its distinctions are described in the
[`dictionary model`](dictionary-model.md); query SQL projection details are in
the [`dictionary schema`](dictionary-schema.md).
