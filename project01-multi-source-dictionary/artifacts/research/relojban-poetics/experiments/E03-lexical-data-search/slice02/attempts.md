# Slice02 acquisition and probe attempts

## places.rs pinned raw acquisition — sandbox attempt

- Requested object: `https://raw.githubusercontent.com/lojban/jbotci/3d9fd41d9ed32de26311ee27ec86b6b165985aa1/crates/jbotci-dictionary/src/places.rs`
- Started: `2026-10-23T15:14:35Z` (UTC, second precision)
- Command: `curl --fail --location --silent --show-error --dump-header /tmp/p02a02-jbotci-places.headers --output /tmp/p02a02-jbotci-places.rs <pinned raw URL>`
- Outcome: exit 6, `curl: (6) Could not resolve host: raw.githubusercontent.com`.
- Classification: sandbox DNS denial only; this does not establish that the pinned source is absent or incompatible. The attempt produced no response body or headers. Follow-up will use the authorized ordinary-TLS execution substrate.

## places.rs pinned raw acquisition — first ordinary-TLS URL correction

- Requested object: `https://raw.githubusercontent.com/lojban/jbotci/3d9fd41d9ed32de26311ee27ec86b6b165985aa1/crates/jbotci-dictionary/src/places.rs`
- Started: `2026-10-23T15:14:48Z`; completed with HTTP failure at approximately `15:14:52Z` (the command printed no completion timestamp after `curl` failed).
- Outcome: ordinary TLS resolved and returned HTTP 404. `curl --fail` did not preserve a response body. The URL owner was inconsistent with the prior pinned source capture's GitHub links (`int19h`); this is a command/source locator error, not evidence that the pinned file is absent.
- Follow-up: retry the same commit/path under the owner established by the captured repository metadata, preserving this attempt.

## jbotci public library build — command-misuse retry allowance

- Pinned isolated checkout: `/tmp/p02a02-jbotci-cc-s02-20261023`, HEAD `3d9fd41d9ed32de26311ee27ec86b6b165985aa1`; clean detached checkout.
- Attempt started: `2026-10-23T15:20:12Z`.
- Intended command: `timeout 1000 cargo build --locked -p jbotci-dictionary --features import`.
- Outcome: exit 127 before Cargo ran because this host has no `timeout` executable (`/opt/homebrew/bin/bash: line 1: timeout: command not found`). This is a command misuse only, not a build result. The single allowed diagnostic retry is a direct Cargo invocation, monitored against the total 20-minute cap.

## jbotci pinned package build — direct Cargo retry and outcome

- Isolated checkout: `/tmp/p02a02-jbotci-cc-s02-20261023`, HEAD `3d9fd41d9ed32de26311ee27ec86b6b165985aa1`, clean detached tree.
- Command: `cargo build --locked -p jbotci-dictionary --features import`.
- Started: `2026-10-23T15:20:12Z`; finished approximately `15:21:22Z` (shell output reports Cargo compile duration 29.45 s; rustup stable component synchronization preceded Cargo).
- Outcome: exit 0. Cargo used stable-aarch64-apple-darwin and synchronized stable to Rust 1.99.1 (dated 2026-10-15); it resolved the pinned repository lock and compiled `jbotci-dictionary` with `import` plus its local `bityzba`, morphology, phonetic and `vec1` dependencies. Registry resolution/download succeeded over ordinary TLS. Cargo lock and dependency tree are preserved under `probes/jbotci/`. This is a package build only; no runtime conclusion yet.

## jbotci public API runtime probe — sandbox cache-write limit

- External probe source copied to `/tmp/p02a02-jbotci-public-probe`, outside both the jbotci and Relojban workspaces. It calls the public importer and places map/segment functions.
- Command: `cargo run --offline --manifest-path /tmp/p02a02-jbotci-public-probe/Cargo.toml -- <captured La> <captured Lensisku farvi> <six labelled fixture paths>`.
- Started after the successful package build; ended in about five seconds with exit 101 before compilation/runtime. Cargo resolved the standalone probe lock, then could not unpack cached `tinyvec v1.12.0` under the sandbox-protected `~/.cargo/registry/src/...` (`Operation not permitted`). This is a local cache write restriction; it is not an importer/compiler outcome. The standalone probe lock remains in the temporary probe and will be copied into evidence before an ordinary-TLS retry using an isolated writable Cargo cache.

