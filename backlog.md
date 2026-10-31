# Planning backlog

Repository-wide candidates for future investigation or implementation. Entries
link to their originating records; evidence stays in the owning project's
artifacts. Capture does not authorize execution, change an active plan or close
an acceptance obligation. Promote an item into an approved plan before work starts.

Use stable `BL-NNNN` identifiers; retain completed or declined items with their
resolution and links rather than renumbering or deleting them.

## BL-0001: Replace per-query SHA256 in reference metadata cache freshness checks

- **Status:** Proposed; unscheduled, implementation not authorized.
- **Raised:** 2026-10-27 (America/Chicago).
- **Origin:** Project01 / Arc04 / Slice05 readable and compact output.
- **Source examined:** `011af09a44abc153b8b8253f70d84147e21a6bb0`.
- **Owner:** Unassigned until planning/approval.

### Problem and evidence

The parsed-metadata cache still reads and SHA256-hashes complete descriptor bytes
on every reference access before reusing a decoded entry. In the diagnostic's
1,752 captured main-thread samples, 1,207 (68.89%) were in SHA256 compression
beneath metadata matching. Another 92 SHA samples belonged to the observer and
must not be attributed to the product. The mixed setup/priming/request window
does not isolate the final failed measured lookup.

The controlling unsampled `jbotci-local` result was 1.206865292 seconds against
the unchanged 750 ms maximum. All six S05 criteria remain OPEN. Recording this
candidate neither defers nor waives those obligations or the 250 ms median target.

