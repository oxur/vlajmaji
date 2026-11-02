# Integrated dictionary workbench

A workbench is a new, explicitly prepared view over selected installed source
packs and an optional revision from one owned journal. It writes a checked
`binding.json` and a derived `dictionary/` store under a new root. It does not
modify either source, substitute a default store, or replace an existing
workbench. The parent directory must already exist.

`vlajmaji_store::WorkbenchView::build_new` creates the view;
`vlajmaji_search::WorkbenchSearchEngine` retains it for Full and Compact
queries and complete-hit detail. Build input is a strict version-1 JSON
selection. `packs` and `disabled` preserve caller order. `owned` is `null`,
`"head"`, or `{"revision":"owned-r1"}`. A context binding names a curation
context and an explicit ordered list of selected, enabled packs. One-pack
contexts use that original validated dictionary; multi-pack contexts compose
in their listed order. Missing context bodies remain visible as typed outcomes.

## Generate a complete synthetic input set

From the workspace root, run the self-contained producer into a fresh path:

```sh
cargo run -p vlajmaji-search --example integrated_workbench -- workbench-example
```

The example writes `workbench-example/inputs/` before creating its synthetic
catalogue, owned journal, or workbench. It contains the tiny La-Lojban input,
its normalized dictionary and sealed pair, an empty owned genesis, the r1 edit
with two `farvi` definitions, a valid r2 edit that retains those definitions
and adds a third, the selection, exact request, and input-derived expected
rosters. The upstream source is `lensisku` (“advance”); the separate
`owned-example` source contributes “grow” and “develop” at r1, then “flourish”
at r2. The producer builds an r1 Head view, commits r2, confirms the retained
r1 view is stale, and builds a fresh r2 Head view. These are illustrative
synthetic rows, not a rights grant or a substitute for pinned source packs.

Build the CLI from this checkout. The following commands prepare a separate
CLI-owned journal and two fresh Head-bound workbench roots from the emitted
files:

```sh
cargo build -p vlajmaji-cli --bin vlajmaji
CLI="${CARGO_TARGET_DIR:-target}/debug/vlajmaji"
```

The example's catalogue is reusable, while the CLI recipe uses a separate
journal and fresh view roots. Initialize it from the emitted genesis and apply
both emitted revision files in order. It also emits `historical-selection.json`, which selects
`owned-r1` explicitly before any writes:

```sh
CLI_OWNED=workbench-example/cli-owned
"$CLI" owned init "$CLI_OWNED" --input workbench-example/inputs/owned-seed.json
"$CLI" owned edit "$CLI_OWNED" --expected-head owned-r0 \
  --input workbench-example/inputs/owned-r1.json
CLI_W1=workbench-example/cli-workbench-r1
"$CLI" workbench build "$CLI_W1" \
  --selection workbench-example/inputs/selection.json \
  --catalogue workbench-example/catalogue --owned "$CLI_OWNED"
"$CLI" workbench info "$CLI_W1" > workbench-example/info-r1.json
```

Run the same request through all three renderings of the r1 view:

```sh
"$CLI" workbench query "$CLI_W1" \
  --request workbench-example/inputs/request.json --format full \
  > workbench-example/full.json
"$CLI" workbench query "$CLI_W1" \
  --request workbench-example/inputs/request.json --format compact \
  > workbench-example/compact.json
"$CLI" workbench query "$CLI_W1" \
  --request workbench-example/inputs/request.json --format readable
```

The Full response is the complete existing typed query response. Compact
retains the existing compact hit and its detail key; the readable command uses
the existing compact renderer with the outer binding and small selection
summary. To request complete detail for the first compact hit, take its
`detail_ref` and use the binding actually returned by `info`:

```sh
jq '.result.response.groups[0].hits[0].detail_ref' workbench-example/compact.json \
  > workbench-example/detail-reference.json
BINDING=$(jq -r '.binding.binding_sha256' workbench-example/info-r1.json)
"$CLI" workbench detail "$CLI_W1" \
  --reference workbench-example/detail-reference.json \
  --expected-binding "$BINDING" --format full
"$CLI" workbench detail "$CLI_W1" \
  --reference workbench-example/detail-reference.json \
  --expected-binding "$BINDING" --format readable
```

The emitted `inputs/expected.json` records the input-derived source order and
definitions for the exact `farvi` request at r1 and r2. It is the oracle for
the subsequent commands, rather than a value inferred from their results. The
outer binding digest identifies this particular prepared workbench; Compact's
own snapshot digest has a different purpose and must not be substituted for
it.

## Computed lexical and proposal queries

The current unpublished 0.3 workspace adds six source-backed computed query
families to the same Full and Compact workbench providers: spelling distance,
prosody, nuclei, contrast, components, and composition. A returned assertion
remains the retained source occurrence; its phonological analysis,
decomposition, or link hops are separate derived evidence. `ProposalExact`
constructs identities for caller-supplied proposal rows without adding them to
the dictionary, and `Generate` reports a morphology-engine value without
claiming that value is attested. Components and Generate require the optional
`morphology` feature.

Run the integrated producer with morphology enabled to exercise all eight
operations:

```sh
cargo +1.98.0 run --locked -p vlajmaji-search --features morphology \
  --example integrated_workbench -- workbench-example
```

The producer keeps the earlier `inputs/`, `artifacts/lexical-inputs`, and
`artifacts/links-inputs` authorities and adds exactly twelve files under
`artifacts/computed-inputs`. Six are typed source/journal/selection inputs;
the other six are fixed query, expected-result, fixture, and cursor-protocol
authorities. It writes these files before creating
`computed-workbench/owned-journal`, installing the mixed synthetic catalogue,
or building a view. The `computed` profile is configured at revision one,
refreshed against r1 at revision two, made default at revision three, then
refreshed against r2 at revision four. The prepared profile sequence is zero
at r1 and one at r2; the journal's own r0/r1/r2 revision identities are a
separate sequence.

The fixed model deliberately includes source occurrences with `Missing` and
`Null` native fields, excluded rows, an editorial Note with a retained source
span, and an owned r2 tombstone. Those states remain typed input evidence; the
queries do not rewrite them to make an analysis succeed. The expected file is
the pre-input identity, ordering, and count authority, so compare complete
returned records and totals to it rather than deriving expectations from one
query's output.

### Prepare the seven derived requests

The producer emits the six authorities under `artifacts/computed-inputs`.
These seven additional requests are guide inputs derived from those fixed
authorities; they are not extra producer outputs. Each keeps the exact
`ActiveOnly` alias value and null page/cursor controls:

