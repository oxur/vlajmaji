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