**Subsequent Operator disposition:** Duncan accepts the reported current timings
as-is at source011af09a. This optimisation is optional future work; the observed
latency overrun no longer requires a corrective pass. The [explicit exception and
remaining acceptance obligations](project01-multi-source-dictionary/arc04-composed-sqlite-lookup/slice05-readable-compact-output/ledger.md#operator-disposition-current-timings-accepted)
are recorded in the slice ledger. Original targets, measurements and failures
remain preserved; no unexecuted result or resource gate is accepted by inference.

### Candidate approach and trade-offs

Retain a bounded exact-byte snapshot beside each parsed cache entry. Compare
freshly read bytes directly with the retained bytes before reuse, replacing the
cryptographic fingerprint operation. On changes, invalidate before decoding and
never fall back to stale successful metadata or cached errors.

Preserve root binding, current-row reads, payload-kind/header checks, pruning,
clear/count/weight accounting, uncached fallback, duplicate/order semantics and
typed error precedence. Specify and account for additional retained raw bytes;
the existing 512 MiB serialized-weight proxy is not a decoded-plus-raw heap bound.
Byte comparison and SQLite reads still cost time. Actual latency and RSS benefits
remain unmeasured. A database change-token design would need separate analysis of
snapshot lifetimes and out-of-band changes; it is not implied by this item.

### What a future approved implementation must establish

- Existing changed-byte, malformed-byte, SQL-kind, restore, root and error-order
  controls retain their behavior, including same-engine use and cache fallback.
- Added raw-byte retention is explicitly bounded/accounted, and measured RSS
  remains within the unchanged 8 GiB ceiling.
- Relevant library, feature-boundary and public API/CLI checks pass under the
  then-approved validation plan.
- Unsampled, comparable measurements establish whether the unchanged 250 ms
  median / 750 ms maximum goals are met; the illustrative projections are not
  acceptance evidence. Independent disposition and remaining S05/arc gates stay
  with the active plan.

### Links

- [Findings and proposed change](project01-multi-source-dictionary/arc04-composed-sqlite-lookup/slice05-readable-compact-output/artifacts/sampled-diagnostic01/controls/root-sampled-findings01.md).
- [Timing scenarios, formula and assumptions](project01-multi-source-dictionary/arc04-composed-sqlite-lookup/slice05-readable-compact-output/artifacts/metadata-byte-comparison-projections.md).
- [Root's recomputed sample counts and response parity](project01-multi-source-dictionary/arc04-composed-sqlite-lookup/slice05-readable-compact-output/artifacts/sampled-diagnostic01/controls/root-sampled-analysis01.json).
- [Independent CDC sample analysis](project01-multi-source-dictionary/arc04-composed-sqlite-lookup/slice05-readable-compact-output/artifacts/sampled-diagnostic01/controls/sampled-diagnostic01-cdc-analysis.json).
- [Raw sample](project01-multi-source-dictionary/arc04-composed-sqlite-lookup/slice05-readable-compact-output/artifacts/sampled-diagnostic01/output/sample.txt) and [diagnostic receipt](project01-multi-source-dictionary/arc04-composed-sqlite-lookup/slice05-readable-compact-output/artifacts/sampled-diagnostic01/output/receipt.json).
- [Exact diagnostic preservation manifest](project01-multi-source-dictionary/arc04-composed-sqlite-lookup/slice05-readable-compact-output/artifacts/sampled-diagnostic01/manifest.json).
- [Unsampled failure disposition](project01-multi-source-dictionary/arc04-composed-sqlite-lookup/slice05-readable-compact-output/review-numeric02.md) and [failure evidence manifest](project01-multi-source-dictionary/arc04-composed-sqlite-lookup/slice05-readable-compact-output/artifacts/numeric-failed02/manifest.json).
- [Owning slice plan](project01-multi-source-dictionary/arc04-composed-sqlite-lookup/slice05-readable-compact-output/slice-plan.md) and [open acceptance ledger](project01-multi-source-dictionary/arc04-composed-sqlite-lookup/slice05-readable-compact-output/ledger.md).

## BL-0002: Investigate exclusive temporary-root allocation in CLI maintenance tests

- **Status:** Addressed for the named CLI helper during the 2026-10-31 delivery; original collision cause remains unproven.
- **Raised:** 2026-10-29 (America/Chicago), Arc05/S02.
- **Observed source:** inherited collections_maintenance_contract fresh_root helper, unchanged by854865a7.
- **Owner:** Root delivery, with retained CDC read-only review; completed for the named helper.

The original S02 Synthetic all-features attempt failed at create_dir AlreadyExists before the named test's product calls. Collision cause and exact collided path are unproven. One explicitly authorized corrected-source archive with fresh owned environment passed; it does not establish a helper fix or causal diagnosis. Investigate actual allocation evidence and consider exclusive bounded temporary-root creation rather than relying only on PID/time names. A future task must preserve baseline failures, distinguish fixture setup from product assertions and validate portability; this entry neither authorizes an out-of-scope repair nor adds a current gate.

Evidence: [original failed raw](project01-multi-source-dictionary/arc05-owned-dictionary/slice02-reviewed-curation/artifacts/logs/vlajmaji-s02-all-features-archive-final01.log), [sole corrected raw](project01-multi-source-dictionary/arc05-owned-dictionary/slice02-reviewed-curation/artifacts/strict-archive01/all-features.log), [existing chronology](project01-multi-source-dictionary/arc05-owned-dictionary/slice02-reviewed-curation/artifacts/execution.md), [CDC verification](project01-multi-source-dictionary/arc05-owned-dictionary/slice02-reviewed-curation/cdc-verification.md) and [retrospective](project01-multi-source-dictionary/arc05-owned-dictionary/slice02-reviewed-curation/cdc-retrospective.md).

### Delivery disposition — 2026-10-31

The original 2026-10-29 status was “Proposed, unscheduled; no implementation authorized or shipped.” Duncan's later explicit README/main-merge request opened a delivery interval. Fresh PR macOS CI reproduced AlreadyExists in the named CLI allocator; a separate push macOS job later reproduced it in the store policy test allocator. Both scoped test-only repairs use exclusive bounded allocation and per-process sequences, with deterministic occupied-directory/file preservation controls. They are shipped through [PR7](https://github.com/oxur/vlajmaji/pull/7) at main `cb13d377`; final candidate and merged-main Linux/macOS CI passed. [Delivery record and raw failures](project01-multi-source-dictionary/artifacts/arc05-delivery/delivery.md) preserve the evidence. This resolves the named allocation weakness without claiming a causal diagnosis of the historical collision or an audit of every temporary-root helper. The original S02 failures and retrospective above remain unchanged.