## Local Rust dependency check — sandbox attempt

- Started after source intake receipt and initial library skeleton creation; rustfmt was applied first.
- Command: `cargo check -p relojban-lexicon --all-targets` from `R/`.
- Outcome: failed before compilation while Cargo retried DNS for `index.crates.io`; final error says it could not download crates.io `config.json` while resolving workspace dependency `clap` (exit 101). This is sandbox DNS denial, not a Rust source failure or upstream incompatibility. No compiler diagnostics were produced. The ordinary-TLS build attempt is pending.

## jbotci public importer/places runtime evidence correction

- The preserved initial `runtime.log` was invoked with `research/lexical-data-search.md` as its final argument instead of the existing `farvi-extra-field.json`. It printed `expected value`; that output did not test unknown-field rejection. The original argv/log remain unchanged.
- Under refinement01 at planning `4225a97`, within the original 20-minute cap and without rebuilding upstream, a corrected standalone probe was prepared. Its first compile attempt failed because `DictionaryEntry.notes` is `String`, not `Option<String>`; the probe was corrected to pass the value directly. Its next execution printed the intended fixture outcomes but then panicked on a probe-only loop endpoint (`index 9` with `len 9`); the full argument vector had indices 0–8. The final bounded loop was corrected and rerun successfully. The compile diagnostic and panic were emitted to the task tool output; the final successful output is preserved at `probes/jbotci/runtime-corrected.log`.
- Corrected results: the actual `farvi-extra-field.json` rejects `source_extra_control` as an unknown field; the unknown-native-kind fixture rejects `new-native-class`. `farvi-missing.json`, `farvi-null.json`, and `farvi-empty.json` each import one row with notes `""`, selmaho `None`; rafsi projection is `fav` in each. In the two-row real farvi copy, IDs 69002 and 333 remain before the optional best reduction, which drops one when explicitly called. The 69002 definition segments as places 1–5 and ID333 as 1–4. A corrected alias control based on `$x_1=b_1$` resolves notes `$b_1$` to Place(1) and leaves `$z_7$` unmapped. This supersedes only the earlier mis-argument inference; it does not delete or replace the initial attempt.

## Local validator and contract-test corrections

- The first recursive duplicate-key walker mishandled scalar JSON values when `serde_json/arbitrary_precision` was enabled. It was corrected to recurse through object/array values only, retaining large numeric lexemes as scalars; duplicate keys at root and nested object levels remain explicit negative cases.
- An early full-corpus test showed the LLG rafsi text parser had split the row such that the final gloss text was malformed. It was corrected to locate tokens in the trimmed line by offsets and preserve the complete gloss remainder. The corrected full-population test now passes.
- An additional archive identity guard was added after the first 5/5 integration + 4/4 doctest pass. It verifies namespace metadata and the raw-byte hash when rebuilding. The test now tampers with an archived raw-byte value and checks rejection.
- `cargo clippy --workspace --all-targets --locked -- -D warnings` first reported one collapsible nested `if` in the hash guard; it was simplified and the same command passed. A separate all-target test build first caught `&&str` conversion in a required-field error; this was fixed and the crate suite passed.

## Outside-workspace consumer corrections

