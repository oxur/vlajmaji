# Reference metadata byte comparison: illustrative timing scenarios

Recorded 2026-10-27 (America/Chicago) from the follow-up discussion after the
single sampled diagnostic. This preserves arithmetic and assumptions for the
[planning backlog](../../../../backlog.md); it adds no measurements or authority.

## Inputs and provenance

- Current unsampled failed lookup: **1.206865292 seconds**, `jbotci-local`, from
  the [replacement Full failure disposition](../review-numeric02.md).
- Sampled metadata SHA256 share: **1,207 / 1,752 = 0.6889269406392694**, from
  the [root's retained-output analysis](sampled-diagnostic01/controls/root-sampled-analysis01.json).
- The [independent CDC analysis](sampled-diagnostic01/controls/sampled-diagnostic01-cdc-analysis.json)
  agrees with that aggregation. The 92 observer-side SHA samples are excluded.
- The [findings](sampled-diagnostic01/controls/root-sampled-findings01.md),
  [raw sample](sampled-diagnostic01/output/sample.txt) and
  [preservation manifest](sampled-diagnostic01/manifest.json) retain context and limitations.

## Model, not a forecast

Let `T = 1.206865292 s`, `f = 1207 / 1752`, and `r` be the new byte-comparison
cost divided by the old hashing cost. If that mixed-window sample fraction also
applies to this particular lookup, and all other costs stay unchanged:

`T_new = T * ((1 - f) + f * r)`

| Assumed comparison cost relative to hashing (`r`) | Projected lookup | Speedup |
| --- | ---: | ---: |
| 5% | 417 ms | 2.89x |
| 10% | 459 ms | 2.63x |
| 25% | 583 ms | 2.07x |
| 0%, idealized zero-cost comparison | 375 ms | 3.21x |

The 5%, 10% and 25% replacement costs are illustrative, unmeasured assumptions.
The 417–583 ms range is a set of conditional scenarios, not a confidence interval
or an empirically established prediction. Even the 375 ms idealized value is
conditional on this model, not a general lower bound on an optimized query.

## Interpretation and unresolved questions

These scenarios fall below the unchanged 750 ms maximum, while remaining above
the 250 ms median target. One observed request and this model cannot establish
three-round median acceptance. The sample covered early setup/priming/multiple
requests, not an isolated profile of the final failed measured Local request.
Sampling overhead, phase mixture, unchanged SQLite read/copy costs, memory/cache
pressure and the actual cost of equality comparison limit transfer to future
code. Compact/Readable behavior and other reference identities need their own
approved evidence.

Next planning should address bounded raw-byte storage, freshness/error behavior,
actual unsampled latency and RSS before making a performance claim. All six S05
criteria remain OPEN. No source change, implementation pass, benchmark retry,
additional campaign or push is authorized by this record.
