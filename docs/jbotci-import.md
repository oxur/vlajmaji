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

## CLI and local proof

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

For the local pinned files, the public example checks both exact input byte
captures, writes the canonical model bytes to a fresh output path, reloads the
release, and checks the full population. The example uses exclusive create and
does not append a line feed; the CLI's human-readable output file does append
one final line feed, so those file byte counts differ by one:

```sh
cargo +1.98.0 run --locked --offline -p vlajmaji-lexicon \
  --example import_jbotci -- RAW.json METADATA.toml /tmp/jbotci-release.json
```

The observed pinned export contains 33,526 rows and 278,254 ordered native
fields. It produces 33,516 lexical occurrences, 10 excluded occurrences,
31,248 distinct eligible spellings, and 238 contributors. The raw capture is
17,163,482 bytes and metadata is 514 bytes. These are replay observations,
not claims about other revisions or a redistributable data package. Keep raw
and metadata files local; acquisition and packaging are separate work.