| File | Exact request JSON |
| --- | --- |
| `computed-prosody.json` | `{"version":1,"query":{"operation":"prosody","kind":"brivla","syllables":2,"nuclei":2,"primary_nucleus":0},"options":{"sources":[],"alias_scope":"ActiveOnly","page_size":null,"cursor":null}}` |
| `computed-nuclei.json` | `{"version":1,"query":{"operation":"nuclei","kind":"brivla","pattern":["a","i"]},"options":{"sources":[],"alias_scope":"ActiveOnly","page_size":null,"cursor":null}}` |
| `computed-contrast.json` | `{"version":1,"query":{"operation":"contrast","anchor":"farvi","kind":"brivla","require_mismatch":true},"options":{"sources":[],"alias_scope":"ActiveOnly","page_size":null,"cursor":null}}` |
| `computed-components.json` | `{"version":1,"query":{"operation":"components","components":["mlatu"]},"options":{"sources":[],"alias_scope":"ActiveOnly","page_size":null,"cursor":null}}` |
| `computed-compose.json` | `{"version":1,"query":{"operation":"compose","definition_contains":"plum","links_from":"farvi","min_nuclei":2,"kind":"brivla"},"options":{"sources":[],"alias_scope":"ActiveOnly","page_size":null,"cursor":null}}` |
| `computed-generate-allow-y.json` | `{"version":1,"query":{"operation":"generate","components":["mlatu","kerlo"],"setting":"allow_y"},"options":{"sources":[],"alias_scope":"ActiveOnly","page_size":null,"cursor":null}}` |
| `computed-generate-force-y.json` | `{"version":1,"query":{"operation":"generate","components":["mlatu","kerlo"],"setting":"force_y"},"options":{"sources":[],"alias_scope":"ActiveOnly","page_size":null,"cursor":null}}` |

For repeatable preparation, save the following as `prepare-computed-requests.py`
and run it from the output parent after the producer. It reads the fixed
expected/request files and writes only those seven derived requests:

```python
import json
from pathlib import Path

root = Path("workbench-example")
inputs = root / "artifacts/computed-inputs"
expected = json.loads((inputs / "computed-expected.json").read_text())
source = json.loads((inputs / "computed-source-request.json").read_text())
generate = json.loads((inputs / "computed-generate-request.json").read_text())
queries = {
    "computed-prosody.json": expected["r1"]["prosody"]["query"],
    "computed-nuclei.json": expected["r1"]["nuclei"]["query"],
    "computed-contrast.json": expected["r1"]["contrast"]["query"],
    "computed-components.json": expected["r1"]["components"]["query"],
    "computed-compose.json": expected["r1"]["compose"]["query"],
    "computed-generate-allow-y.json": {**generate["query"], "setting": "allow_y"},
    "computed-generate-force-y.json": {**generate["query"], "setting": "force_y"},
}
for name, query in queries.items():
    path = root / name
    if path.exists():
        raise FileExistsError(path)
    request = {"version": 1, "query": query, "options": source["options"]}
    path.write_text(json.dumps(request, ensure_ascii=False, separators=(",", ":")) + "\n")
```

### Run the twelve CLI queries

These are the twelve query children used with the emitted example workspace.
Run them from the directory containing `workbench-example`; `H` is the
explicit historical r1 root, while the two profile calls use the current r2
publication. The Full named/default comparison is about the result body; their
effective binding identities are allowed to differ.

```sh
SOURCE=/path/to/vlajmaji
TARGET_DIR="${CARGO_TARGET_DIR:-$SOURCE/target}"
case "$TARGET_DIR" in
  /*) ;;
  *) TARGET_DIR="$SOURCE/$TARGET_DIR" ;;
esac
CLI="$TARGET_DIR/debug/vlajmaji"
W=workbench-example
C="$W/artifacts/computed-inputs"
H="$W/computed-workbench/view-historical-r1"
CFG="$W/computed-workbench/profiles.json"

"$CLI" workbench lexical "$H" --request "$C/computed-source-request.json" --format full > "$W/computed-guide-01.json"
"$CLI" workbench lexical "$H" --request "$W/computed-prosody.json" --format compact > "$W/computed-guide-02.json"
"$CLI" workbench lexical "$H" --request "$W/computed-nuclei.json" --format readable > "$W/computed-guide-03.txt"
"$CLI" workbench lexical "$H" --request "$W/computed-contrast.json" --format full > "$W/computed-guide-04.json"
"$CLI" workbench lexical "$H" --request "$W/computed-components.json" --format compact > "$W/computed-guide-05.json"
"$CLI" workbench lexical "$H" --request "$W/computed-compose.json" --format compact > "$W/computed-guide-06.json"
"$CLI" workbench lexical "$H" --request "$C/computed-proposal-request.json" --format readable > "$W/computed-guide-07.txt"
"$CLI" workbench lexical "$H" --request "$C/computed-generate-request.json" --format full > "$W/computed-guide-08.json"
"$CLI" workbench lexical "$H" --request "$W/computed-generate-allow-y.json" --format full > "$W/computed-guide-09.json"
"$CLI" workbench lexical "$H" --request "$W/computed-generate-force-y.json" --format readable > "$W/computed-guide-10.txt"
"$CLI" workbench profiles lexical --config "$CFG" --profile computed --request "$W/computed-prosody.json" --format full > "$W/computed-guide-11.json"
"$CLI" workbench profiles lexical --config "$CFG" --request "$W/computed-prosody.json" --format full > "$W/computed-guide-12.json"
```

The command sequence was exercised before this guide section was added; the
documentation records that tested recipe and its output-parent-relative path
convention. Stderr capture and full per-command evidence are retained in the
slice execution record.

The r1 view is Head-bound. Append the already-emitted r2 candidate, confirm the
old view rejects a query as stale, and build both a fresh Head view and a fresh
historical r1 view. The example checks that the historical r1 Full result stays
equal to the original r1 result after the append:

```sh
"$CLI" owned edit "$CLI_OWNED" --expected-head owned-r1 \
  --input workbench-example/inputs/owned-r2.json
if "$CLI" workbench query "$CLI_W1" \
    --request workbench-example/inputs/request.json --format full \
    > workbench-example/stale-r1.stdout.json \
    2> workbench-example/stale-r1.stderr.json; then
  echo "expected the retained r1 Head view to be stale" >&2
  exit 1
fi
jq -e '.error.code == "stale_workbench"' workbench-example/stale-r1.stderr.json
CLI_HISTORICAL_R1=workbench-example/cli-workbench-historical-r1
"$CLI" workbench build "$CLI_HISTORICAL_R1" \
  --selection workbench-example/inputs/historical-selection.json \
  --catalogue workbench-example/catalogue --owned "$CLI_OWNED"
"$CLI" workbench query "$CLI_HISTORICAL_R1" \
  --request workbench-example/inputs/request.json --format full \
  > workbench-example/full-historical-r1.json
CLI_W2=workbench-example/cli-workbench-r2
"$CLI" workbench build "$CLI_W2" \
  --selection workbench-example/inputs/selection.json \
  --catalogue workbench-example/catalogue --owned "$CLI_OWNED"
"$CLI" workbench query "$CLI_W2" \
  --request workbench-example/inputs/request.json --format full \
  > workbench-example/full-r2.json
```

