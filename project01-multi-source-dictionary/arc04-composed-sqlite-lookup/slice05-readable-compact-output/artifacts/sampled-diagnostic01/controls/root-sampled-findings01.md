# S05 sampled diagnostic: findings and next proposal

One diagnostic observer and one exact-PID sampler ran under the Operator's
approval, "I agree to the diagnostic run for data gathering". No source changes,
Cargo commands, retry, acceptance campaign or push occurred.

The sampler successfully attached to observer PID31364, ten-second sampling
request at five-millisecond intervals, and exited0. The observer exited101 at its
existing jbotci-local warm maximum assertion. Observer/sampler elapsed time was
14.510426459 seconds; neither timed out. Outer launcher session85731 exited0,
indicating successful diagnostic collection and integrity/resource checks, not
successful product performance acceptance.

## Observations

The main-thread graph contains 1,752 samples. Counts below are sampled stack
observations in the captured window, not exact per-request elapsed times.

| Observation | Samples | Fraction of main-thread samples |
| --- | ---: | ---: |
| SHA256 compression beneath reference metadata matching | 1,207 | 68.89% |
| Descriptor SQL row retrieval/copy/reset beneath that path | 191 | 10.90% |
| Descriptor decode beneath that path | 49 | 2.80% |
| Entire metadata-matching subtree, including the above | 1,525 | 87.04% |
| Observer's own SHA256 hashing, outside the product query | 92 | 5.25% |

The entire-subtree row overlaps its constituents; do not sum those rows.
All SHA256 compression totals1,299 samples (74.14%), including the92 observer
samples; attributing all74.14% to the product would be incorrect.

Source at commit011af09a explains the observed hot path:

- `crates/search/src/collection_query.rs:1105` opens the current catalogue,
  lists registrations and obtains reusable reference metadata; both Local and
  Catalog reference branches call the cache-aware store accessor.
- `crates/store/src/collections.rs:1730` reads the current descriptor row and
  computes SHA256 over all its bytes before checking the parsed-metadata cache.
- Consequently, a parsed-cache hit still performs the complete descriptor read
  and cryptographic hash. The sample strongly identifies this hashing path as a
  substantial optimization target in the observed workload window.

Saved stdout contains setup plus46 requests (27prime,19measured). All46 response
hashes and byte sizes match the corresponding unsampled failed02 prefix.
The final sampled jbotci-local query measured1,195,746,417ns. This timing is
diagnostic only; the controlling unsampled failure remains1,206,865,292ns against
the unchanged750,000,000ns maximum.

The before/after database hash receipts match the accepted catalogue and fixed G4
dictionary identities. The diagnostic's own target-cache pair is70,746,484,736B
before and after, delta0; this is not an assertion of equality with historical
campaign target measurements. Maximum RSS was2,942,550,016B, below8GiB. Observed
named scratch was738,543,211B, below12GiB, before writing its own final inventory
and receipt. Per-file outputs were below128MiB. These resource checks were after
completion; they do not establish peak scratch or live memory enforcement.

## Limits

This was a ten-second requested window near process start, mixing setup and
priming requests. No timestamped per-request stack stream was collected. Summed
nonoverlapping query durations before the final measured request already exceed
11.32 seconds, so the nominal capture window does not isolate that final request.
The percentages cannot be assigned to the1.207-second unsampled failure alone,
converted into promised seconds saved, or generalized to Compact/Readable modes.
No implementation alternative was tested, and sampling overhead invalidates
numerical acceptance. All six S05 criteria remain OPEN.

## Recommended next change for review, not execution

The smallest candidate is to retain a bounded exact-byte snapshot alongside each
parsed metadata cache entry, then compare freshly read descriptor bytes directly
with that snapshot before reuse. This would replace the sampled SHA256
fingerprint operation with byte equality while retaining the current-read rule.
It requires no new public API or dependency in principle, but needs implementation
and independent review before that claim becomes accepted behavior.

The next approved contract would need to preserve root binding, registration
pruning, entry/weight limits, checked arithmetic, no negative caching, uncached
fallback, SQL payload-kind/header validation, immediate visibility of changed or
malformed current bytes, duplicate/order semantics and typed error precedence.
It must explicitly account for additional retained raw bytes, retain the8GiB RSS
limit, and verify the existing corruption/restoration controls. Raw snapshots
increase memory; current evidence does not establish their eventual RSS or speed.

A database change-token design could later avoid repeated descriptor reads too,
but would require a separate freshness/snapshot contract and verification of
connection lifetimes, out-of-band SQL updates and malformed-data behavior. This
diagnostic provides no basis for silently substituting that design now.

No seventh implementation pass or further campaign is issued by this report.
Stop after evidence preservation; changes and validation need separate approval.

## Retained evidence

Raw outputs and integrity/resource receipts: `sampled-diagnostic01/` under the
existing S05 scratch home. Machine-readable root recomputation:
`root-sampled-analysis01.json`. Launcher:
`root-sampled-diagnostic01.py`, SHA256
6dbfebf1c0fc851b5da7eef0b1810e2c7ab10ac740277c1d08656052cecd3da1.
Sample text SHA256:
076cbfda02b5d597ee7bdfa39d51edd9dcdfe3ec290d3d1254406f6637d8a662.
Canonical copies and their exact file inventory are recorded separately in the
existing slice artifacts; larger wire output may be retained by exact local path
and hash instead of embedding it in planning Git.
