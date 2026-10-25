# Grouped exact dictionary lookup

`vlajmaji-search` exposes grouped lookup over a typed draft read from a persisted
`vlajmaji-lexicon::Dictionary` store. A newly built store receives all explicit
verification checks; opening an existing store checks only its minimum header.
The dictionary JSON remains the canonical source record; SQLite is a rebuildable
candidate index and declared projection, not a substitute for explicit audit.
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
spelling, resolved source set, alias policy, schema, and recorded canonical
digest from the store header. Fast open does not authenticate that digest;
request the `hashes` check or `all` before treating it as verified. The cursor
is a corruption-detection token, not an authentication credential. Malformed
encoding/checksum, a cursor for another query or store state, and a valid cursor
position that is not a result produce distinct typed errors. Page size may
change while continuing a traversal.

## Stable references

`DictionarySearchEngine::resolve` checks a `RelationTarget` against the typed
registry draft and graph. Local `Entry`, `Assertion`, `Form`, and
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
vlajmaji dictionary-build dictionary.json
printf '%s\n' '{"operation":"exact","spelling":"farvi","options":{"page_size":2}}' \
  | vlajmaji dictionary-query
vlajmaji dictionary-query \
  --request '{"operation":"exact","spelling":"farvi"}'
vlajmaji info
vlajmaji verify all
```

When `STORE` is omitted from `dictionary-build`, `dictionary-query`, or `info`,
the CLI uses `$XDG_DATA_HOME/vlajmaji/stores/default`. If `XDG_DATA_HOME` is
unset, empty, or relative, it uses
`$HOME/.local/share/vlajmaji/stores/default`. Both variables must supply an
absolute path; if neither provides a usable base, the command reports a
structured `store_path` error. A positional store overrides this resolution.
The default location is intended for a future dictionary-of-dictionaries; the
initial Lensisku-only store is provisional. No merging or download happens
automatically.

Successful build emits the dictionary state hash, the selected store path, and
capture, occurrence, entry, and assertion counts. Default build creates missing
parent directories after reading the input, with private permissions, and
retains the store's exclusive create behavior. A missing input creates no
parents; invalid dictionary JSON can leave empty parent directories but never a
store leaf. It does not replace an existing file, directory, or symlink. Since
the build response includes the store path as a JSON string, a selected
non-UTF-8 build path returns a structured `store_path` error before filesystem
mutation; queries and info retain native platform path handling.
Query and info only open existing stores; they never create a directory or
store. Query emits the exact/reference response directly. `info` reports grouped
dictionary counts and source records for a grouped store, while preserving the
legacy counts response for an archive store. If both store-format markers are
present, info reports an ambiguity; a corrupt grouped marker is never treated
as a legacy store. The grouped info response omits the local filesystem path.
New-command runtime errors exit nonzero, leave stdout empty, and write one JSON
object shaped as `{"error":{"code":"...","message":"...","sources":[]}}`
to stderr. Existing `build` and `search` commands stay explicit and retain their
legacy archive behavior; explicit legacy `info STORE` retains its previous
response.

`verify CHECK [STORE]` performs an explicit audit on a separate read-only
snapshot. Select one of `schema`, `metadata`, `hashes`, `model`, `fks`, or
`projection`, or request `all` for six fixed-order rows. Selective reports
contain only the requested row. Each row is `passed`, `failed`, or `blocked`
when a necessary prerequisite could not be read. A failed verification exits
nonzero, keeps stdout empty, and writes the complete report with the structured
`verification_failed` error to stderr. Verification does not repair the store.

The source model and its distinctions are described in the
[`dictionary model`](dictionary-model.md); query SQL projection details are in
the [`dictionary schema`](dictionary-schema.md).