The dedicated reference commands consume the typed targets emitted before the
example's writes. The fresh r2 root is current for both upstream catalogue and
owned references; the earlier `CLI_W1` remains a stale Head view. Full returns
the complete reference response, Compact retains its own detail key, and
Readable uses the existing compact renderer:

```sh
REFERENCE_INPUTS=workbench-example/artifacts/reference-inputs
"$CLI" workbench reference "$CLI_W2" \
  --target "$REFERENCE_INPUTS/reference-upstream.json" --format full \
  > workbench-example/reference-upstream-full.json
"$CLI" workbench reference "$CLI_W2" \
  --target "$REFERENCE_INPUTS/reference-owned-assertion.json" --format compact \
  > workbench-example/reference-owned-compact.json
"$CLI" workbench reference "$CLI_W2" \
  --target "$REFERENCE_INPUTS/reference-unknown.json" --format readable
jq -e '.kind == "vlajmaji-workbench-reference" and
       .result.status == "Resolved" and
       (.result.payload.confirmed | length) == 1' \
  workbench-example/reference-upstream-full.json
jq -e '.format == "compact" and .result.status == "Resolved" and
       (.result.payload.confirmed[0].detail_ref != null)' \
  workbench-example/reference-owned-compact.json
```

The expected r1 result has source order `lensisku`, `owned-example`, with one
upstream hit and two owned hits (three total). The explicit historical r1 view
retains that same full result after r2 is appended, while the former r1 Head
view is stale. The expected r2 result keeps
that order and adds the owned “flourish” hit (four total). The stale r1 query
must report the typed stale-dependency error; it does not mutate either view.

## Freshness and inspection boundaries

A `Head`-bound workbench becomes stale after a later owned commit; an
explicitly selected historical revision remains usable while retained. A
schema-one journal has no curation identity. Explicit migration to schema two
changes that identity and requires a newly built workbench. Build reads source
bytes, while ordinary open and queries compare the recorded catalogue and
selected owned identities without re-reading every source release or
reconstructing complete history.

The binding retains every declared context pin, including omitted bodies, and
the supplied/omitted observation. The per-query selection summary is
deliberately smaller; full pin, release, resolution, and owned identity details
remain available through `info` and the library binding accessor. Composition
errors remain errors even when individual records also have unavailable
context outcomes. A failure after successful persistence does not roll back
the workbench, and an incomplete new candidate is left in place for inspection.

## Configure and refresh named profiles

These APIs are new in the unpublished workspace 0.3.0; the 0.2.0 dependency
snippets above do not expose them. The producer emits three profile inputs
before creating any catalogue or journal: `profile-alpha-selection.json` is an
exact copy of the mixed selection, `profile-beta-selection.json` selects only
the owned Head, and `profiles-expected.json` declares revisions 1–6,
publication sequences 0–2, and input-derived source-group and occurrence
counts.

Use a fresh CLI-owned journal for this sequence. The earlier `CLI_OWNED` from
the unprofiled recipe has already advanced to r2. Initialize this new journal
from the emitted seed and apply r1 first:

```sh
CLI_CONFIGURED_OWNED=workbench-example/cli-configured-owned
"$CLI" owned init "$CLI_CONFIGURED_OWNED" --input workbench-example/inputs/owned-seed.json
"$CLI" owned edit "$CLI_CONFIGURED_OWNED" --expected-head owned-r0 \
  --input workbench-example/inputs/owned-r1.json
PROFILE_CONFIG=workbench-example/cli-workbench-profiles.json
CLI_ALPHA_R1=workbench-example/cli-configured-alpha-r1
"$CLI" workbench profiles configure --config "$PROFILE_CONFIG" --name alpha \
  --selection workbench-example/inputs/profile-alpha-selection.json \
  --catalogue workbench-example/catalogue --owned "$CLI_CONFIGURED_OWNED" --make-default \
  > workbench-example/profile-alpha-configure.json
jq -e '.revision == 1 and .changed == true and .active == null' \
  workbench-example/profile-alpha-configure.json
"$CLI" workbench profiles refresh "$CLI_ALPHA_R1" --config "$PROFILE_CONFIG" \
  --expected-revision 1 > workbench-example/profile-alpha-refresh.json
jq -e '.publication.revision == 2 and .publication.active.sequence == 0' \
  workbench-example/profile-alpha-refresh.json
"$CLI" workbench profiles query --config "$PROFILE_CONFIG" --profile alpha \
  --request workbench-example/inputs/request.json --format full \
  > workbench-example/profile-alpha-full.json
"$CLI" workbench profiles query --config "$PROFILE_CONFIG" --profile alpha \
  --request workbench-example/inputs/request.json --format compact \
  > workbench-example/profile-alpha-compact.json
"$CLI" workbench profiles query --config "$PROFILE_CONFIG" --profile alpha \
  --request workbench-example/inputs/request.json --format readable
jq --slurpfile expected workbench-example/inputs/profiles-expected.json -e \
  '.result.response.totals.occurrences == $expected[0].alpha_r1.occurrences and
   (.result.response.groups | length) == $expected[0].alpha_r1.source_groups' \
  workbench-example/profile-alpha-full.json
```

While alpha's prepared Head is current, resolve the same emitted upstream
target through the configured route. The outer binding is the effective
profile identity; it is distinct from the direct workbench binding and Compact
snapshot key:

```sh
"$CLI" workbench profiles reference --config "$PROFILE_CONFIG" --profile alpha \
  --target workbench-example/artifacts/reference-inputs/reference-upstream.json \
  --format compact > workbench-example/profile-alpha-reference-compact.json
"$CLI" workbench profiles reference --config "$PROFILE_CONFIG" --profile alpha \
  --target workbench-example/artifacts/reference-inputs/reference-upstream.json \
  --format readable
jq -e '.format == "compact" and .result.status == "Resolved" and
       (.result.payload.confirmed | length) == 1' \
  workbench-example/profile-alpha-reference-compact.json
```

The compact detail reference and mandatory outer binding key come from the
same configured response. The inner compact snapshot token remains a separate
identity:

