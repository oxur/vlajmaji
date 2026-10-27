//! Read-only providers for a frozen logical dictionary snapshot.

use std::{
    collections::BTreeSet,
    error::Error,
    fmt,
    path::{Path, PathBuf},
};

use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use vlajmaji_lexicon::{
    DictionaryDocument, EntityId, PackError, UnverifiedDictionaryDocument,
    compose_unverified_documents,
};

use crate::{
    DictionaryStore, DictionaryStoreError,
    dictionary::{check_minimal_metadata, read_canonical_from, read_metadata_from},
    dictionary_schema::DATABASE_FILE,
};

/// Physical provider used for one immutable logical snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DictionaryReadTopology {
    /// One unified derived database.
    Unified,
    /// One SQLite connection with read-only attached pack databases.
    Attached,
    /// One read-only connection per pack database.
    Fanout,
}

/// One sealed member expected by a snapshot manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DictionarySnapshotMember {
    /// Stable pack identity.
    pub id: String,
    /// Native local store directory.
    pub path: PathBuf,
    /// Expected recorded canonical state digest.
    pub expected_state_sha256: String,
}

/// Caller-frozen identity and membership for one query snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DictionarySnapshotManifest {
    /// Manifest schema version.
    pub version: u32,
    /// Caller-selected generation identity.
    pub generation: String,
    /// Expected composed canonical logical state digest.
    pub logical_state_sha256: String,
    /// One unified member or one-to-four per-pack members.
    pub members: Vec<DictionarySnapshotMember>,
    /// Optional runtime selection overlay.
    #[serde(default)]
    pub selected_release_ids: Option<Vec<EntityId>>,
}

/// Failure opening or composing a read snapshot.
#[derive(Debug)]
#[non_exhaustive]
pub enum DictionarySnapshotError {
    /// The complete manifest failed validation before database reads.
    InvalidManifest(String),
    /// ATTACH requires a UTF-8 native path on this URI boundary.
    UnsupportedAttachmentPath(PathBuf),
    /// A member store failed its existing minimum-open checks.
    Store(DictionaryStoreError),
    /// A database read failed.
    Backend(rusqlite::Error),
    /// Filesystem path resolution failed.
    Io(std::io::Error),
    /// Member or logical state differed from the frozen manifest.
    StateMismatch {
        /// Member identity, or `logical-state` for a composed-state mismatch.
        member: String,
        /// Digest frozen in the manifest.
        expected: String,
        /// Digest read or composed by this provider.
        actual: String,
    },
    /// Unverified document composition failed.
    Composition(PackError),
    /// Canonical document serialization failed.
    Canonical(String),
}

impl fmt::Display for DictionarySnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidManifest(detail) => write!(f, "invalid snapshot manifest: {detail}"),
            Self::UnsupportedAttachmentPath(path) => write!(
                f,
                "ATTACH does not support this non-UTF-8 path: {}",
                path.display()
            ),
            Self::Store(error) => write!(f, "snapshot store: {error}"),
            Self::Backend(error) => write!(f, "snapshot SQLite error: {error}"),
            Self::Io(error) => write!(f, "snapshot filesystem error: {error}"),
            Self::StateMismatch {
                member,
                expected,
                actual,
            } => write!(
                f,
                "snapshot state mismatch for {member}: expected {expected}, found {actual}"
            ),
            Self::Composition(error) => write!(f, "snapshot composition: {error}"),
            Self::Canonical(detail) => write!(f, "snapshot canonical document: {detail}"),
        }
    }
}
impl Error for DictionarySnapshotError {}
impl From<DictionaryStoreError> for DictionarySnapshotError {
    fn from(value: DictionaryStoreError) -> Self {
        Self::Store(value)
    }
}
impl From<rusqlite::Error> for DictionarySnapshotError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Backend(value)
    }
}
impl From<std::io::Error> for DictionarySnapshotError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
impl From<PackError> for DictionarySnapshotError {
    fn from(value: PackError) -> Self {
        Self::Composition(value)
    }
}

enum OwnedStores {
    Unified(DictionaryStore),
    Fanout(Vec<DictionaryStore>),
    Attached(Connection),
}

/// A minimally opened physical provider with its own logical document and state.
pub struct DictionaryReadSnapshot {
    topology: DictionaryReadTopology,
    manifest: DictionarySnapshotManifest,
    stores: OwnedStores,
    logical_state_sha256: Option<String>,
    attached_limit: i32,
}

