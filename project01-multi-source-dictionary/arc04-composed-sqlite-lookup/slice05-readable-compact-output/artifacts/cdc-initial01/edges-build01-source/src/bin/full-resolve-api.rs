// Exact frozen public API proof; compile only. No operation is executed.
use vlajmaji_search::{DictionarySearchEngine,DictionaryProjectionProfile,ReferenceResolution,DictionaryQueryError};
use vlajmaji_lexicon::RelationTarget;
fn main() {
    let _: fn(&DictionarySearchEngine, &RelationTarget) -> Result<(ReferenceResolution, DictionaryProjectionProfile), DictionaryQueryError> = DictionarySearchEngine::resolve_profiled;
}