```sh
jq '.result.response.groups[0].hits[0].detail_ref' \
  workbench-example/profile-alpha-compact.json \
  > workbench-example/profile-alpha-detail-reference.json
PROFILE_BINDING=$(jq -r '.binding_sha256' workbench-example/profile-alpha-compact.json)
"$CLI" workbench profiles detail --config "$PROFILE_CONFIG" --profile alpha \
  --reference workbench-example/profile-alpha-detail-reference.json \
  --expected-binding "$PROFILE_BINDING" --format full \
  > workbench-example/profile-alpha-detail.json
jq -e '.result.hit != null' workbench-example/profile-alpha-detail.json
```

The exact repeated beta configuration is a semantic no-op: revision and config
bytes must remain unchanged. Reopening alpha by name returns the same result
body as the earlier default-mode query, with a different effective identity:

```sh
CLI_BETA_R1=workbench-example/cli-configured-beta-r1
CLI_BETA_R2=workbench-example/cli-configured-beta-r2
"$CLI" workbench profiles configure --config "$PROFILE_CONFIG" --name beta \
  --selection workbench-example/inputs/profile-beta-selection.json \
  --owned "$CLI_CONFIGURED_OWNED" --expected-revision 2 \
  > workbench-example/profile-beta-configure.json
jq -e '.revision == 3 and .changed == true and .default_profile == "alpha"' \
  workbench-example/profile-beta-configure.json
cp "$PROFILE_CONFIG" workbench-example/profile-config-before-noop.json
shasum -a 256 "$PROFILE_CONFIG" > workbench-example/profile-config-before-noop.sha256
"$CLI" workbench profiles configure --config "$PROFILE_CONFIG" --name beta \
  --selection workbench-example/inputs/profile-beta-selection.json \
  --owned "$CLI_CONFIGURED_OWNED" --expected-revision 3 \
  > workbench-example/profile-beta-noop.json
jq -e '.revision == 3 and .changed == false' workbench-example/profile-beta-noop.json
cmp "$PROFILE_CONFIG" workbench-example/profile-config-before-noop.json
shasum -a 256 -c workbench-example/profile-config-before-noop.sha256
"$CLI" workbench profiles query --config "$PROFILE_CONFIG" --profile alpha \
  --request workbench-example/inputs/request.json --format full \
  > workbench-example/profile-alpha-named-full.json
jq --slurpfile prior workbench-example/profile-alpha-full.json -e \
  '.result == $prior[0].result and .binding_sha256 != $prior[0].binding_sha256' \
  workbench-example/profile-alpha-named-full.json
```

Refresh beta's r1 view, then make beta the default before advancing its owned
Head. A default-mode query must return the same two-hit body as the named beta
query. Appending r2 makes that retained preparation stale; refreshing in
default mode then publishes sequence two. Named and default beta queries after
refresh have equal result bodies and distinct effective identities:

```sh
"$CLI" workbench profiles refresh "$CLI_BETA_R1" --config "$PROFILE_CONFIG" \
  --profile beta --expected-revision 3 > workbench-example/profile-beta-r1.json
"$CLI" workbench profiles query --config "$PROFILE_CONFIG" --profile beta \
  --request workbench-example/inputs/request.json --format full \
  > workbench-example/profile-beta-full-r1.json
jq --slurpfile expected workbench-example/inputs/profiles-expected.json -e \
  '.result.response.totals.occurrences == $expected[0].beta_r1.occurrences and
   (.result.response.groups | length) == $expected[0].beta_r1.source_groups' \
  workbench-example/profile-beta-full-r1.json
"$CLI" workbench profiles reference --config "$PROFILE_CONFIG" --profile beta \
  --target workbench-example/artifacts/reference-inputs/reference-owned-entry.json \
  --format full > workbench-example/profile-beta-owned-reference.json
jq -e '.format == "full" and .result.status == "Resolved" and
       .result.catalogue_matches == [] and .result.registrations == []' \
  workbench-example/profile-beta-owned-reference.json
"$CLI" workbench profiles default --config "$PROFILE_CONFIG" --profile beta \
  --expected-revision 4 > workbench-example/profile-default-beta.json
"$CLI" workbench profiles query --config "$PROFILE_CONFIG" \
  --request workbench-example/inputs/request.json --format full \
  > workbench-example/profile-beta-default-before-r2.json
jq --slurpfile named workbench-example/profile-beta-full-r1.json -e \
  '.result == $named[0].result and .configuration.requested_profile == null' \
  workbench-example/profile-beta-default-before-r2.json
"$CLI" owned edit "$CLI_CONFIGURED_OWNED" --expected-head owned-r1 \
  --input workbench-example/inputs/owned-r2.json
if "$CLI" workbench profiles query --config "$PROFILE_CONFIG" \
    --request workbench-example/inputs/request.json --format full \
    > workbench-example/profile-beta-stale.stdout.json \
    2> workbench-example/profile-beta-stale.stderr.json; then
  echo "expected the retained beta Head preparation to be stale" >&2
  exit 1
fi
jq -e '.error.code == "stale_workbench"' workbench-example/profile-beta-stale.stderr.json
"$CLI" workbench profiles refresh "$CLI_BETA_R2" --config "$PROFILE_CONFIG" \
  --expected-revision 5 > workbench-example/profile-beta-r2.json
jq -e '.publication.revision == 6 and .publication.active.sequence == 2' \
  workbench-example/profile-beta-r2.json
"$CLI" workbench profiles query --config "$PROFILE_CONFIG" --profile beta \
  --request workbench-example/inputs/request.json --format full \
  > workbench-example/profile-beta-full-r2.json
jq --slurpfile expected workbench-example/inputs/profiles-expected.json -e \
  '.result.response.totals.occurrences == $expected[0].beta_r2.occurrences and
   (.result.response.groups | length) == $expected[0].beta_r2.source_groups' \
  workbench-example/profile-beta-full-r2.json
"$CLI" workbench profiles query --config "$PROFILE_CONFIG" \
  --request workbench-example/inputs/request.json --format full \
  > workbench-example/profile-beta-default-r2.json
jq --slurpfile named workbench-example/profile-beta-full-r2.json -e \
  '.result == $named[0].result and .binding_sha256 != $named[0].binding_sha256' \
  workbench-example/profile-beta-default-r2.json
"$CLI" workbench profiles info --config "$PROFILE_CONFIG" \
  > workbench-example/profile-info.json
jq -e '.default_profile == "beta" and .revision == 6' workbench-example/profile-info.json
"$CLI" workbench profiles history --config "$PROFILE_CONFIG" \
  > workbench-example/profile-history.json
jq -e '.revision == 6 and (.prepared | map(.sequence)) == [0,1,2]' \
  workbench-example/profile-history.json
```

