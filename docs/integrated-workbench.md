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
