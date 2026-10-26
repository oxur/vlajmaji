# Local collection registration and installation

The collection catalogue records local source-pack ownership separately from
the installed dictionary and from user selection. Register, install, list, and
inspect use an explicit ROOT and do not alter the default store, download
inputs, or enable a source. The separate configure, profiles, selection, and
lookup commands use an explicit config path or resolve configuration from
`XDG_CONFIG_HOME/vlajmaji/config.toml`, falling back to
`HOME/.config/vlajmaji/config.toml`.

Prepare source packs locally with the existing `data-pack fixture` or
`data-pack materialize` commands. A manifest can select a sealed pack:

```json
{
  "version": 1,
  "inputs": [
    {
      "kind": "sealed",
      "pack_id": "llg",
      "release": "packs/llg/release.json",
      "seal": "packs/llg/seal.json"
    }
  ]
}
```

Manifest paths are Unicode strings resolved relative to the manifest's parent.
Direct paths supplied to the Rust API or CLI retain native operating-system
path bytes. The manifest is structurally validated before any selected file is
read. `--pack` selects one or more IDs; `--all` requires actual records for all
four supported IDs. Unselected records are not opened. Raw input records name
their existing importer and pins explicitly; the manifest parser does not
silently fill in importer defaults. Synthetic seals require the explicit
`--allow-synthetic` option.

For example, this raw LLG record uses the same four local files, hashes and
explicit locator exercised by the CLI contract test:

```json
{
  "kind": "llg",
  "pack_id": "llg",
  "payload_kind": "synthetic",
  "gismu": "llg-gismu.txt",
  "cmavo": "llg-cmavo.txt",
  "rafsi": "llg-rafsi.txt",
  "notice": "llg-notice.txt",
  "options": {
    "expected_gismu_sha256": "30af4ce20e2309a04cac0b6a8a4f8af05046940e3fcef7bc2d8bd1b4522773be",
    "expected_cmavo_sha256": "b913f30055f7299d64129c2da46f504c89f37ef5208f254389af60b37d3a7261",
    "expected_rafsi_sha256": "1fe380c73539440359596ae97e48a1b3b62b1460bf7319e4252ac39d87bb1c79",
    "expected_notice_sha256": "db400d74ca75a8a66981393306d86cf8cb37860b4828101d013231508e5880ea",
    "base_locator": "https://example.invalid/llg"
  }
}
```

Registering validates the selected pack and stores reference, provenance,
capture, terms, and complete keyed-record digest metadata. Registration copies
no raw or normalized release payload and does not create an installed
generation:

```sh
vlajmaji collections register ./local-catalogue \
  --manifest ./batch.json --pack llg --allow-synthetic
```

Installation is a separate explicit operation. It adds the selected packs to
one new Unified dictionary generation, verifies that derived store with the
`all` verification scope, then publishes its catalogue transaction:

```sh
vlajmaji collections install ./local-catalogue \
  --manifest ./batch.json --pack llg --allow-synthetic
vlajmaji collections list ./local-catalogue
vlajmaji collections inspect ./local-catalogue llg
```

The install report includes the derived store path at
`generation.path.utf8` when the path is ordinary Unicode. Copy that value as
`STORE` to use the existing grouped query and verification commands explicitly;
this does not activate or modify the default store:

```sh
vlajmaji dictionary-query "$STORE" --request '{"operation":"exact","spelling":"farvi"}'
vlajmaji verify all "$STORE"
```

The catalogue retains immutable generation and registration history. An exact
installed repeat verifies incoming and owned bytes and the current derived
store; it does not create another generation. A different release for an
already-installed pack reports `update_required`; replacing an installed pack
belongs to a later lifecycle operation. Failed or interrupted stages remain
identifiable for inspection and are not silently cleaned up.

`list` and `inspect` are read-only. They report the minimum readable format
boundary, current owned-file presence, registration inputs, terms, attribution,
and the recorded verification scope. They do not rehash all release bytes or
perform an `all` audit. Use the explicit store verification command when a new
full integrity check is needed. The generation's `path` descriptor contains a
UTF-8 convenience value when representable and an encoding-tagged hex value for
native path fidelity.

Collection failures are versioned JSON reports on stderr with a stable
category, operation, detail, nullable context fields, input dispositions, and
an explicit activation observation (`not_activated`, `committed`,
`indeterminate`, or `not_applicable`). A failed report does not claim rollback
when the durable outcome is uncertain. Existing dictionary and archive command
output is unchanged.

The catalogue records source rights and notices as supplied by each pack. It
does not grant redistribution permission; held or unknown data terms remain
held or unknown. See the [store guide](../crates/store/README.md) for the
underlying immutable dictionary store and its verification boundary.

## Configured selection and combined lookup

Configuration is a separate strict version-1 TOML file. Each named profile
stores one catalogue ROOT, ordered default pack membership, and a separate
disabled-pack list. Empty membership means no selected packs; `--all` is an
explicit request for all four registered pack records. Configuration changes
never install data or update the existing global default store.
The CLI `collections configure` command changes profile membership while
preserving that profile's disabled-pack list; a new profile starts with no
disabled packs. The library's explicit `CollectionProfiles::configure`
request remains authoritative for callers that deliberately replace that
list.

```sh
vlajmaji collections configure ./local-catalogue \
  --profile research --pack llg --pack sutysisku --make-default
vlajmaji collections profiles
vlajmaji collections selection --profile research
vlajmaji lookup farvi --profile research --source llg
```

`lookup` accepts a word with source/page/cursor flags, or a complete typed
request from `--request` or standard input. Its snapshot reports default
membership, request selection, disabled state, producer eligibility, minimum
derived-store state, and advisory owned-copy presence. The outer versioned
cursor binds the captured profile/catalogue/generation and selection policy;
copy-maintenance warnings do not change that binding. Exact lookup hydrates the
complete typed dictionary and builds its search indexes. A selected installed
payload whose minimum store cannot be opened returns a typed payload error;
registered-only, disabled, or unselected references can be reported from
retained catalogue metadata without claiming the payload exists.
