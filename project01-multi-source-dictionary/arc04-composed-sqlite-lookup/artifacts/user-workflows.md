# Arc04 user workflows and state contract

Planning specification1.0,2026-10-25. **Mandatory outcomes; command names/flags below are conceptual, not implemented CLI syntax.** A04 is PLANNED only. Source7b1c565 currently offers explicit JSON dictionary-build/query/info/verify plus local pack materializers; this document does not change published/current behavior or authorize installation.

| Workflow | User's journey / required result | Delivery and acceptance |
| --- | --- | --- |
| W1 — install locally | Select one/several/all4 upstream collections, supply explicit local raw inputs or sealed packs, install once without manually composing canonical JSON/DBs. | S02 native API+CLI takes a declared input manifest/set; accepted adapters normalize when raw is supplied, strict seal/model/derived verification completes before activation. “All” means every one of the four required inputs/packs; missing/Held inputs are not silently skipped. The explicit local batch manifest/provider and partial-failure/retained-staging policy are decided before S02: incomplete batch cannot claim all installed or activate an incomplete requested generation. Exact repeats are idempotent; different content claiming same ID or occupied unknown destination fails, staging never becomes current. No automatic download/publication. |
| W2 — know what is installed | Register metadata if payload is not installed; list and inspect collections/releases, source names/abbreviations, distributor/lineage, pins/terms and local state. | S02 returns accurate source≠distributor≠collection≠DB structure, selected/default/enabled state and verification scope. Registered metadata is not an installed payload or verified state. Read-only inspect never creates missing data. |
| W3 — choose defaults and look up | Configure a selected/all4 default selection; one command exact-searches or resolves a reference, optionally restricted by sourceID/alias. | S03 keeps configuration resolution in CLI, explicit library selection and existing explicitSTORE path. Logical aggregate default is separate from a mandatory one-file topology. Farvi5hits/3groups, all alternatives/full totals/pages, unknown/missing errors and release-scoped native333 are preserved. No manual intermediate composition required. |
| W4 — maintain the selection | Update a collection, disable/re-enable it, remove/reinstall, rebuild indexes and recover interrupted changes while continuing to understand old references. | S04 preserves prior releases/identities/history, stages and verifies before generation activation, rejects conflicting updates, keeps readers coherent, reports stale cursors. Removed payload remains catalogue-known/unavailable; exact reinstall recovers the same identity. No hidden default replacement or cleanup. |
| W5 — read or automate results | Read grouped competing definitions in terminal; choose compact JSON or full detail/JSON, following stable refs for omitted heavy metadata. | S05 declares projection before hydration/serialization. Readable shows named source/abbreviation, every occurrence/definition and provenance/detail locator; compact provides stable occurrence IDs/source-release-capture-native provenance, totals/page/availability summaries and explicit presence states. Full mode retains current semantics. No jq-only path or automatic winner. |

## State dimensions and observable failures

These dimensions are orthogonal, even if one transaction updates several:

| Dimension | Meaning / acceptance distinction |
| --- | --- |
| registered | Known logical collection/release/provenance/terms record; may have no payload. No inferred install/enablement. |
| installed | Required sealed local payload and chosen backend generation exist and passed installation checks; does not itself enable/default-select. |
| enabled | May participate when selected by a query/profile. Disabling retains local bytes, IDs and history. |
| default/profile membership | Named collection/release selection used by the new default route. It is not a source alias and not the physical database count/path. |
| runtime availability | Selected generation is actually readable/compatible now. Missing, corrupt and unsupported versions are distinct from registered-only/disabled, and from zero matches. |
| verification evidence | Install/create checks versus later explicit verify results, identity/generation/time and scope. Ordinary minimal open never claims an all-check audit. |

Enable/default membership is an external collection/user-policy overlay, not an edit to immutable source ReleaseRecord.selected fields or release history. Before S03, specify how that overlay and derived backend selection compose with the original recorded flags.

Register-existing paths are user-managed unless proven/declared otherwise. Unregister/remove must not delete their files by implication. Tool-owned installs may remove only exact owned artifacts under an explicit tested ownership/deletion policy; no arbitrary parents, external paths or followed symlinks.

