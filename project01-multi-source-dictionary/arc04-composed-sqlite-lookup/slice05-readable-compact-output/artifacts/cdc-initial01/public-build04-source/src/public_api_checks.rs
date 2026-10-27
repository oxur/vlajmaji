// Prospective SOURCE ONLY. Function pointers check the exact issued public
// signatures at compilation; they do NOT execute an operation or prove behavior.
fn public_api_checks() {
    use vlajmaji_store::{CollectionProfiles, DictionaryReadTopology, DictionarySnapshotManifest};
    let _: fn(&[u8]) -> Result<DictionaryHitDetailRef, DictionaryProjectionError> = DictionaryHitDetailRef::from_json_bytes;
    let _: fn(&DictionaryHitDetailRef) -> Result<(), DictionaryProjectionError> = DictionaryHitDetailRef::validate;
    let _: fn(&str) -> Result<(), DictionaryProjectionError> = validate_detail_snapshot_sha256;
    let _: fn(&Path) -> Result<DictionaryCompactSearchEngine, DictionaryQueryError> = |p| DictionaryCompactSearchEngine::open(p);
    let _: fn(DictionaryReadTopology, DictionarySnapshotManifest) -> Result<DictionaryCompactSearchEngine, DictionaryQueryError> = DictionaryCompactSearchEngine::open_snapshot;
    let _: fn(DictionaryReadTopology, DictionarySnapshotManifest) -> Result<(DictionaryCompactSearchEngine, DictionarySnapshotOpenProfile), DictionaryQueryError> = DictionaryCompactSearchEngine::open_snapshot_profiled;
    let _: fn(&DictionaryCompactSearchEngine) -> &str = DictionaryCompactSearchEngine::state_sha256;
    let _: fn(&DictionaryCompactSearchEngine) -> &str = DictionaryCompactSearchEngine::snapshot_sha256;
    let _: fn(&DictionaryCompactSearchEngine, &str, DictionaryExactOptions) -> Result<DictionaryCompactExactResponse, DictionaryQueryError> = DictionaryCompactSearchEngine::exact;
    let _: fn(&DictionaryCompactSearchEngine, &RelationTarget) -> Result<DictionaryCompactReferenceResolution, DictionaryQueryError> = DictionaryCompactSearchEngine::resolve;
    let _: fn(&DictionaryCompactSearchEngine, DictionaryRequest) -> Result<DictionaryCompactQueryResult, DictionaryQueryError> = DictionaryCompactSearchEngine::execute;
    let _: fn(&DictionaryCompactSearchEngine, DictionaryRequest) -> Result<(DictionaryCompactQueryResult, DictionaryProjectionProfile), DictionaryQueryError> = DictionaryCompactSearchEngine::execute_profiled;
    let _: fn(&DictionaryCompactSearchEngine, &DictionaryHitDetailRef, Option<&str>) -> Result<DictionaryHitDetailResponse, DictionaryProjectionError> = DictionaryCompactSearchEngine::detail;
    let _: fn(&Path, CollectionQuerySelection) -> Result<CollectionCompactSearchEngine, CollectionQueryError> = |p,s| CollectionCompactSearchEngine::open(p,s);
    let _: fn(&CollectionProfiles, Option<&str>, Option<&[PackId]>) -> Result<CollectionCompactSearchEngine, CollectionQueryError> = CollectionCompactSearchEngine::open_configured;
    let _: fn(&CollectionProfiles, Option<&str>, Option<&[PackId]>) -> Result<(CollectionCompactSearchEngine, std::time::Duration), CollectionQueryError> = CollectionCompactSearchEngine::open_configured_profiled;
    let _: fn(&CollectionCompactSearchEngine) -> &CollectionSelectionSnapshot = CollectionCompactSearchEngine::selection;
    let _: fn(&mut CollectionCompactSearchEngine, DictionaryRequest) -> Result<CollectionCompactQueryResponse, CollectionQueryError> = CollectionCompactSearchEngine::execute;
    let _: fn(&mut CollectionCompactSearchEngine, DictionaryRequest) -> Result<(CollectionCompactQueryResponse, CollectionProjectionProfile), CollectionQueryError> = CollectionCompactSearchEngine::execute_profiled;
    let _: fn(&mut CollectionCompactSearchEngine, &DictionaryHitDetailRef, Option<&str>) -> Result<CollectionHitDetailResponse, CollectionQueryError> = CollectionCompactSearchEngine::detail;
    // DIAGNOSTIC ONLY: missing frozen full resolve_profiled method remains a
    // failing original API proof in public-build01-source; restore this pointer
    // after an actual scoped repair before any INITIAL acceptance.
    let _: fn(&CollectionProfiles, Option<&str>, Option<&[PackId]>) -> Result<(CollectionSearchEngine, std::time::Duration), CollectionQueryError> = CollectionSearchEngine::open_configured_profiled;
    let _: fn(&mut CollectionSearchEngine, DictionaryRequest) -> Result<(CollectionQueryResponse, CollectionProjectionProfile), CollectionQueryError> = CollectionSearchEngine::execute_profiled;
    let _: fn(&CollectionCatalogue, PackId, &str, &RelationTarget) -> Result<vlajmaji_store::CollectionReferenceProjection, vlajmaji_store::CollectionError> = CollectionCatalogue::reference_projection;
    let _: fn(&ReleaseRecord) -> vlajmaji_store::CollectionReleaseSummary = |release| vlajmaji_store::CollectionReleaseSummary::from(release);
    println!("PASS compiled remaining issued public method signatures (missing full resolve_profiled proof remains FAIL; no operations executed by signatures)");
}