impl fmt::Debug for DictionaryReadSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DictionaryReadSnapshot")
            .field("topology", &self.topology)
            .field("generation", &self.manifest.generation)
            .field("logical_state_sha256", &self.logical_state_sha256)
            .finish_non_exhaustive()
    }
}

impl DictionaryReadSnapshot {
    /// Freeze and open a unified, attached, or fanout snapshot.
    ///
    /// # Errors
    ///
    /// Returns a manifest, path, store, SQLite, state, or composition error.
    pub fn open(
        topology: DictionaryReadTopology,
        mut manifest: DictionarySnapshotManifest,
    ) -> Result<Self, DictionarySnapshotError> {
        validate_manifest(topology, &manifest)?;
        if let Some(selected) = manifest.selected_release_ids.as_mut() {
            selected.sort();
        }
        let (stores, member_states) = match topology {
            DictionaryReadTopology::Unified => {
                let store = DictionaryStore::open(&manifest.members[0].path)?;
                let state = store.recorded_state_sha256().to_owned();
                (OwnedStores::Unified(store), vec![state])
            }
            DictionaryReadTopology::Fanout => {
                let mut stores = Vec::new();
                let mut states = Vec::new();
                for member in &manifest.members {
                    let store = DictionaryStore::open(&member.path)?;
                    states.push(store.recorded_state_sha256().to_owned());
                    stores.push(store);
                }
                (OwnedStores::Fanout(stores), states)
            }
            DictionaryReadTopology::Attached => {
                let first = manifest
                    .members
                    .first()
                    .ok_or_else(|| DictionarySnapshotError::InvalidManifest("no members".into()))?;
                let first_db = first.path.join(DATABASE_FILE);
                let path = first_db.to_str().ok_or_else(|| {
                    DictionarySnapshotError::UnsupportedAttachmentPath(first_db.clone())
                })?;
                let uri = read_only_uri(Path::new(path))?;
                let connection = Connection::open_with_flags(
                    uri,
                    OpenFlags::SQLITE_OPEN_READ_ONLY
                        | OpenFlags::SQLITE_OPEN_URI
                        | OpenFlags::SQLITE_OPEN_NO_MUTEX,
                )?;
                for (index, member) in manifest.members.iter().enumerate().skip(1) {
                    let db = member.path.join(DATABASE_FILE);
                    let path = db.to_str().ok_or_else(|| {
                        DictionarySnapshotError::UnsupportedAttachmentPath(db.clone())
                    })?;
                    let uri = read_only_uri(Path::new(path))?;
                    let schema = match index {
                        1 => "pack1",
                        2 => "pack2",
                        3 => "pack3",
                        _ => unreachable!(),
                    };
                    let sql = format!("ATTACH DATABASE ?1 AS {schema}");
                    connection.execute(&sql, [uri])?;
                }
                connection.execute_batch(
                    "PRAGMA foreign_keys=ON; PRAGMA query_only=ON; BEGIN DEFERRED;",
                )?;
                let foreign_keys: i64 =
                    connection.pragma_query_value(None, "foreign_keys", |row| row.get(0))?;
                if foreign_keys != 1 {
                    return Err(DictionarySnapshotError::Backend(
                        rusqlite::Error::InvalidQuery,
                    ));
                }
                let mut states = Vec::new();
                for index in 0..manifest.members.len() {
                    let schema = if index == 0 {
                        "main".to_owned()
                    } else {
                        format!("pack{index}")
                    };
                    if !connection.is_readonly(schema.as_str())? {
                        return Err(DictionarySnapshotError::InvalidManifest(format!(
                            "schema {schema} is writable"
                        )));
                    }
                    let metadata = read_metadata_from(&connection, &schema)?;
                    check_minimal_metadata(&metadata)?;
                    let state = metadata
                        .get("canonical_sha256")
                        .cloned()
                        .unwrap_or_default();
                    states.push(state);
                }
                (OwnedStores::Attached(connection), states)
            }
        };
        for (member, actual) in manifest.members.iter().zip(&member_states) {
            if member.expected_state_sha256 != *actual {
                return Err(DictionarySnapshotError::StateMismatch {
                    member: member.id.clone(),
                    expected: member.expected_state_sha256.clone(),
                    actual: actual.clone(),
                });
            }
        }
        let logical_state_sha256 = if topology == DictionaryReadTopology::Unified {
            Some(member_states[0].clone())
        } else {
            None
        };
        if let Some(actual) = &logical_state_sha256
            && manifest.logical_state_sha256 != *actual
        {
            return Err(DictionarySnapshotError::StateMismatch {
                member: "logical-state".into(),
                expected: manifest.logical_state_sha256.clone(),
                actual: actual.clone(),
            });
        }
        let attached_limit = match &stores {
            OwnedStores::Unified(store) => store.runtime_attached_limit()?,
            OwnedStores::Fanout(stores) => stores
                .first()
                .ok_or_else(|| {
                    DictionarySnapshotError::InvalidManifest("fanout has no stores".into())
                })?
                .runtime_attached_limit()?,
            OwnedStores::Attached(connection) => {
                connection.limit(rusqlite::limits::Limit::SQLITE_LIMIT_ATTACHED)?
            }
        };
        Ok(Self {
            topology,
            manifest,
            stores,
            logical_state_sha256,
            attached_limit,
        })
    }
    /// Load the provider's own typed document, composing A/F members and checking
    /// their canonical logical digest without asserting full-model validity.
    ///
    /// # Errors
    ///
    /// Returns canonical parsing, composition, backend, or logical-state errors.
    pub fn load_document(
        &mut self,
    ) -> Result<UnverifiedDictionaryDocument, DictionarySnapshotError> {
        if self.topology == DictionaryReadTopology::Unified {
            let OwnedStores::Unified(store) = &self.stores else {
                unreachable!()
            };
            let document = store
                .unverified_document()
                .map_err(DictionarySnapshotError::Store)?;
            self.check_selection(&document)?;
            return Ok(document);
        }
        let mut documents = Vec::new();
        match &self.stores {
            OwnedStores::Fanout(stores) => {
                for store in stores {
                    let (document, has_legacy_archive) = store
                        .unverified_document_with_legacy_archive()
                        .map_err(DictionarySnapshotError::Store)?;
                    if has_legacy_archive {
                        return Err(DictionarySnapshotError::Composition(
                            PackError::LegacyArchiveUnsupported,
                        ));
                    }
                    documents.push(document.ok_or_else(|| {
                        DictionarySnapshotError::Canonical(
                            "canonical draft is absent without a legacy marker".into(),
                        )
                    })?);
                }
            }
            OwnedStores::Attached(connection) => {
                for index in 0..self.manifest.members.len() {
                    let schema = if index == 0 {
                        "main".to_owned()
                    } else {
                        format!("pack{index}")
                    };
                    let (bytes, legacy) = read_canonical_from(connection, &schema)?;
                    if legacy.is_some() {
                        return Err(DictionarySnapshotError::Composition(
                            PackError::LegacyArchiveUnsupported,
                        ));
                    }
                    documents.push(
                        DictionaryDocument::from_unverified_json_bytes(&bytes)
                            .map_err(|e| DictionarySnapshotError::Canonical(e.to_string()))?,
                    );
                }
            }
            OwnedStores::Unified(_) => unreachable!(),
        }
        let document = compose_unverified_documents(documents)?;
        let bytes = serde_json::to_vec(&document)
            .map_err(|e| DictionarySnapshotError::Canonical(e.to_string()))?;
        let actual = format!("{:x}", Sha256::digest(bytes));
        if actual != self.manifest.logical_state_sha256 {
            return Err(DictionarySnapshotError::StateMismatch {
                member: "logical-state".into(),
                expected: self.manifest.logical_state_sha256.clone(),
                actual,
            });
        }
        self.check_selection(&document)?;
        self.logical_state_sha256 = Some(self.manifest.logical_state_sha256.clone());
        Ok(document)
    }
    fn check_selection(
        &self,
        document: &UnverifiedDictionaryDocument,
    ) -> Result<(), DictionarySnapshotError> {
        if let Some(selected) = &self.manifest.selected_release_ids {
            let known: BTreeSet<_> = document
                .metadata
                .releases
                .iter()
                .map(|row| &row.id)
                .collect();
            if let Some(unknown) = selected.iter().find(|id| !known.contains(*id)) {
                return Err(DictionarySnapshotError::InvalidManifest(format!(
                    "unknown selected release ID {unknown}"
                )));
            }
        }
        Ok(())
    }
    /// Return the frozen generation identifier.
    pub fn generation(&self) -> &str {
        &self.manifest.generation
    }
    /// Return the composed logical state digest.
    pub fn logical_state_sha256(&self) -> &str {
        self.logical_state_sha256.as_deref().unwrap_or("")
    }
    /// Return SQLite's actual read-only `SQLITE_LIMIT_ATTACHED` for this provider.
    pub fn runtime_attached_limit(&self) -> i32 {
        self.attached_limit
    }
    /// Return the immutable runtime release selection overlay.
    pub fn selected_release_ids(&self) -> Option<&[EntityId]> {
        self.manifest.selected_release_ids.as_deref()
    }
    /// Query exact eligible assertion IDs across this provider, deduplicated by logical ID.
    pub fn exact_assertion_ids(
        &self,
        spelling: &str,
    ) -> Result<Vec<EntityId>, DictionarySnapshotError> {
        let mut ids = BTreeSet::new();
        match &self.stores {
            OwnedStores::Unified(store) => ids.extend(
                store
                    .exact_assertion_ids(spelling)
                    .map_err(DictionarySnapshotError::Store)?,
            ),
            OwnedStores::Fanout(stores) => {
                for store in stores {
                    ids.extend(
                        store
                            .exact_assertion_ids(spelling)
                            .map_err(DictionarySnapshotError::Store)?,
                    );
                }
            }
            OwnedStores::Attached(connection) => {
                for index in 0..self.manifest.members.len() {
                    let schema = if index == 0 {
                        "main".to_owned()
                    } else {
                        format!("pack{index}")
                    };
                    let sql = format!(
                        "SELECT a.id FROM {schema}.entries AS e JOIN {schema}.assertions AS a ON a.entry_id=e.id WHERE e.spelling_state='value' AND e.spelling_value=?1 COLLATE BINARY AND a.eligibility='Lexical' ORDER BY e.source_id COLLATE BINARY,e.spelling_value COLLATE BINARY,a.occurrence_id COLLATE BINARY"
                    );
                    let mut stmt = connection.prepare(&sql)?;
                    let rows = stmt.query_map([spelling], |row| row.get::<_, String>(0))?;
                    for row in rows {
                        ids.insert(
                            EntityId::new(row?)
                                .map_err(|e| DictionarySnapshotError::Canonical(e.to_string()))?,
                        );
                    }
                }
            }
        }
        Ok(ids.into_iter().collect())
    }
}

