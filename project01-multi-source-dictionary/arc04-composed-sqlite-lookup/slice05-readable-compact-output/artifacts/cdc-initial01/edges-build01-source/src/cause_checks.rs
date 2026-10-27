// Prospective SOURCE ONLY. Inherited SnapshotError has a source() traversal stop.
// Reach that typed boundary, then match its PUBLIC Store variant explicitly.
fn concrete_store_cause<'a>(mut error: &'a (dyn std::error::Error + 'static))
    -> Option<&'a vlajmaji_store::DictionaryStoreError>
{
    loop {
        if let Some(store) = error.downcast_ref::<vlajmaji_store::DictionaryStoreError>() {
            return Some(store);
        }
        if let Some(vlajmaji_store::DictionarySnapshotError::Store(store)) =
            error.downcast_ref::<vlajmaji_store::DictionarySnapshotError>()
        {
            return Some(store);
        }
        error = error.source()?;
    }
}

fn same_store_boundary(actual: &vlajmaji_store::DictionaryStoreError,
                       direct: &vlajmaji_store::DictionaryStoreError) -> bool {
    use vlajmaji_store::DictionaryStoreError;
    match (actual, direct) {
        (DictionaryStoreError::Io(a), DictionaryStoreError::Io(b)) =>
            a.kind() == b.kind() && a.raw_os_error() == b.raw_os_error(),
        (DictionaryStoreError::InvalidStore { reason: a, .. },
         DictionaryStoreError::InvalidStore { reason: b, .. }) => a == b,
        (DictionaryStoreError::Backend(a), DictionaryStoreError::Backend(b)) =>
            match (a, b) {
                (rusqlite::Error::SqliteFailure(a, _), rusqlite::Error::SqliteFailure(b, _)) => a == b,
                _ => false, // Do not promote an unqualified backend-variant match to proof.
            },
        _ => false,
    }
}
