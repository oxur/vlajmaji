# jbotci export import

`vlajmaji-lexicon` exposes `import_jbotci_export(raw, metadata, options)`. It is
a pure, bounded normalizer for the pinned Lensisku English export distributed
by jbotci. It opens no files and performs no network access. The input slices
remain the authority: both complete byte captures and each ordered top-level
native field lexeme are retained in the returned validated `Dictionary`.
Repeated JSON object keys, nested repeated keys, repeated definition IDs, and
repeated spellings are never collapsed. A duplicate-key row has no ambiguous
typed projection and remains an excluded raw occurrence. Non-object JSON rows
are represented by a `$row` native field and excluded.

## Pins and identity

Default `JbotciImportOptions` pins the local source revision
`3d9fd41d9ed32de26311ee27ec86b6b165985aa1`, raw-export SHA-256
`12e826751c3e50c10dd8df1adafb5f9eb32cdf542b22c4244f284b59d3824dde`, and
metadata SHA-256
`fe101eead51e0be1b885f3ee1b4555136e333e839baa250ddbd7b84c9af5fc98`.
Options can name another exact revision and supply its two expected hashes.
The embedded metadata `sha256` must equal the expected raw digest. Raw capture
identity is namespace plus complete-byte digest; occurrence identity is that
capture plus the array index. Release identity includes distributor revision,
raw digest, and metadata digest. Thus identical bytes attributed to a different
revision have a distinct release, while capture, occurrence, assertion, and
entry identities remain stable. `CaptureMetadata.revision` truthfully records
the selected distributor revision. Entry and assertion IDs include the raw
digest, role, and row index (for example, `jbotci-<sha>-entry-0`), so repeated
native IDs and spellings retain separate row identities.

Input and metadata SHA mismatches are errors. Raw syntax, invalid UTF-8, invalid
options, malformed metadata, unsupported metadata values, registry failures,
and model validation failures are returned as typed errors. Metadata accepts a
bounded assignment format: blank lines and `#` comments, unique bare keys,
JSON-quoted strings, booleans, and nonnegative `u64` integers. The required
keys are `language_tag`, `language_realname`, `source_language_tag`, `format`,
`positive_scores_only`, `filename`, `metadata_url`, `download_url`,
`lensisku_created_at`, `sha256`, `definition_count`, and `entry_count`.
Unknown well-formed keys are retained as diagnostics; this parser does not
claim general TOML compatibility.

## Row projection

Each input row yields one assertion and one raw occurrence, whether eligible or
excluded. Eligibility requires an object, no duplicate key at any nesting
level, a nonempty string `word`, a supported string `word_type`, and a
non-whitespace string `definition`. The 16 supported classes are `bu-letteral`,
`cmavo`, `cmavo-compound`, `cmevla`, `experimental cmavo`, `experimental gismu`,
`fu'ivla`, `gismu`, `lujvo`, `nalvla`, `obsolete cmavo`, `obsolete cmevla`,
`obsolete fu'ivla`, `obsolete zei-lujvo`, `phrase`, and `zei-lujvo`. `wiki`,
unknown classes, invalid critical values, and non-object rows are retained with
exclusions and diagnostics. A low or negative score does not affect eligibility.
Supplier count mismatches are reported; they do not discard rows.

| Export field | Normalized projection |
| --- | --- |
| `word` | Exact entry spelling, source language `jbo`; no script or POS inference |
| `word_type`, `selmaho` | Exact native word class and selma'o presence |
| `definition` | Assertion text in `en`; markup remains missing rather than inferred |
| `definition_id` | Registry native assertion ID; never re-labelled as a sense ID |
| `score` | Raw JSON assertion score; never used to choose a winner |
| `notes`, `jargon` | Note/domain contexts with exact value and native field-index evidence |
| `gloss_keywords` | Ordered gloss contexts from objects' `word` values; each original object is also retained as an extension |
| `place_keywords` | Exact whole-field assertion extension; no argument frames or place parsing |
| `rafsi`, `experimental_rafsi` | Ordered source-reported forms, preserving order, duplicates, empties, and source-column standing |
| `etymology` | Etymology statement only; no inferred components |
| `user` | Source-scoped contributor attribution; no real-name merging |
| Unknown fields | Exact native field plus an assertion extension and diagnostic |

Missing, null, and empty values remain distinct where the model has a typed
home. Unsupported optional shapes stay in the raw capture and receive a
diagnostic without excluding an otherwise eligible row. The adapter does not
derive frames, relations, sense groupings, owned revisions, dates, morphology,
or computed annotations.

The source tree's `THIRD-PARTY-NOTICES.md` says the jbovlaste databases are
placed in the public domain and individual definitions are contributed under
those terms. This remains a supplier claim: content redistribution is marked
held pending verification of source coverage and authorization. The adapter
does not copy or publish corpus data. Its Apache-2.0 code notice is recorded
separately from the content notice.

## Pinned local workflow