The report in `profiles-expected.json` is the input-defined count oracle for
these commands; it is not inferred from query responses. Config revision 3
changes the effective identity captured by an earlier alpha engine, while the
immutable alpha root and its history row remain available for inspection.

## Literal reverse-definition and stored-Form queries

The integrated example adds an independent lexical workspace under
`workbench-example/lexical-workbench`. Its ten typed inputs and fixed protocol
oracles are written first to `workbench-example/artifacts/lexical-inputs/`;
the earlier `inputs/` and `artifacts/reference-inputs/` files remain unchanged.
The stage retains an upstream source, an owned r0→r1→r2 journal, and configured
r1/r2 preparations. At r1, reverse lookup for `gRoW` returns owned assertion
a and the two upstream literal matches a and b; owned assertion b has definition
`develop` and is not a match. The seven `farvi-form` rows include four upstream
Forms and three owned Forms, including two with no eligible assertions. At r2
the owned assertion pair is tombstoned and only assertion c remains on the owned
stem; the upstream rows and their evidence remain intact. The `view-r1` direct
root is Head-bound and becomes stale after the append. The separate
`view-historical-r1` root explicitly retains r1 and remains usable.

The retained historical r1 view also supports the literal reverse-definition
request. Full and Compact preserve the pre-input assertion order:
`lexical-owned-a`, `lexical-upstream-a`, then `lexical-upstream-b`.

```sh
CLI="${CARGO_TARGET_DIR:-target}/debug/vlajmaji"
LEXICAL_INPUTS=workbench-example/artifacts/lexical-inputs
LEXICAL_VIEW=workbench-example/lexical-workbench/view-historical-r1
"$CLI" workbench lexical "$LEXICAL_VIEW" \
  --request "$LEXICAL_INPUTS/reverse-grow.json" --format full \
  > workbench-example/lexical-reverse-full.json
"$CLI" workbench lexical "$LEXICAL_VIEW" \
  --request "$LEXICAL_INPUTS/reverse-grow.json" --format compact \
  > workbench-example/lexical-reverse-compact.json
jq -e '.result.response.totals.records == 3 and
       ([.result.response.groups[].hits[].assertion.id] | flatten) ==
       ["lexical-owned-a", "lexical-upstream-a", "lexical-upstream-b"]' \
  workbench-example/lexical-reverse-full.json
jq -e '.result.response.totals.records == 3 and
       ([.result.response.groups[].hits[].assertion.id] | flatten) ==
       ["lexical-owned-a", "lexical-upstream-a", "lexical-upstream-b"]' \
  workbench-example/lexical-reverse-compact.json
```

Run a Full direct query and then a Compact query from the same emitted request:

```sh
LEXICAL_INPUTS=workbench-example/artifacts/lexical-inputs
LEXICAL_VIEW=workbench-example/lexical-workbench/view-historical-r1
LEXICAL_CONFIG=workbench-example/lexical-workbench/workbench-profiles.json
"$CLI" workbench lexical "$LEXICAL_VIEW" \
  --request "$LEXICAL_INPUTS/form-all.json" --format full \
  > workbench-example/lexical-direct-full.json
"$CLI" workbench lexical "$LEXICAL_VIEW" \
  --request "$LEXICAL_INPUTS/form-all.json" --format compact \
  > workbench-example/lexical-direct-compact.json
jq -e '.format == "compact" and
       .result.response.totals.records == 7 and
       .result.response.totals.source_groups == 2' \
  workbench-example/lexical-direct-compact.json
jq '.result.response.groups[0].hits[0].detail_ref' \
  workbench-example/lexical-direct-compact.json \
  > workbench-example/lexical-direct-form-detail-ref.json
DIRECT_BINDING=$(jq -r '.binding_sha256' workbench-example/lexical-direct-compact.json)
"$CLI" workbench form-detail "$LEXICAL_VIEW" \
  --reference workbench-example/lexical-direct-form-detail-ref.json \
  --expected-binding "$DIRECT_BINDING" --format full \
  > workbench-example/lexical-direct-form-detail.json
```

Configured queries accept either an explicit profile name or the current
default. The result bodies agree when both modes select the same prepared
profile, while their effective bindings remain distinct. The Compact key must
be used with the matching configured binding:

```sh
"$CLI" workbench profiles lexical --config "$LEXICAL_CONFIG" \
  --profile lexical --request "$LEXICAL_INPUTS/form-all.json" --format compact \
  > workbench-example/lexical-named-compact.json
"$CLI" workbench profiles lexical --config "$LEXICAL_CONFIG" \
  --request "$LEXICAL_INPUTS/form-all.json" --format full \
  > workbench-example/lexical-default-full.json
jq -e '.configuration.requested_profile == null and
       .result.response.totals.records == 7' \
  workbench-example/lexical-default-full.json
jq '.result.response.groups[0].hits[0].detail_ref' \
  workbench-example/lexical-named-compact.json \
  > workbench-example/lexical-named-form-detail-ref.json
NAMED_BINDING=$(jq -r '.binding_sha256' workbench-example/lexical-named-compact.json)
"$CLI" workbench profiles form-detail --config "$LEXICAL_CONFIG" \
  --profile lexical --reference workbench-example/lexical-named-form-detail-ref.json \
  --expected-binding "$NAMED_BINDING" --format full \
  > workbench-example/lexical-named-form-detail.json
```

After the owned journal advances to r2, the configured Head-bound r1
preparation becomes stale. Refresh the profile to create a new r2 preparation,
then run named and default queries and resolve a Compact reference using the
matching effective binding. Same-projection named and default bodies agree,
while their effective bindings remain distinct. The explicit historical r1
direct root remains usable. The refreshed preparation advances retained history
without rewriting the earlier r1 row. `lexical-expected.json` fixes the
pre-input IDs, order, association unions and literal definitions;
`cursor-protocol-oracle.json` fixes the typed cursor bytes and checksum
independently of a live view.


## Bounded Links traversal

The integrated example writes the fixed Links inputs before creating its
separate journal and catalogue. Inspect the authorities under
`workbench-example/artifacts/links-inputs/`: both typed requests, the expected
Full/Compact rows, cursor protocol, amended fixture spec, and owned, mixed-pack,
and selection dictionaries. The live workbench is under
`workbench-example/links-workbench/`; its explicit historical r1 view remains
usable after the journal advances to r2, while the configured `links` profile
is refreshed to r2.

