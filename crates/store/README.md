# vlajmaji-store

This checkout is on the 0.3.0 development cycle. The dependency and installation
examples below target the published 0.2.0 release.

Rebuildable indexes for [Vlajmaji](https://github.com/oxur/vlajmaji)'s source-preserving lexical archives.

SQLite and FTS5 provide local persistence and query indexes. The optional `graph` feature enables an Oxigraph/RocksDB backend for research comparisons and requires native C++/Clang build tooling. The lexical archive remains canonical; indexes do not merge competing source assertions.

This crate also exposes `DictionaryStore`, the immutable SQLite persistence
boundary for the versioned `vlajmaji-lexicon::Dictionary` model. It writes a
concrete schema-1 projection to a fresh directory and returns it only after
explicitly checking canonical bytes, schema, foreign keys and every projected
SQL value. Later ordinary `open` checks only a minimally readable header; it
does not repeat that comprehensive audit.
It is separate from the existing rebuildable `SqliteStore` search index.
Its `exact_assertion_ids` helper returns byte-exact lexical candidates in
persisted source/spelling/occurrence order. It does not normalize spelling or
apply catalogue selection; the grouped policy is implemented by
`vlajmaji-search` after the candidate query.

`CollectionCatalogue` is a separate local provisioning record. `register`
retains source/reference and ownership metadata without copying pack payloads;
`install` creates a verified Unified generation and publishes it transactionally.
Catalogue `list`/`inspect` are read-only minimum-format summaries, not payload
integrity checks. Use the explicit verification API for that boundary. The
[collection guide](../../docs/collections.md) describes manifests, local paths,
registration, installation, history, and error reports.

Version-1 catalogues remain readable, but writes require the explicit
`CollectionCatalogue::migrate(root, expected_local_revision)` operation.
Migration fences the old physical table names and advances the catalogue
revision transactionally. `update` replaces selected currently installed packs
at an expected revision; `rebuild` reconstructs the current generation from its
retained sealed inputs and publishes a new generation identity. Neither
operation changes user selection or the global default store. A stale revision,
missing installed pack, or invalid retained input is reported without silently
falling back to an older generation. `remove` changes active membership without
deleting registrations, sealed inputs, or historical generations;
`reinstall` requires the exact retained release digest and complete declaration.
`reconcile` explicitly opens an existing catalogue for SQLite recovery and
reports retained attempts and minimum derived-store observations; it does not
activate, migrate, delete, or claim a full integrity audit.

`OwnedJournal` is a separate authoritative append-only history for a complete
owned dictionary. It requires a fresh explicit root and a validated owned
genesis; each `commit` uses the persisted expected head and preserves every
prior typed vector prefix. `OwnedJournalReader::head` and `history` validate
summary metadata, while `show` explicitly reconstructs and validates the full
requested state. This journal does not change the default store or collection
catalogue. See the [owned dictionary guide](../../docs/owned-dictionary.md)
and the `owned_journal` example.

`OwnedCurationDocument` is the separate, validated sidecar for pinned external
context and reviewed Link/Sense records. After explicit schema migration,
`commit_curated` appends its complete history at the same expected base head;
`OwnedJournalReader::curation_show` returns the retained document without
resolving it. Its pure `materialize` operation consumes the supplied pinned
Dictionary inputs and returns a distinct normalized view with visible
unavailable/unresolved outcomes. See the `owned_curation` example for a
self-generated Tiny input set and the [curation workflow](../../docs/owned-dictionary.md#reviewed-curation-and-explicit-context).

`OwnedJournalReader::snapshot` exports the complete retained history, including
the exact 24-section prefix roster and any migrated curation sidecar.
`OwnedJournal::import_snapshot` validates before creating a fresh root and
recognizes only an exact canonical replay. The `owned_snapshot` example emits
a self-contained schema-two history, curation/context inputs, and sealed
snapshot pair; see the [sealed-history workflow](../../docs/owned-dictionary.md#sealed-complete-history).

```rust,no_run
use vlajmaji_lexicon::Dictionary;
use vlajmaji_store::DictionaryStore;

let dictionary = Dictionary::from_json_bytes(&std::fs::read("dictionary.json")?)?;
let store = DictionaryStore::create("data/dictionary", &dictionary)?;
drop(store);
let reopened = DictionaryStore::open("data/dictionary")?;
let report = vlajmaji_store::DictionaryStoreVerification::verify(
    "data/dictionary", vlajmaji_store::DictionaryStoreVerificationCheck::All,
)?;
assert!(report.ok);
assert_eq!(reopened.dictionary()?.document(), dictionary.document());
# Ok::<(), Box<dyn std::error::Error>>(())
```

The destination leaf must be fresh and its parent must already exist. Open is
read-only, does not migrate or repair, and makes no claim about hashes,
canonical model validity, foreign keys, or full projection. `summary()` reads
persisted counts and source records without loading canonical JSON;
`dictionary()` and `canonical_bytes()` are explicit fallible reads.
`DictionaryStoreVerification::verify(path, check)` performs one selected
read-only audit (`schema`, `metadata`, `hashes`, `model`, `fks`, `projection`,
or `all`). See the
[schema and trust-boundary guide](https://github.com/oxur/vlajmaji/blob/main/docs/dictionary-schema.md) and the
`dictionary_store` example.

`DictionaryReadSnapshot` is the separate read-only provider for an explicit
composed view. Its versioned manifest freezes a generation, logical state
digest, member IDs and paths, each member's expected persisted-state digest,
and an optional selected-release set before opening the provider. `Unified`
uses one store, `Fanout` reads one to four stores independently, and `Attached`
uses one primary SQLite connection with fixed internal names for additional
read-only databases. Attached paths must be UTF-8; a native path that cannot be
represented safely returns a typed unsupported-path error instead of being
lossily rewritten. The attached connections are read-only and pinned for the
snapshot lifetime. SQLite ATTACH does not promise atomic publication across
separate files.

Opening a snapshot checks the member headers and expected recorded state. The
explicit `load_document()` result is an **unverified** logical document: for
fanout and attach, it reads and composes the member documents and checks the
resulting logical digest, but it does not audit every SQL projection row or
prove the databases were published together. Call
`DictionaryStoreVerification::verify(..., All)` when full persisted-store
verification is required. Snapshot generation and selected releases are
runtime view identity, not edits to stored release-selection history.

```toml
[dependencies]
vlajmaji-store = "0.2.0"
```

Requires Rust 1.98 or newer. Store APIs use a store directory, rather than the internal SQLite file. Corpora are not bundled.

See the [API documentation](https://docs.rs/vlajmaji-store). Licensed under Apache-2.0.