Unregistered ID is an unknown-selection error. Registered-uninstalled/removed is catalogue-known but payload-unavailable. Disabled explicit selection is a typed disabled/unavailable result, not an empty query. A previously installed artifact that disappears reports missing; corruption/version mismatch reports that failure without fallback to another source/generation. Default selection with no usable data explains why and how to install/enable it; it must not silently create/download or pretend a successful search. Recovery may preserve an earlier intact active generation, but only under the documented explicit activation policy, not a corruption-masking catch-all.

## Required transition counterparts

- Selected/all batch with one missing/invalid input → explicit failure/accounting and prior active generation intact; verify every requested collection disposition, including any retained nonactive staging. No silent partial “all” success.
- Fresh install → inspect installed/verified but activation/default membership only as explicitly requested; identical repeat → unchanged IDs/bytes; mismatched repeat/occupied file/link/directory → failure with prior state intact.
- Register without payload → list catalogue-only; install that release → same catalogue IDs now available. Failed pin/schema/model/write/verification/activation → no new active generation.
- Default-set selected/all4 → same full logical results as explicit equivalent selection. SourceID/alias filters restrict that set; distributor or collection selectors cannot accidentally masquerade as source aliases.
- Disable → installed bytes remain, default/result availability reflects disabled; enable → same unchanged release IDs/results. Saved cursor across membership changes rejects as stale.
- Update → new release/provenance/history plus coherent new generation; in-flight readers finish their pinned old generation or report the declared typed invalidation. Failed update keeps prior active generation; restart recovery handles pre/post-switch interruption.
- Register-existing external path → later remove unregisters only and leaves exact bytes; tool-owned path removal uses the declared exclusive ownership counterpart.
- Remove → retained catalogue/ref history, unavailable payload, no successful zero substitution. Exact reinstall → original stable refs recover; a different release is not silently substituted. Rebuild → same logical tuples/results with new derived artifact identity as declared, not new linguistic attestations.
- Explicit scratch migration of old standalone/default Lensisku store → old bytes preserved, backcompat explicitSTORE works, selection switch requires explicit action and is reversible. No implementation/reviewer run writes the actual user's installed default unless separately requested.

## Representation and detail contract

Existing dictionary-query JSON remains full and compatible on its existing route. New readable/compact naming/default TTY behavior, JSON schema/version, stable detail-reference form and any new request projection fields must be decided before the owning prompt. Full and compact request semantics share identical occurrence selection/global ordering/totals/cursors; representation is not a hidden filtering policy. Page-size-zero preserves full counts and selection summaries without hit hydration.

Compact must not hydrate/copy/serialize unrelated whole-release assertion-ID inventories. It must not first construct the old multi-megabyte full response and discard it. Projection-specific data access may be necessary, but this plan does not preselect a general lazy-SQL rewrite. Every displayed/machine hit retains occurrence identity, source label/abbreviation, definition and provenance references; optional fields distinguish missing/null/unknown as declared. Heavy native fields/contexts/release inventories can be omitted only under a documented projection with a stable explicit detail operation. The full detail route must recover the original values and every alternative, including excluded raw reference targets. Readable mode signals partial pages and does not truncate silently; wrapping/terminal-width, Unicode and stdout/stderr/exit behavior are specified/tested before delivery.

Arc03's full farvi capture was7,160,366B includingLF for5hits/3groups, with canonical JSON7,160,365B. It is a historical local observation with merged stderr/stdout and source-supported stdout attribution. It does not predict compact latency or justify dropping provenance. Resolve numeric bytes/latency/setup budgets before affected code/trials, and measure actual baseline versus candidate with hydration/serialization separated. No accepted threshold exists yet.

## Scope and future readiness

The configured catalogue/default workflow is new capability: follow source AGENTS' confyg requirement if adding file configuration; no custom ad-hoc configuration framework. Exact locations/version/override rules and logicaldefault-to-topology mapping remain design decisions after the comparison, while current explicitSTORE and native-path behavior stay compatible. Keep tests in isolated temporaryXDG/HOME/scratch, new private directories where supported, no real default changes/network. Ordinary query/list/info/resolve remains read-only.

A05 owns the mutable owned source/editor/history/export package, not an upstream install made writable here. A06 owns broader Q01–Q06+owned/removal/reload/project-wide integration; A04 still delivers W1–W5 and preserves current behavior. Eventual end-to-end recipes must actually run all5journeys through library/CLI after implementation; this planning document is not evidence of those runs.