```sh
LINK_INPUTS=workbench-example/artifacts/links-inputs
LINK_WORKBENCH=workbench-example/links-workbench
CLI=target/debug/vlajmaji
"$CLI" workbench lexical \
  "$LINK_WORKBENCH/view-historical-r1" \
  --request "$LINK_INPUTS/links-depth2.json" --format full \
  > workbench-example/links-direct-full.json
"$CLI" workbench lexical \
  "$LINK_WORKBENCH/view-historical-r1" \
  --request "$LINK_INPUTS/links-depth2.json" --format compact \
  > workbench-example/links-direct-compact.json
jq '.result.response.groups[0].hits[0].detail_ref' \
  workbench-example/links-direct-compact.json \
  > workbench-example/links-direct-detail-ref.json
LINK_BINDING=$(jq -r '.binding_sha256' workbench-example/links-direct-compact.json)
"$CLI" workbench link-detail \
  "$LINK_WORKBENCH/view-historical-r1" \
  --reference workbench-example/links-direct-detail-ref.json \
  --expected-binding "$LINK_BINDING" --format full \
  > workbench-example/links-direct-detail.json
"$CLI" workbench profiles lexical \
  --config "$LINK_WORKBENCH/links-workbench-profiles.json" \
  --profile links --request "$LINK_INPUTS/links-depth2.json" --format readable \
  > workbench-example/links-configured-readable.txt
```

The fixed depth-two request returns thirteen steps across the owned and upstream
publishers. Its three source-clause tokens share the Note’s source evidence, and each token
retains its own exact byte span. `links-expected.json` fixes the
Full/Compact row authority, and `links-cursor-protocol.json` pins typed cursor
bytes independently of a live view. A Compact detail reference belongs to the
effective binding that issued it, so use that response's `binding_sha256` for
detail.

The producer also configures the named `links` profile, refreshes it once for
the retained r1 publication, makes it the default, appends the owned r2
revision, and refreshes that profile again. The r1 direct view remains an
explicit historical view; the profile's configured Head follows r2. Query the
same r2 request through the named and default profile to compare their Full
results, then take a Compact key and its binding from the same named response
for detail:

```sh
"$CLI" workbench profiles lexical \
  --config "$LINK_WORKBENCH/links-workbench-profiles.json" \
  --profile links --request "$LINK_INPUTS/links-depth2.json" --format full \
  > workbench-example/links-named-full-r2.json
"$CLI" workbench profiles lexical \
  --config "$LINK_WORKBENCH/links-workbench-profiles.json" \
  --request "$LINK_INPUTS/links-depth2.json" --format full \
  > workbench-example/links-default-full-r2.json
jq -e --slurpfile named workbench-example/links-named-full-r2.json \
  '.result == $named[0].result' workbench-example/links-default-full-r2.json
"$CLI" workbench profiles lexical \
  --config "$LINK_WORKBENCH/links-workbench-profiles.json" \
  --profile links --request "$LINK_INPUTS/links-depth2.json" --format compact \
  > workbench-example/links-named-compact-r2.json
jq '.result.response.groups[0].hits[0].detail_ref' \
  workbench-example/links-named-compact-r2.json \
  > workbench-example/links-named-detail-ref-r2.json
LINK_NAMED_BINDING=$(jq -r '.binding_sha256' \
  workbench-example/links-named-compact-r2.json)
"$CLI" workbench profiles link-detail \
  --config "$LINK_WORKBENCH/links-workbench-profiles.json" \
  --profile links --reference workbench-example/links-named-detail-ref-r2.json \
  --expected-binding "$LINK_NAMED_BINDING" --format full \
  > workbench-example/links-named-detail-r2.json
```

The named and default queries have matching Full result bodies for this same
request and current preparation. Their configured identities can differ, so a
Compact key must stay paired with the binding from the response that issued
that key; do not transfer detail keys between named, default, or direct views.


## Full offline four-pack and owned-history workflow

This section is for a local source checkout and the exact original inputs listed
below. It uses the native materializers and the strict sealed-pack manifest;
it does not fetch or replace any input, use an embedded Full package, or open a
retained Full/default catalogue. Every output destination is fresh and explicit.
The source files remain separate authorities even when two distributions share
Lensisku lineage. Their seals establish byte integrity, not ownership, a
license grant, or permission to redistribute the corpus. The LLG notice is
retained with that import; the La-Lojban and Sutysisku content holds remain in
effect.

| Original input | Existing path below its root | Bytes | Expected SHA-256 |
| --- | --- | ---: | --- |
| jbotci raw export | `dictionary-en.json` | 17,163,482 | `12e826751c3e50c10dd8df1adafb5f9eb32cdf542b22c4244f284b59d3824dde` |
| jbotci metadata | `dictionary-en.metadata.toml` | 514 | `fe101eead51e0be1b885f3ee1b4555136e333e839baa250ddbd7b84c9af5fc98` |
| LLG gismu list | `lexical/llg-gismu.txt` | 368,403 | `f2fc4cff4f265ad02717d56733b4627ffa78c616db78dcaa44caab694e1dd5b7` |
| LLG cmavo list | `lexical/llg-cmavo.txt` | 145,139 | `544414d61f3ff35dd2cf4e4f79f3c86bf9bb0568eb3403f57288057e16796402` |
| LLG rafsi list | `lexical/llg-rafsi.txt` | 30,883 | `31b90a8551cb3cb9f4d0c6873867af1906e9210c398bf49f20e5c21aa02d0d24` |
| LLG notice | `notices/llg-public-domain.txt` | 1,023 | `9fb2ae1345252ba30999523a0980e1ae69795f5940d2fd9cea8a56c67be18a55` |
| La-Lojban English export | `lexical/la-lojban-en.json` | 7,259,156 | `861e71f66173b383298f6b1823e3ee1ed76db4b9c3fb97e7177de1fd7b667d35` |
| Sutysisku English projection | `lexical/sutysisku-parsed-en.json` | 4,563,090 | `8e80f6626d2e68a17025d52c3f0cf5ea8c32cbfeeb0c2ff1417fcba539bd043d` |

The jbotci files are in the existing Arc02 directory at revision
`3d9fd41d9ed32de26311ee27ec86b6b165985aa1`. The remaining six are under the
existing E02 `raw/` directory shown in the commands. La-Lojban is a second
distribution of shared Lensisku material; jbotci and La-Lojban therefore do
not represent two independent attestations. Sutysisku lineage remains
unresolved, and LLG is the third authority. The four PackIds are
`lensisku-jbotci`, `llg`, `la-lojban`, and `sutysisku`.

