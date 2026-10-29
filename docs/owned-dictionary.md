# Owned dictionary journals

An owned journal is an explicit, local history for a dictionary a person or
project maintains. It stores complete typed dictionary states in an append-only
SQLite journal. It does not import a source, change catalogue selection, or use
the global default store. Keep the source JSON inputs that matter to your own
workflow; the journal records canonical model states and their revision history.

## CLI workflow

Create a seed dictionary with one `OwnedDraft` source, registered editor,
collection, component, release, and content notice. Then create a new journal
under a parent directory that already exists:

```sh
vlajmaji owned init ./data/my-dictionary --input seed.json
vlajmaji owned head ./data/my-dictionary
```

To append a complete next dictionary, name the current expected revision. The
CLI accepts either a file or standard input; it parses and validates the full
typed dictionary before opening the journal for a write:

```sh
vlajmaji owned edit ./data/my-dictionary \
  --expected-head owned-r0 --input revision-1.json
cat revision-2.json | vlajmaji owned edit ./data/my-dictionary \
  --expected-head owned-r1
```

Inspect the compact committed revision list or reconstruct a selected complete
state:

```sh
vlajmaji owned history ./data/my-dictionary
vlajmaji owned show ./data/my-dictionary
vlajmaji owned show ./data/my-dictionary --revision owned-r0
```

Each edit appends exactly one revision. Existing typed records, ordering,
presence values, raw capture bytes, and release selection are immutable. A new
revision names its complete active assertion set; assertions removed from the
prior active set appear as tombstones. If another writer has advanced the head,
the expected-head check returns a conflict and leaves the candidate unapplied.
Inspect `head` and `history` before preparing an explicit retry.

A `OwnedJournalReader` pins a read-only SQLite snapshot. In DELETE-journal mode,
that reader can make a concurrent writer's commit return `Busy`. Drop the
reader, inspect the current head, and retry explicitly if the candidate is
still appropriate; the library does not retry or change the expected head for
you.

`head` and `history` validate journal headers and revision summaries without
reconstructing every body. `show` is the explicit full-state read: it rebuilds
the chosen state, validates the dictionary, and checks its canonical state
digest. Opening a journal never silently repairs it or substitutes the newest
revision for a missing requested one.

The journal root is always explicit. These commands do not consult HOME or XDG
configuration, create a default store, or download data. A report-write error
can occur after an edit has committed; inspect the requested head before
retrying, since a failed terminal write does not roll back a committed edit.

## Reviewed curation and explicit context

The separate curation sidecar is opt-in. Migrate a schema-1 journal at its
current head before adding a complete curation document. Context inputs are
strict files containing both a declared pin ID and the actual pinned
Dictionary. `curation-show` reads the retained document; it does not resolve
targets. `materialize` computes a separate normalized view, and writes its
Dictionary bytes to a new output file only after semantic validation succeeds.

The shipped Tiny example creates a genesis, one ordinary revision, a curation
revision with a three-member Sense and dependent Link, and the exact context
input. It demonstrates the visible missing-context result before supplying the
context:

```sh
cargo run -p vlajmaji-store --example owned_curation -- \
  /tmp/owned-curation-inputs /tmp/owned-curation-journal \
  /tmp/owned-curation-normalized.json
```

The generated files are `seed.json`, `r1.json`, `curation.json`, and
`context.json`. Use a different fresh root to exercise the CLI lifecycle:

```sh
CLI_CURATED_ROOT=/tmp/owned-curation-cli
vlajmaji owned init "$CLI_CURATED_ROOT" --input /tmp/owned-curation-inputs/seed.json
vlajmaji owned edit "$CLI_CURATED_ROOT" --expected-head curation-example-r0 \
  --input /tmp/owned-curation-inputs/r1.json
vlajmaji owned migrate-curation "$CLI_CURATED_ROOT" --expected-head curation-example-r1
vlajmaji owned curate "$CLI_CURATED_ROOT" --expected-head curation-example-r1 \
  --input /tmp/owned-curation-inputs/curation.json
vlajmaji owned curation-show "$CLI_CURATED_ROOT"
vlajmaji owned materialize "$CLI_CURATED_ROOT" \
  --output /tmp/owned-curation-cli-unavailable.json
vlajmaji owned materialize "$CLI_CURATED_ROOT" \
  --context /tmp/owned-curation-inputs/context.json \
  --output /tmp/owned-curation-cli-resolved.json
```

The first materialization reports `Unavailable` with `MissingContext`; the
second resolves the retained records. The normalized file is an explicit view,
not a sealed snapshot. To query it through the ordinary dictionary store,
build another fresh root and query the spelling present in the generated Tiny
input:

```sh
vlajmaji dictionary-build /tmp/owned-curation-cli-resolved.json \
  /tmp/owned-curation-derived-store
vlajmaji dictionary-query /tmp/owned-curation-derived-store \
  --format compact --request '{"operation":"exact","spelling":"coi"}'
```

The compact hit's complete `detail_ref` can be passed unchanged to
`dictionary-detail` for the same derived store. Context pin IDs, native
attribution, occurrence identity, and expected hit IDs should come from the
typed input before querying; a query result is not a source of identity.

## Rust API

`vlajmaji_store::OwnedJournal::create(root, &genesis)` validates the owned
genesis before creating a fresh root. `OwnedJournal::open(root)` opens an
existing journal for writes, and `commit(&expected_head, &candidate)` appends
one complete state with persisted-head compare-and-swap. `OwnedJournalReader`
provides read-only `head()`, `history()`, `show_head()`, and `show(revision)`
operations. The public reports and `OwnedJournalError` preserve typed revisions,
state digests, conflict details, filesystem/backend sources, and stable error
categories. See the crate API and the `owned_journal` example for a small
complete seed/edit lifecycle.

To generate fresh, self-contained sample inputs and exercise the shipped Rust
example:

```sh
cargo run -p vlajmaji-store --example owned_journal -- \
  /tmp/owned-example-inputs /tmp/owned-example-journal
```

The example writes `seed.json` and `edit.json` under the input directory and
prints create/edit reports, head, history, current show, and historical show.
The CLI can consume those same files in a different fresh root:

```sh
CLI_ROOT=/tmp/owned-cli-journal
vlajmaji owned init "$CLI_ROOT" --input /tmp/owned-example-inputs/seed.json
vlajmaji owned edit "$CLI_ROOT" --expected-head owned-r0 \
  --input /tmp/owned-example-inputs/edit.json
vlajmaji owned history "$CLI_ROOT"
vlajmaji owned show "$CLI_ROOT" --revision owned-r0
```

The history contains sequences zero and one, and the explicit final command
reconstructs the original `owned-r0` state. Choose new output directories for
each run; both the example and CLI create their roots exclusively.
