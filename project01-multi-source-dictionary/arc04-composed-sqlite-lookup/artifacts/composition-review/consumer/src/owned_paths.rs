// Prospective method guard: reject receipt path escape BEFORE any operation on it.
fn assert_owned_store_path(root: &Path, store: &Path) {
    use std::path::Component;
    assert!(root.is_absolute() && store.is_absolute());
    let relative = store.strip_prefix(root).expect("returned store must remain in the NEW owned ROOT");
    assert!(relative.components().all(|part| matches!(part, Component::Normal(_))));
    assert!(relative.components().next().is_some());
    assert!(std::fs::symlink_metadata(root).unwrap().file_type().is_dir());
    let mut path = root.to_path_buf();
    for component in relative.components() {
        path.push(component.as_os_str());
        let metadata = std::fs::symlink_metadata(&path).unwrap();
        assert!(!metadata.file_type().is_symlink(), "no symlink in returned owned store path");
        assert!(metadata.is_dir());
    }
}