Use the defaults supplied by the accepted importers: jbotci revision
`3d9fd41d9ed32de26311ee27ec86b6b165985aa1`, La-Lojban revision
`de47055c68141c283841b1035cd7425bb32bf6ca`, Sutysisku revision
`2c71205747143ea965749ffc3ac3b48ef5c0a517`, and the four LLG component and
notice hashes in the table. These defaults are checked against the complete
local bytes. If an input is absent or does not match, stop that pack’s import;
do not download, repin, substitute a Tiny/Synthetic fixture, or use an existing
normalized store.

From a shell with Rust 1.98 and the workspace's sibling `bauske` checkout:

```sh
SOURCE=/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc04/vlajmaji
JBOTCI=/Users/oubiwann/lab/oxur/vlajmaji/.worktrees/project01-arc02/data/jbotci/3d9fd41d9ed32de26311ee27ec86b6b165985aa1
RAW='/Users/oubiwann/Library/Application Support/ChatGPT-Profiles/personal/codex-home/.chatgpt-projects/g-p-6a8c72d334048191887d4e00472f8430/relojban-poetics/experiments/E02-resource-discovery/raw'
OWNED=/private/tmp/vlajmaji-s06-cdc.u6hg0U/workbench-example/inputs

cd "$SOURCE"
cargo +1.98.0 build --locked --offline -p vlajmaji-cli
TARGET_DIR=${CARGO_TARGET_DIR:-$SOURCE/target}
case "$TARGET_DIR" in
  /*) ;;
  *) TARGET_DIR="$SOURCE/$TARGET_DIR" ;;
esac
CLI="$TARGET_DIR/debug/vlajmaji"
OUT=$(mktemp -d "${TMPDIR:-/tmp}/vlajmaji-four-pack.XXXXXX")
export OUT
mkdir -p "$OUT/packs"

"$CLI" data-pack materialize jbotci \
  "$JBOTCI/dictionary-en.json" "$JBOTCI/dictionary-en.metadata.toml" \
  "$OUT/packs/jbotci"
"$CLI" data-pack materialize llg \
  "$RAW/lexical/llg-gismu.txt" "$RAW/lexical/llg-cmavo.txt" \
  "$RAW/lexical/llg-rafsi.txt" "$RAW/notices/llg-public-domain.txt" \
  "$OUT/packs/llg"
"$CLI" data-pack materialize la-lojban \
  "$RAW/lexical/la-lojban-en.json" "$OUT/packs/la-lojban"
"$CLI" data-pack materialize sutysisku \
  "$RAW/lexical/sutysisku-parsed-en.json" "$OUT/packs/sutysisku"

"$CLI" owned init "$OUT/publisher-journal" --input "$OWNED/owned-seed.json"
"$CLI" owned edit "$OUT/publisher-journal" --expected-head owned-r0 \
  --input "$OWNED/owned-r1.json"
"$CLI" owned edit "$OUT/publisher-journal" --expected-head owned-r1 \
  --input "$OWNED/owned-r2.json"
"$CLI" owned export "$OUT/publisher-journal" \
  --output "$OUT/owned-snapshot.json" --seal "$OUT/owned-seal.json"
```

Each materializer creates a new pack directory containing `release.json`,
`report.json`, and `seal.json`; `Full` is the materializer's pinned local input
kind, not an embedded package or a license statement. The publisher inputs are
the original `owned-seed.json`, `owned-r1.json`, and `owned-r2.json` paths above,
with their previously recorded byte pins. They create the single source
`owned-example`: r0 is empty, r1 has active assertions `a` and `b`, and r2 has
`a`, `b`, and `c`. The history has no Context, Relation, or curation records.
The materialized journal is an authoritative publisher journal, not a catalogue
or workbench database.

Before writing an installed catalogue or destination journal, a Rust consumer
can validate the exported sealed history as a pure value and then explicitly
import it. This uses the public umbrella re-exports and the same original bytes:

```rust,no_run
use std::{env, fs, path::PathBuf};
use vlajmaji::{lexicon::load_owned_snapshot, store::OwnedJournal};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = PathBuf::from(env::var_os("OUT").expect("OUT names the fresh recipe directory"));
    let snapshot_bytes = fs::read(out.join("owned-snapshot.json"))?;
    let seal_bytes = fs::read(out.join("owned-seal.json"))?;
    let snapshot = load_owned_snapshot(&snapshot_bytes, &seal_bytes)?;

    let destination = out.join("imported-owned-journal");
    let (_journal, _report) = OwnedJournal::import_snapshot(&destination, &snapshot)?;
    Ok(())
}
```

`load_owned_snapshot` checks the complete seal, canonical bytes, history, and
reconstructed states without opening a journal. `OwnedJournal::import_snapshot`
then creates a different journal explicitly. Do this before installing the
catalogue or building views in a new Rust consumer. For a shell-only route,
choose these CLI commands instead of running the Rust import example; they
validate the same snapshot and create the same fresh destination journal:

```sh
"$CLI" owned check --input "$OUT/owned-snapshot.json" \
  --seal "$OUT/owned-seal.json"
"$CLI" owned import "$OUT/imported-owned-journal" \
  --input "$OUT/owned-snapshot.json" --seal "$OUT/owned-seal.json"
```

The catalogue manifest is strict version 1. Its relative paths are resolved
from the manifest's parent, and `--all` selects exactly the four records named
here:

```sh
cat > "$OUT/packs.json" <<'JSON'
{
  "version": 1,
  "inputs": [
    {"kind":"sealed","pack_id":"lensisku-jbotci","release":"packs/jbotci/release.json","seal":"packs/jbotci/seal.json"},
    {"kind":"sealed","pack_id":"llg","release":"packs/llg/release.json","seal":"packs/llg/seal.json"},
    {"kind":"sealed","pack_id":"la-lojban","release":"packs/la-lojban/release.json","seal":"packs/la-lojban/seal.json"},
    {"kind":"sealed","pack_id":"sutysisku","release":"packs/sutysisku/release.json","seal":"packs/sutysisku/seal.json"}
  ]
}
JSON

"$CLI" collections install "$OUT/catalogue" \
  --manifest "$OUT/packs.json" --all
```

The four pack identifiers are selection keys, not four upstream authorities.
Installation verifies the new derived generation and writes only to the fresh
explicit catalogue. It does not select a global default or consult ambient
XDG/HOME state.

Build a historical r1 view and separately configure a Head profile for r2.
The strict selection files retain the caller's pack order and explicitly bind
all four packs with no curation contexts:

