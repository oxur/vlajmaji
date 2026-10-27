# vlajmaji-data-llg

This unpublished, optional dependency exposes compile-time embedded gismu,
cmavo, and assigned rafsi lists normalized into Vlajmaji's versioned dictionary
model. A clean checkout has no payload. Prepare ignored local data with
`vlajmaji data-pack fixture llg crates/data-llg/data` for synthetic builds or
`vlajmaji data-pack materialize llg GISMU CMAVO RAFSI NOTICE crates/data-llg/data`
for a locally pinned full release. Run these commands from the workspace root;
the output path is where this crate's `include_bytes!` payload is read at build
time. Preparation reads only the input paths supplied by the caller and does
not create a dictionary store.

Enable the `embedded` feature before using `release_bytes`, `seal_bytes`, or
`load`. The default feature set has no payload and requires no corpus files.
The seal is a local integrity inventory, not publisher authentication or
redistribution permission. The pinned source notice remains separate from the
Apache-2.0 code license.
