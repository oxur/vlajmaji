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
