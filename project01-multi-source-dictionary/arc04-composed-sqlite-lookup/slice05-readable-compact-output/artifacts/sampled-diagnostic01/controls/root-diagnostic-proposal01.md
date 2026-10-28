# Proposed sampled diagnostic — NOT AUTHORIZED OR EXECUTED

Purpose: locate the remaining Full Local/reference cost at source commit
011af09a44abc153b8b8253f70d84147e21a6bb0 before proposing another implementation pass.
This is a diagnostic observation under sampling overhead, not performance acceptance.

## Evidence already available

Replacement Full campaign failed stage13: measured jbotci-local whole query
1,206,865,292ns against the unchanged 750,000,000ns maximum. Twelve fresh stages
passed; stages14–22 did not execute. All six S05 criteria remain OPEN.

Root independently checked the retained failed02 manifest: 103 payloads,
2,682,593 bytes, exact hashes/sizes/original copies/no extra payloads. Saved stdout
contains setup plus 46 requests: 27 prime and 19 measured, matching the frozen
request prefix and preinput facts/error category. All46 request wire hashes and
sizes equal failed01. These comparisons executed no product/data query.

Diagnostic priming observations (whole nanoseconds, old -> new):

- jbotci-local: 1,245,563,917 -> 1,539,841,583.
- jbotci-catalogue-native333: 19,844,226,833 -> 1,292,680,250.
- la-catalogue-native333: 2,701,522,125 -> 1,260,235,583.
- llg-component: 870,683,833 -> 1,239,240,416.
- sut-native-entry: 6,320,514,875 -> 1,262,606,792.
- wrong-role: 821,476,334 -> 1,228,742,750 (whole error; no query profile).
- missing: 20,528,113,375 -> 1,662,715,625.

These are individual saved primes, not three-round acceptance or isolated phase
measurements. They show neither a universal improvement nor the residual cause.

Historical S04 settled-inventory.json (SHA256
400cce12fffcf746e06d8b970a0384aa3bd65c206483e19e451e4dd93dcf7f7b)
records the entire catalogue file at 170,835,968 bytes, below the 536,870,912-byte
cache serialized-weight bound. This is historical size evidence, not a current
DB read or direct cache-admission observation. A new descriptor-length-only
query would not adequately explain the remaining time; do not execute one here.

## Concrete requested authorization

Authorize one diagnostic observer launch, with one concurrent macOS sample
invocation targeting that exact observer PID, and preservation/review of results.
No source edit, Cargo invocation, implementation pass, acceptance campaign,
retry, push, import, installation, deletion, or access to the user's default store.

1. Verify the frozen observer/config/workload identities against packet04;
   verify the two explicitly fixed campaign databases against recorded SHA256s.
   No discovery traversal, extra data store, or input substitution.
2. Use a fresh output directory under this existing D5 scratch root. Isolate HOME,
   XDG_DATA_HOME and XDG_CONFIG_HOME there as in the frozen runner. Keep source,
   observer, explicit Full profile, ROOT and embedded54-request workload unchanged.
3. Launch EXACTLY once:
   `/tmp/vlajmaji-p01-a04-s05-cdc/numeric-frozen-binaries02/numeric-observer warm-full full 1 NEW_OUTPUT_DIRECTORY`.
   Capture raw stdout/stderr, argv, environment, timestamps, exit, RSS and exact PID.
   Preserve existing observer stop-on-first-failed-budget behavior. Do not run the
   22-stage runner or bypass its performance assertions.
4. Sample only that PID once using `/usr/bin/sample PID 10 5 -mayDie -file SAMPLE_OUTPUT`.
   This requests ten seconds at five-millisecond intervals. Start promptly after
   obtaining the launched observer PID; never select an unrelated process by name.
   If attachment/symbolication is denied or the process exits first, preserve that
   result and stop. No retry or installation of diagnostic software.
5. Enforce a 60-second outer timeout for the diagnostic process group, terminate
   and reap its observer/sampler children on timeout or launcher failure. Retain
   the existing 8GiB postcompletion RSS ceiling, 128MiB per-output ceiling,
   12GiB total named S05 scratch ceiling, 16GiB free-space precondition and own
   quiet target allocated-byte before/after pair. These are not live RSS/disk
   enforcement. No Cargo/product concurrency.
6. Finish one before/after hash pair for the same two databases, even if the
   diagnostic fails; do not hash other datasets. Preserve every attempted output
   and method failure. Sampling overhead invalidates numerical acceptance.
7. Correlate sample stacks with existing request order, distinguishing setup,
   list/open, row copy/hash, cache admission/decode, payload search and cleanup
   only where symbols/observations discriminate them. Whole-process samples
   may mix phases; report inconclusive attribution explicitly. Produce a finding
   and a concrete next proposal; all six S05 criteria stay OPEN.

Stop after this single diagnostic. Further implementation, retries or performance
campaigns require separate explicit authorization. The exhausted authority is the
human-approved sixth correction pass/one additional campaign, widened to six files;
this proposal changes neither that history nor any acceptance threshold.