```sh
cat > "$OUT/selection-r1.json" <<'JSON'
{"version":1,"packs":["lensisku-jbotci","llg","la-lojban","sutysisku"],"disabled":[],"owned":{"revision":"owned-r1"},"contexts":[]}
JSON
cat > "$OUT/selection-head.json" <<'JSON'
{"version":1,"packs":["lensisku-jbotci","llg","la-lojban","sutysisku"],"disabled":[],"owned":"head","contexts":[]}
JSON

"$CLI" workbench build "$OUT/view-r1" \
  --selection "$OUT/selection-r1.json" \
  --catalogue "$OUT/catalogue" --owned "$OUT/imported-owned-journal"
"$CLI" workbench profiles configure --config "$OUT/profiles.json" \
  --name four-pack --selection "$OUT/selection-head.json" \
  --catalogue "$OUT/catalogue" --owned "$OUT/imported-owned-journal" \
  --make-default
"$CLI" workbench profiles refresh "$OUT/view-r2" \
  --config "$OUT/profiles.json" --profile four-pack --expected-revision 1
```

The workbench selection points at the installed four-pack catalogue and one
explicit owned revision. It does not flatten the publisher history into the
catalogue. `owned-r1` remains an explicit historical view; `Head` captures r2
when the profile is refreshed. Catalogue revision, installed generation,
profile/config revision, publication sequence, source revision, and owned
journal revision are separate identities. Direct and configured binding
digests should each be interpreted in their own scope; their equality or
inequality is not a correctness condition.

A finite Full exact query illustrates the historical r1 composition and the
configured r2 profile using the native request and CLI shapes:

```sh
cat > "$OUT/exact-farvi.json" <<'JSON'
{"operation":"exact","spelling":"farvi","options":{"sources":[],"alias_scope":"ActiveOnly","page_size":null,"cursor":null}}
JSON
"$CLI" workbench info "$OUT/view-r1" > "$OUT/info-r1.json"
"$CLI" workbench query "$OUT/view-r1" \
  --request "$OUT/exact-farvi.json" --format full > "$OUT/farvi-r1.json"
"$CLI" workbench profiles query --config "$OUT/profiles.json" \
  --profile four-pack --request "$OUT/exact-farvi.json" --format full \
  > "$OUT/farvi-r2.json"
```

The fixed source/input oracle expects five upstream `farvi` occurrences from
three source IDs, seven at owned r1 from four sources, and eight at owned r2
from four sources. Compare complete rows and provenance to the loaded releases
and owned revisions: jbotci contributes two occurrences from one release,
La-Lojban and Sutysisku are separate captured distributions, LLG contributes
its retained gismu row, and the owned journal contributes its revisioned
assertions. Equal spellings or native numeric IDs in different source/release
identities are not the same record.

The lexical API and `workbench lexical` route expose the accepted stored-Form,
reverse-definition, Links, and computed request families over the same views.
For example, these strict version-one requests keep nullable fields explicit
and distinguish a requested source from the query's output:

```sh
cat > "$OUT/form-fav.json" <<'JSON'
{"version":1,"query":{"operation":"form","value":"fav","role":null,"kind":null},"options":{"sources":["llg"],"alias_scope":"ActiveOnly","page_size":null,"cursor":null}}
JSON
cat > "$OUT/links-farvi.json" <<'JSON'
{"version":1,"query":{"operation":"links","spelling":"farvi","depth":1},"options":{"sources":[],"alias_scope":"ActiveOnly","page_size":null,"cursor":null}}
JSON
cat > "$OUT/distance-farvi.json" <<'JSON'
{"version":1,"query":{"operation":"spelling_distance","anchor":"farvi","max_distance":0},"options":{"sources":[],"alias_scope":"ActiveOnly","page_size":null,"cursor":null}}
JSON
"$CLI" workbench lexical "$OUT/view-r1" --request "$OUT/form-fav.json" \
  --format full > "$OUT/form-fav-r1.json"
"$CLI" workbench lexical "$OUT/view-r1" --request "$OUT/links-farvi.json" \
  --format compact > "$OUT/links-farvi-r1.json"
"$CLI" workbench lexical "$OUT/view-r1" --request "$OUT/distance-farvi.json" \
  --format readable > "$OUT/distance-farvi-r1.txt"
```

A small Rust caller uses the same explicit historical view and typed public
requests. `OUT` is exported by the shell recipe above, so the paths below point
at the artifacts it just created:

```rust,no_run
use std::{env, path::PathBuf};
use vlajmaji::search::{
    DictionaryExactOptions, DictionaryRequest, WorkbenchLexicalOptions,
    WorkbenchLexicalQuery, WorkbenchLexicalRequest, WorkbenchSearchEngine,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = PathBuf::from(env::var_os("OUT").expect("OUT names the fresh recipe directory"));
    let mut engine = WorkbenchSearchEngine::open(out.join("view-r1"))?;
    let exact = engine.execute(DictionaryRequest::Exact {
        spelling: "farvi".into(),
        options: DictionaryExactOptions::default(),
    })?;
    let links = engine.execute_lexical(&WorkbenchLexicalRequest {
        version: 1,
        query: WorkbenchLexicalQuery::Links {
            spelling: "farvi".into(),
            depth: 1,
        },
        options: WorkbenchLexicalOptions::default(),
    })?;
    println!("{exact:?}\n{links:?}");
    Ok(())
}
```

The fixed independent consumer also derives a jbotci `RelationTarget` from the
loaded release's actual native Entry 333, and checks ReverseDefinition
`contains="grow"`, the retained LLG Form `fav`, depth-one Links, and the
bounded computed families against those pre-query inputs. A target is bound to
its actual pack/release identity: the repeated native value 333 in the separate
La-Lojban release is not a substitute. The configured morphology request uses
`Generate` with components `mlatu`, `kerlo`, setting `force_y`; run it on the
configured profile as follows:

```sh
cat > "$OUT/generate-force-y.json" <<'JSON'
{"version":1,"query":{"operation":"generate","components":["mlatu","kerlo"],"setting":"force_y"},"options":{"sources":[],"alias_scope":"ActiveOnly","page_size":null,"cursor":null}}
JSON
"$CLI" workbench profiles lexical --config "$OUT/profiles.json" \
  --profile four-pack --request "$OUT/generate-force-y.json" \
  --format readable > "$OUT/generate-force-y.txt"
```

`vlajmaji-search` keeps morphology off by default; `vlajmaji-cli` enables the
search morphology feature in its dependency, so this CLI operation is
available in the ordinary CLI build. The optional `data-*` embedded packages
are a separate choice and are not needed to read these caller-supplied files.
Keep raw exports, normalized releases, seals, owned snapshots, live journals,
catalogue generations, workbench roots, and profile publications as distinct
artifacts. Rebuild derived stores from retained sealed inputs; do not edit a
seal or treat one layer's version/hash as another layer's identity.
