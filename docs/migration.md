# Research lexical migration

This repository contains four lexical/search packages migrated from the
Relojban poetry research checkout at accepted source ref
7a05516b5d429f3184eaf6a2bde2fd9c5d12e9bf. The migration preserves the
source archive, record identity encoding, proposal hash domain, search policy
IDs, query JSON, and relojban-search executable name. It changes package
identities and workspace paths.

| Package | Path |
| --- | --- |
| vlajmaji-lexicon | crates/lexicon |
| vlajmaji-store | crates/store |
| vlajmaji-search | crates/search |
| vlajmaji-cli | crate/cli |

The search workspace now resolves `bauske-phonology` from the sibling
repository at `../bauske/crates/phonology`. Its package and Rust library
identities were renamed in the Bauske migration; phonological behavior and the
`relojban-search` executable remain unchanged. A standalone V checkout must
have the Bauske sibling at that path to build.

The source corpus and its acquisition records remain in the Relojban poetry
research checkout. Synthetic tests run by default. The complete corpus
integration test is explicitly ignored by default and requires
RELOJBAN_RESEARCH_ROOT pointing to that checkout. Run it from this
repository with:

    RELOJBAN_RESEARCH_ROOT="/Users/oubiwann/Library/Application Support/ChatGPT-Profiles/personal/codex-home/.chatgpt-projects/g-p-6a8c72d334048191887d4e00472f8430/relojban-poetics" cargo +1.98.0 test --locked -p vlajmaji-lexicon --test contract full_corpus_import_reconciles_all_input_populations -- --ignored --exact

The corpus importer accepts a research root and output archive path:

    cargo +1.98.0 run --locked -p vlajmaji-lexicon --example import_corpus -- "/Users/oubiwann/Library/Application Support/ChatGPT-Profiles/personal/codex-home/.chatgpt-projects/g-p-6a8c72d334048191887d4e00472f8430/relojban-poetics" "/tmp/vlajmaji-corpus-replay.archive"

Historical E02/E03 manifests and probes remain unchanged records of Arc02 runs against the original source baseline `7a05516b5d429f3184eaf6a2bde2fd9c5d12e9bf`. Replaying those historical commands requires a separate checkout of that baseline; the old source paths no longer exist in the current R checkout. The renamed V build, CLI and explicitly gated corpus command documented here are the active commands for this destination.

The preserved corpus archive used for migration checks has SHA-256
4e5c30e978bde631349e1107599f3ef9738c1f63257ba280e146d2256c7cef5a and is
28,320,417 bytes. This identifies the replay oracle, not a copied corpus
distribution.

Current workspace commands use the renamed package names and preserved CLI binary:

    cargo +1.98.0 build --locked --workspace
    cargo +1.98.0 run --locked -p vlajmaji-cli -- build <ARCHIVE> <STORE>
    cargo +1.98.0 run --locked -p vlajmaji-cli -- info <STORE>
    cargo +1.98.0 run --locked -p vlajmaji-cli -- search <STORE> --query '{"operation":"exact","spelling":"farvi"}'

The source-to-destination package/path map and historical replay locations are
recorded in R's `research/vlajmaji-migration.md`; the subsequent phonology
namespace migration is recorded in `research/bauske-migration.md`.

The initial V README description and its existing LICENSE and .gitignore
are preserved as starting repository materials. Only V's preexisting LICENSE
is the destination repository notice. R has no projectwide license selection,
and none of the moved source files is an originating Apache notice file. This
migration does not declare the source code relicensed and adds no inferred
Cargo license metadata.

## 0.2.0 development command

The migration and 0.1.0 command identities above are historical. Starting with
the 0.2.0 development cycle, the CLI binary is `vlajmaji`. Package names and
archive/search identities remain unchanged. `make build` installs the current
debug binary to `./bin/vlajmaji`; the Cargo commands above also select this
renamed binary automatically.
