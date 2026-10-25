# Reviewer setup observations

The actual unique consumer build/full import/store/query, own reference, reviewed SQL oracle, actual product CLI build/query, five pipeline process controls and full offline Make check all completed at their first execution; no product failure was observed.

One later read-only retained-control table extraction failed before any product execution: it assumed every JSON `cases` member was a list, but pipeline03 uses integer `cases:47` and list `attempts`. Python exited1 with `TypeError: 'int' object is not iterable`. The corrected inspection used `attempts` for pipeline03 and `cases` for alias/boundary/failure-path records, then covered all60 exact retained cases. No product run was retried or relabelled because of this reviewer error.

Some broad initial read-tool outputs were truncated; required bodies were recovered with bounded reads before dependent execution, as the intake records. An attempted exact registry-type search used escaped alternation and returned no matches; corrected extended-regexp lookup supplied the full required CatalogTarget/LocalRef/RelationTarget fields before caller construction. These were read-only reviewer setup observations, not product failures.

No source/build cache/old binary/store was deleted. The full canonical release, fresh full store, complete public/CLI response values and synthetic transport fixture inputs remain local; commit-ready receipts retain hashes and code only. The initial local full process inventory remains local; the commit-ready preflight records its hash and bounded relevant process result. Source-derived witness definition text is replaced in the commit-ready census by exact length/hash; local original remains intact.
