# Arc04 S03 iteration 01 — ISSUED

This is an in-flight author-recipe correction, not a rejected frozen candidate. Root concrete packet review PASS on 2026-10-26 UTC clears this assignment for issue. CC must read back and acknowledge this current numbered assignment before dependent membership-API edits; its existing exclusive source/index/target interval continues. The initial cc-prompt.md remains byte-preserved (SHA-256 1b101b438067db72cd463466c19a6c48faaa6144eac3f4e897cd7b49e4b64704).

## Assignment and unchanged boundaries

Project01 / Arc04 / S03, first numbered refinement; predecessor cc-prompt.md, issued 359a3b7. Retained Luna/high CC implements; Sol6.1/high CDC independently reviews. All six A04-S03-01 through -06 rows remain OPEN. Current source is the existing dirty implementation on baseline ce1bd238 in codex/project01-composed-sqlite-lookup. CC retains its exclusive source/index/target interval. Root remains read-only; CDC owns planning/design/review. No freeze, acceptance, role transfer, new investigation or budget reset is claimed.

The existing slice-plan.md, ledger.md and initial assignment remain binding except for the additive membership-only API and CLI behavior below. Exactly the same 18 source paths, dependencies, catalogue schema1, resource limits, protected/default/stores, no-push/no-Full-author boundaries, mandatory gates and close obligations apply. No criterion or original control is removed. This amendment does not authorize S04 enable/disable commands.

## Source-grounded decision and implementation

Current collection_profiles.rs::configure validates direct native FILE/ROOT and registrations, acquires the exclusive sibling lock, reads old config, then chooses policy and stages publication. CLI main.rs CollectionsCommand::Configure currently constructs the explicit request with disabled=[] although the CLI exposes no disabled argument. That would silently re-enable an existing disabled pack when the user changes membership. Membership and enablement are independent by the original contract.

Binding: CLI configure preserves the target profile's existing disabled set. A new target profile starts disabled=[]. Other profiles stay semantically unchanged. Explicit library configure(path, CollectionProfileWriteRequest) keeps its issued fields and authoritative disabled behavior; do not add a required flag/field or change disabled to Option.

Add an exported CollectionProfileMembershipWriteRequest { name:String, root:PathBuf, packs:Vec<PackId>, make_default:bool } and CollectionProfiles::configure_membership(path:impl AsRef<Path>, request:CollectionProfileMembershipWriteRequest) -> Result<CollectionProfileWriteReport,CollectionProfileError>. Use the same direct-ROOT/native/one-cwd/error/report contract as explicit configure. Pure library code does not resolve HOME/XDG. CLI calls this membership-only method; no pre-lock config read or read-modify-write in CLI.

Recommended mechanics: both public entry points call one private writer with an explicit private policy mode. For membership mode, choose the latest target profile's disabled IDs only after acquiring the writer lock and strictly reading old config. Do not cache them before lock acquisition. Reuse existing pre-parent-creation argument/selected-ROOT registration observations; after selecting preserved IDs under lock, validate those IDs against that captured new ROOT's registrations before any stage/publication. A root change that cannot represent a preserved disabled ID is a typed failure with old config bytes/revision/hash intact, not an implicit reset. Distinguish pre-lock argument/catalogue observations from under-lock preserved-policy validation; do not claim old policy was validated before it was read.

Retain semantic no-op comparison on the resolved final membership+disabled+default policy, exact bytes/comments/revision preservation, checked revisions, private new modes and existing modes, native relative bases, exclusive owned locks/stages, final expected-old bytes/absence recheck, explicit cleanup and truthful publication uncertainty. Do not duplicate the writer or serialize old state outside its existing lock. New public Result docs require #Errors; teach that CLI membership configuration preserves disabled policy while explicit library requests can deliberately replace it.

## Required discriminating controls and retained original obligations

1. Hand-authored valid disabled profile: CLI changes membership/default yet retains exact disabled IDs; an effective semantic no-op preserves bytes/hash/revision. Explicit library configure with disabled=[] still deliberately clears it.
2. New profile has an empty disabled set and preserves unrelated profile semantics. Existing profile moved to a ROOT without one preserved disabled ID fails typed, leaves exact old bytes/revision/hash, then succeeds with a compatible ROOT/request.
3. Deterministic private-hook or equivalent real-writer test: arrange a membership operation before lock acquisition, commit a newer disabled policy from another writer, then resume membership acquisition. It must preserve the latest locked policy; also retain busy and release/retry counterparts. Hooks are private test-only, not shipped failpoints or global environment mutation.
4. Name the new request/method from an external/public test, including an actual CLI child. Keep native FILE/ROOT, shared-cwd, strict config, complete managed-error contexts and writer-outcome vocabulary: unchanged/committed/indeterminate for writer state, not_applicable for reads. Do not substitute S02 activation tokens such as not_activated.

All pending original writer phase/concurrency/receipt-loss/native, literal four-pack/source/reference/history, missing/corrupt/version payload, stale/fresh-cursor, CLI/error/default, legacy-route, feature/package and independent Full-readonly proofs remain required. Keep readiness01 five-hit/source totals 2/1/2 fixtures distinct from the stock CLI six-hit/source4/1/1 fixture; preserve every failed attempt and pre-expectation census lapse. Do not let this correction replace the covered/pending census requested before final gates.

## Intake, gates and return

Read this entire prompt, the new plan amendment/history, relevant current writer/CLI/tests and their actual source extents before dependent edits; retain reliable complete prior guides rather than repeat unrelated intake. Record exact acknowledgment and the source-grounded readback in artifacts/execution.md. Reconcile a missing or conflicting input with CDC.

Complete current format then lint then check with full raw logs using the fixed /tmp/vlajmaji-p01-a04-cargo +1.98.0, explicit retained CARGO_TARGET_DIR and offline/locked policy; run relevant original feature/package controls. No new target/cache, Full creation or protected-store/default use. Return one scoped clean source candidate with both adjacent parsed coauthor trailers, exact attempts/populations/source-binary-lock identities and explicit ownership handback after sessions end. CDC acceptance is still pending; Linux proof remains a later concrete authorized route, not a local macOS claim.
