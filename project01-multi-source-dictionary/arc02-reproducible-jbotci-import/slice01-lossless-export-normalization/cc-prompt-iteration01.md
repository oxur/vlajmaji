# CC assignment — A02 S01 iteration01: explicit provenance identity oracle

This is the current preserved assignment for Code Contributor
`/root/vlajmaji_p01_a02_cdc/vlajmaji_a02_s01_cc`, OpenAI gpt-6-luna/high as
Duncan explicitly selected. Same Autonomous/Expedited Arc02 scope and CDC
reviewer. It supersedes the unstarted initial assignment `cc-prompt.md` as the
active handoff, carrying every original implementation obligation. No product
candidate or review failure exists yet. Executed refinements:0of5; this is a
pre-edit prompt clarification. Preserve both paths and all intake/failure evidence.

Governing source remains clean d1e1c928f7999f2220b046cc5a5817f0529312ca in the
same isolated source worktree/branch. Planning candidate is the commit containing
this prompt/current-assignment update, following84c1694. All7ledger rows remain
OPEN. Source/index ownership, exact allowlist, sole planning write
artifacts/cc-execution.md, target/offline/login/resource guard, no external action,
scoped source commits, return/freeze and all mandatory gates from the initial
prompt remain unchanged. No new source/package/dependency path is authorized.

## Required reading and coherent intake

Required-full: this iteration, preserved initial cc-prompt.md, current
slice-plan.md and ledger.md. Complete the initial prompt's remaining manifest
and data queries; retain actual completed extents rather than pretending to
restart or inheriting another contributor's reading. Its full source/raw/metadata/
guide contract still applies. Recover truncated bodies in bounded contiguous
reads, e.g.150–250lines percall with output sized accordingly; an exit0 does not
prove full receipt. The currently issued prompt must match slice-plan.md.
Append the source-cited identity readback and any missing extents to the same
cc-execution.md intake record before dependent edits. Then proceed with the
entire original implementation assignment without another approval gate.

## Binding identity decision and rationale

Root highlighted an actual interaction: the same raw and metadata bytes may be
vendored at two different Git revisions when supplier code changes. A normalized
release records that distributor revision, so its ID must distinguish those
provenance bodies. Current committed slice-plan.md line129 already specifies
releaseID revision+fullraw+metadata digest; this prompt makes the discriminating
oracle explicit. It does not replace raw identity or grant a new attestation.

- Release identity includes the exact40hex distributor revision plus both full
 64hex input digests (or an unambiguous domain-separated digest of that tuple).
- Raw capture identities remain the established exact namespace+contentSHA tuple.
- Entry/assertion IDs remain full raw digest+rowindex+role as already specified.
 Native ID, spelling/text equality and distributor copies never deduplicate rows
 or multiply source attestations. Release history distinguishes distribution
 provenance while immutable source occurrences remain addressable.
- For the same options/inputs, deterministic release ID and canonical bytes are
 stable. Different revision with byte-identical raw/metadata produces a distinct
 release ID/body and report.revision, while input captures' identities, every
 occurrence locator/legacy tuple encoding, normalized entity IDs, raw/native
 fields, attribution identities and accounting remain identical. Capture
 metadata.revision may truthfully record the new distributor revision; do not
 require complete capture metadata equality across differing provenance.
- A deliberate source-byte change with correctly updated input pins is a new
 snapshot, so occurrence/row-derived IDs must change. A wrong expected digest
 remains HashMismatch, never accepted as a provenance-only update.

No native word-entry/sense identity or assertion timestamps are invented. This
policy is inside the approved separate source/snapshot/release contract; it is
not a new physical corpus topology or a change to old archive identity.

## Concrete valid-counterpart test

Use the same valid synthetic raw bytes and full valid metadata used by normal
import positives. Set expected raw/meta hashes correctly. Import with valid
revision `0000000000000000000000000000000000000000`, repeat it unchanged, then
with valid revision `1111111111111111111111111111111111111111`.

Assert first versus repeated import has equal complete Dictionary/canonical bytes,
report and release ID. Assert first versus second revision has unequal release ID,
canonical bytes and report.revision; corresponding ReleaseRecord.distributor_revision
is each exact supplied revision. Compare capture.identity vectors (not their
metadata.revision), assertion/entry IDs, OccurrenceRef values, all retained raw
bytes and ordered fields, registry occurrence identities, release accounting and
complete per-row normalized values: these remain equal. Count exactly the same
raw/normalized/eligible/excluded populations in both; no second independent
source or doubled assertions is created. Exercise this through the public API;
CLI revision overrides should demonstrate the same release distinction and check
rejects a release regenerated under different revision as jbotci_release_mismatch.
A changed raw-byte positive with recomputed matching metadata/options establishes
new capture and row IDs, complementing the existing hash-mismatch negative.

A release ID based only on raw+metadata, a constant release ID, revision-based raw
occurrence IDs, or a doubled-attestation interpretation must fail these controls.
This test is small enough to use the existing integration tests and outside caller;
no new verifier product or dependency is requested.

## Return and readiness

Execute the full original S01 API/CLI/docs/fullpopulation/gate assignment and
this explicit identity control. Return identifies iteration01, baseline/current
source commit, exact ownership/freeze and all7row proposals; include whether
identity controls ran through public API and actual CLI. CC remains proposed-done
until CDC independently reviews source and reproduces evidence. All failure and
truncation chronology remains, no invented receipts. S02 stays held on acceptance.

CDC author readiness: current model CaptureRef and OccurrenceRef are exact byte
snapshot tuples; ReleaseRecord separately carries distributor_revision. The
existing implementation recipe already makes release identity provenance-aware;
this explicit three-call positive and changed-input counterpart distinguish the
interaction without redesigning the raw model or weakening gates. No source
mutation had begun at this handoff; CC acknowledged hold. Ordinary private helper
mechanics remain local discretion; the observable identity policy is binding.