The repository ships a strict four-key manifest at
`scripts/jbotci-pinned.env`. It is data, never shell code: the workflow accepts
only `version`, `revision`, `raw_sha256`, and `metadata_sha256` assignments.
The default is an exact source commit and two exact digests; it never follows a
branch or chooses a newer revision automatically. `acquire`, `replay`, `check`,
and `update` are explicit operations. Cache directories are keyed by the full
revision/raw/metadata tuple and use a versioned `complete` marker. Existing,
partial, symlinked, or occupied outputs are preserved and rejected; all
successful output files and stores are create-new.

For a local run, choose fresh paths and an explicit cache root:

```sh
export CARGO_TARGET_DIR=/tmp/vlajmaji-target
make jbotci-acquire JBOTCI_CACHE=/tmp/vlajmaji-jbotci-cache
make jbotci-replay JBOTCI_CACHE=/tmp/vlajmaji-jbotci-cache \
  JBOTCI_RELEASE=/tmp/vlajmaji-release.json \
  JBOTCI_STORE=/tmp/vlajmaji-store
make jbotci-check JBOTCI_CACHE=/tmp/vlajmaji-jbotci-cache \
  JBOTCI_RELEASE=/tmp/vlajmaji-release.json
make jbotci-update JBOTCI_NEW_PINS=/tmp/vlajmaji-new-pins.env \
  JBOTCI_CACHE=/tmp/vlajmaji-new-cache JBOTCI_RELEASE=/tmp/vlajmaji-new-release.json \
  JBOTCI_STORE=/tmp/vlajmaji-new-store
```

Fresh replay and update workflows require caller-chosen release paths. They
require a caller-chosen store path only when `JBOTCI_STORE` is supplied; with
that variable omitted, each workflow remains release-only and does not install
data into the CLI's default store. After a
separate local build, `vlajmaji dictionary-query` and `vlajmaji info` can omit
the store argument and resolve `$XDG_DATA_HOME/vlajmaji/stores/default`, or
`$HOME/.local/share/vlajmaji/stores/default` when XDG data home is unset, empty,
or relative. An explicit command-line store takes precedence over those
environment values. Query and info only open a store that already exists.

`acquire` downloads only the two fixed files at the manifest's commit and runs
the real Rust check before creating the completion marker. Repeating it against
an already complete cache verifies it offline. `replay` rechecks that cache,
creates the normalized release and an exact stdout report sidecar, checks the
release against the pinned bytes, and can optionally build a fresh grouped
dictionary store. `check` is read-only. `update` requires a separate explicit
manifest and fresh outputs; it preserves prior pins and releases. A candidate
with valid structure but supplier count mismatches retains its input files,
release and report, exits nonzero, and receives neither a completion marker nor
the requested store. Correcting or retrying that candidate requires a fresh
cache/output path; the pipeline does not remove or adopt an incomplete attempt.
The `CURL` override accepts one executable path and is intended for local
transport tests. Normal acquisition uses curl. Set `VLAJMAJI_BIN` to one
existing executable to bypass Cargo; otherwise the script requires an explicit
`CARGO_TARGET_DIR` and runs the locked Rust 1.98 CLI offline.

The public `jbotci-import` and `jbotci-check` commands remain useful on local
files:

`vlajmaji jbotci-import RAW METADATA FRESH.json` validates the complete import,
serializes it, and creates the output with create-new semantics. It never
replaces an existing path. A write failure can leave a newly created partial
file; the command reports the error and does not remove caller paths.
`vlajmaji jbotci-check RAW METADATA [--release RELEASE.json]` validates without
writing. `--release` checks canonical validated dictionary equality, and check
fails if supplier definition or selected-entry counts disagree with the
observed population. Both commands accept `--revision`, `--raw-sha256`, and
`--metadata-sha256` overrides. Errors are structured JSON on stderr, with empty
stdout and a nonzero status.

For the pinned input, the public example checks both exact input byte captures,
writes the canonical model bytes to a fresh output path, reloads the release,
and checks the full population. The example does not append a line feed; the
CLI output file does append one. The measured API canonical document is
215,412,497 bytes; the CLI release file is 215,412,498 bytes. These are two
representations of the same validated model, not interchangeable byte
identities. The example uses exclusive create:

```sh
cargo +1.98.0 run --locked --offline -p vlajmaji-lexicon \
  --example import_jbotci -- RAW.json METADATA.toml /tmp/jbotci-release.json
```

The observed pinned export contains 33,526 rows and 278,254 ordered native
fields. It produces 33,516 lexical occurrences, 10 excluded occurrences,
31,248 distinct eligible spellings and 238 contributors. The persisted
projection contains 73,226 extension rows: 39,721 per-gloss-object records,
30,683 whole-field gloss records, 2,821 place-field records, and one raw
metadata-text record. The raw capture is
17,163,482 bytes and metadata is 514 bytes. These are replay observations,
not claims about other revisions or a redistributable data package. The
normalized model and persisted projection preserve each raw field, including
repeated object keys, nested keys, unknown values, and source order. Duplicate
native values are not deduplicated into editorial claims. Keep raw and metadata
files local. The `Held` content-redistribution status, the supplier's public
domain statement, and the Apache-2.0 code notice are distinct: no permission to
redistribute the corpus is inferred from the supplier notice or code license.
