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

The search workspace temporarily resolves relojban-phonology through the
relative path
../../../Library/Application Support/ChatGPT-Profiles/personal/codex-home/.chatgpt-projects/g-p-6a8c72d334048191887d4e00472f8430/relojban-poetics/crates/relojban-phonology.
The phonology crate remains in that separate checkout; it was not copied or
vendored. The bridge is temporary. A separate migration to the already
selected bauske repository is queued after this checkpoint is independently
accepted. A standalone checkout without that relative sibling is not
currently buildable.

The source corpus and its acquisition records remain in the Relojban poetry
research checkout. Synthetic tests run by default. The complete corpus
integration test is explicitly ignored by default and requires
RELOJBAN_RESEARCH_ROOT pointing to that checkout. Run it from this
repository with:

    RELOJBAN_RESEARCH_ROOT="/Users/oubiwann/Library/Application Support/ChatGPT-Profiles/personal/codex-home/.chatgpt-projects/g-p-6a8c72d334048191887d4e00472f8430/relojban-poetics" cargo +1.98.0 test --locked -p vlajmaji-lexicon --test contract full_corpus_import_reconciles_all_input_populations -- --ignored --exact

The corpus importer accepts a research root and output archive path:

    cargo +1.98.0 run --locked -p vlajmaji-lexicon --example import_corpus -- "/Users/oubiwann/Library/Application Support/ChatGPT-Profiles/personal/codex-home/.chatgpt-projects/g-p-6a8c72d334048191887d4e00472f8430/relojban-poetics" "/tmp/vlajmaji-corpus-replay.archive"

The preserved corpus archive used for migration checks has SHA-256
4e5c30e978bde631349e1107599f3ef9738c1f63257ba280e146d2256c7cef5a and is
28,320,417 bytes. This identifies the replay oracle, not a copied corpus
distribution.

The initial V README description and its existing LICENSE and .gitignore
are preserved as starting repository materials. Only V's preexisting LICENSE
is the destination repository notice. R has no projectwide license selection,
and none of the moved source files is an originating Apache notice file. This
migration does not declare the source code relicensed and adds no inferred
Cargo license metadata.