fn validate_manifest(
    topology: DictionaryReadTopology,
    manifest: &DictionarySnapshotManifest,
) -> Result<(), DictionarySnapshotError> {
    let valid_digest = |s: &str| {
        s.len() == 64
            && s.bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    };
    if manifest.version != 1
        || manifest.generation.is_empty()
        || manifest.generation.chars().any(char::is_control)
        || !valid_digest(&manifest.logical_state_sha256)
    {
        return Err(DictionarySnapshotError::InvalidManifest(
            "unsupported version, invalid generation, or logical digest".into(),
        ));
    }
    let n = manifest.members.len();
    if n == 0 || n > 4 || (topology == DictionaryReadTopology::Unified && n != 1) {
        return Err(DictionarySnapshotError::InvalidManifest(
            "topology requires one unified or one-to-four pack members".into(),
        ));
    }
    let mut ids = BTreeSet::new();
    for member in &manifest.members {
        if member.id.is_empty()
            || member.id.chars().any(char::is_control)
            || !ids.insert(&member.id)
            || !valid_digest(&member.expected_state_sha256)
        {
            return Err(DictionarySnapshotError::InvalidManifest(
                "invalid or duplicate member ID/digest".into(),
            ));
        }
    }
    if let Some(selected) = &manifest.selected_release_ids {
        let set: BTreeSet<_> = selected.iter().collect();
        if set.len() != selected.len() {
            return Err(DictionarySnapshotError::InvalidManifest(
                "duplicate selected release IDs".into(),
            ));
        }
    }
    Ok(())
}
fn read_only_uri(path: &Path) -> Result<String, DictionarySnapshotError> {
    let canonical = path.canonicalize()?;
    let text = canonical
        .to_str()
        .ok_or_else(|| DictionarySnapshotError::UnsupportedAttachmentPath(canonical.clone()))?;
    let mut uri = String::from("file:");
    for byte in text.as_bytes() {
        if byte.is_ascii_alphanumeric() || b"/-._~:".contains(byte) {
            uri.push(*byte as char);
        } else {
            use std::fmt::Write as _;
            let _ = write!(uri, "%{byte:02X}");
        }
    }
    uri.push_str("?mode=ro");
    Ok(uri)
}