- First consumer invocation from its evidence directory failed because Cargo associated it with the parent workspace. A local `[workspace]` root was added to the probe package, and its path dependency was corrected to the lexicon crate. The first temporary out-of-workspace compile then found a borrow held across proposal insertion/reimport; the consumer now records the length and ends the borrow before mutation. The final independent build/run is preserved at `probes/external-consumer/run.log`; no private library or serde types are used by consumer source.
- When type checks were broadened, a full-corpus test correctly rejected La `gloss_keywords` as non-string-array: actual captured values are arrays of `{word, meaning}` objects (with nullable `meaning`). This was a schema-modeling error in the validator, not malformed source. Validation was narrowed to the known top-level array type while raw nested members remain untouched; the negative case now supplies a string where an array is required. The full suite passes with that correction.
- A plain `cargo fmt` call briefly failed in the rustup shim while the shared toolchain was being accessed (`could not remove 'setup' file: .../.cargo/bin/rustup-init: Operation not permitted`). Running the installed stable toolchain binaries directly through PATH succeeded; this was an invocation-layer restriction, not a formatting or compilation diagnostic.
- An initial attempt to require every keyword array member be a string failed against actual La rows: those arrays contain `{word, meaning}` objects. The final validator checks the declared top-level array type for these fields and retains nested members in raw JSON. Form arrays are separately constrained to string members because those are native source form assertions. Corpus/full-suite checks passed after this distinction.
- Source-form projection initially represented each space-delimited rafsi source string as one assertion. Inspection of actual `farvi` (`fav farv`) and `snime` (`si'e snim`) rows showed that this underrepresented the distinct forms. The raw field lexeme was already retained, but the derived form view is now whitespace-tokenized without normalization, and raw rafsi arrays are validated as string lists. Full-corpus counts and controls were rerun; the final source-reported assertion count is 1,988 (versus the earlier one-per-populated-field count 1,417).

## Refinement02 corrected candidate rerun

- The first post-refinement corpus-example invocation supplied only the archive output path in the positional slot for repository root. It exited 1 with OS error 20 (`Not a directory`) before importing records. The output is preserved in `probes/corpus-example-misargument.log`; the pre-refinement successful output was recovered byte-for-byte from the candidate commit into `probes/corpus-example-initial.log`.
- The corrected invocation passes repository root and archive output separately and succeeds under Rust 1.98.0. Its regenerated archive uses the now-pinned La Git repository/revision/path from accepted E02 metadata; raw E02 bytes are unchanged. Its cross-reference count changes from 13,522 to 13,521 because the corrected parser excludes the false-positive TeX brace; exact valid target spans remain covered by the updated contract tests.

## Refinement02 source-fidelity and replay corrections

- The parser rule is now `see-also-braces-v2`: an explicit ASCII-insensitive `See also` marker must have non-alphanumeric boundaries; extraction stops at sentence/line/semicolon delimiters and excludes brace targets occurring within `$...$` math spans. Tests preserve the genuine farvi links and reject same-clause TeX `{1}`, the `Foresee` substring, unmarked free braces, and text after clause boundaries.
- JSON field presence treats JSON `null` as Null. LLG text views always retain present fields as Value, including literal `null` and known blank fixed rafsi columns; those blanks create no assigned-form assertion. LLG rafsi source rows now expose a gloss and no definition prose. Original source bytes/lines remain the fidelity authority.
- Proposal IDs now use a domain-separated length-prefixed tuple encoding. Proposal itself no longer implements public deserialization; archive v2 deserializes a private DTO and validates namespace/identity through the constructor. The retained v1 branch exists only to reconstruct archives made by preserved candidate `07b96b4`, migrating them to v2 IDs. It is not a general migration/adoption commitment. Tests cover NUL tuple distinction, v1 migration, forged v2 namespace/identity and roundtrip.
- The refreshed full import remains 21,241 rows and 16 snapshots: 19,696 definition-bearing rows plus 1,545 assigned-rafsi gloss rows. The 16 exact farvi rows include 13 Lensisku rows across languages; no source alternatives are reduced. The example archive is now v2 and 28,320,417 bytes. Its cross-reference count is 13,521; the removed record was the same-clause TeX false positive.
- The external consumer was rerun from a copied out-of-workspace directory on Rust 1.98.0 and now compares each of the 16 archived source byte strings with the corresponding accepted E02 raw file. It passes with `raw_source_bytes=16/16`; all-language farvi count is 16, Lensisku farvi count 13, exact `si'e` 2 and `sie` 0, and a separate proposal remains.
- The final Rust 1.98.0 logs are under `probes/rust-1.98-final2-{fmt,check,clippy,test,doc}.log`; 66 unit/integration tests and 5 doctests pass. The E02 source guard passes 103/103, the existing archive guard passes, Project01 phonology diff is empty, and `verify_manifest.py` verifies the current evidence/candidate inventory. Exact commands are in `README.md`.
