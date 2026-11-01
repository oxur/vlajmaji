//! Authoritative, append-only local journals for explicitly owned dictionaries.
use std::{
    error::Error,
    fmt, fs, io,
    path::{Path, PathBuf},
};

use rusqlite::{Connection, OpenFlags, params};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use vlajmaji_lexicon::{
    Dictionary, DictionaryDocument, DictionaryError, DictionaryMetadata, EntityId, OwnedContextPin,
    OwnedCurationData, OwnedCurationDocument, OwnedCurationError, OwnedCurationRecord,
    OwnedCurationRevision, OwnedHead, OwnedRevisionRecord, OwnedSectionPrefix, OwnedSnapshot,
    OwnedSnapshotCommit, OwnedSnapshotCuration, OwnedSnapshotCurationPrefix, OwnedSnapshotData,
    OwnedSnapshotError, RegistryDocument, SourceId, SourceOrigin, SourceRegistry,
};

use crate::owned_schema::{self, SECTIONS};

const FAMILY: &str = "vlajmaji-owned";

/// A stable summary of one committed revision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnedCommitSummary {
    /// Zero-based append sequence.
    pub sequence: u64,
    /// The complete typed revision record.
    pub revision: OwnedRevisionRecord,
    /// SHA-256 of the complete canonical dictionary state.
    pub state_sha256: String,
}

/// The compact persisted identity of one owned revision.
///
/// This value intentionally omits active and tombstone rosters. Those are
/// checked while reading the typed revision summary, but are not copied into
/// identity bindings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnedRevisionIdentity {
    /// Persisted owned-journal schema version.
    pub journal_schema: u32,
    /// The source that owns this revision.
    pub source: SourceId,
    /// The selected revision identifier.
    pub revision: EntityId,
    /// Zero-based append sequence.
    pub sequence: u64,
    /// SHA-256 of the complete canonical dictionary state.
    pub state_sha256: String,
    /// SHA-256 of the selected curation prefix; absent only for schema one.
    #[serde(deserialize_with = "deserialize_required_option")]
    pub curation_state_sha256: Option<String>,
}

fn deserialize_required_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: DeserializeOwned,
{
    Option::<T>::deserialize(deserializer)
}

/// Current owned source and its persisted head.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnedHeadReport {
    /// Report schema version.
    pub version: u32,
    /// The sole owned source.
    pub source: SourceId,
    /// Current committed head.
    pub head: OwnedCommitSummary,
}

/// Typed committed history, genesis first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnedHistoryReport {
    /// Report schema version.
    pub version: u32,
    /// The sole owned source.
    pub source: SourceId,
    /// Current revision identifier.
    pub head: EntityId,
    /// Ordered commit summaries from genesis through the current head.
    pub commits: Vec<OwnedCommitSummary>,
}

/// Whether an explicit snapshot import created a journal or found an exact replay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnedImportOutcome {
    /// A new journal was created from the complete imported history.
    Imported,
    /// The existing journal already contained exactly this complete history.
    AlreadyPresent,
}

/// Result of importing or exactly replaying a sealed owned snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnedImportReport {
    /// Report schema version.
    pub version: u32,
    /// Whether this call imported a new journal or found an exact replay.
    pub outcome: OwnedImportOutcome,
    /// The sole owned source.
    pub source: SourceId,
    /// Final committed head.
    pub head: EntityId,
    /// Original journal schema version.
    pub journal_schema: u32,
    /// Number of retained commits, including genesis.
    pub revision_count: u64,
    /// SHA-256 of the canonical typed snapshot payload.
    pub snapshot_sha256: String,
}

/// A complete validated historical dictionary state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnedRevisionView {
    /// Report schema version.
    pub version: u32,
    /// The sole owned source.
    pub source: SourceId,
    /// Requested commit metadata.
    pub commit: OwnedCommitSummary,
    /// Fully reconstructed and validated dictionary.
    pub dictionary: Dictionary,
}

/// The journal operation represented by a successful write.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnedWriteOperation {
    /// Created genesis revision zero.
    Init,
    /// Appended one revision.
    Edit,
}

/// Result of creating or appending to an owned journal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnedWriteReport {
    /// Report schema version.
    pub version: u32,
    /// The committed operation.
    pub operation: OwnedWriteOperation,
    /// The sole owned source.
    pub source: SourceId,
    /// Previous head, absent only at genesis.
    pub previous_head: Option<EntityId>,
    /// Newly committed head.
    pub head: OwnedCommitSummary,
}

/// Result of an explicit schema-one to schema-two curation migration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnedMigrationOutcome {
    /// Curation sidecar was created for every committed base revision.
    Migrated,
    /// The journal already had a valid schema-two header.
    AlreadyCurrent,
}

/// Summary returned by an explicit curation migration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnedCurationMigrationReport {
    /// Report schema version.
    pub version: u32,
    /// The sole owned source.
    pub source: SourceId,
    /// Current committed head.
    pub head: EntityId,
    /// Journal schema before the operation.
    pub from_schema: u32,
    /// Journal schema after the operation.
    pub to_schema: u32,
    /// Whether migration occurred or was already complete.
    pub outcome: OwnedMigrationOutcome,
}

/// Stable categories for owned journal failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum OwnedJournalErrorCategory {
    /// Invalid caller policy or request shape.
    InvalidRequest,
    /// Invalid typed dictionary model.
    InvalidDictionary,
    /// Unsupported journal version.
    UnsupportedVersion,
    /// Another SQLite family occupies the path.
    WrongFamily,
    /// The expected head is no longer current.
    Conflict,
    /// An existing row was changed or removed.
    ImmutableRecord,
    /// Requested revision does not exist.
    NotFound,
    /// Persisted bytes violate integrity checks.
    Integrity,
    /// SQLite could not acquire or complete a transaction due to locking.
    Busy,
    /// Native filesystem failure.
    Io,
    /// SQLite backend failure.
    Backend,
    /// A curation operation requires explicit schema-two migration.
    MigrationRequired,
    /// Curation validation or persisted curation integrity failed.
    InvalidCuration,
    /// Incoming snapshot validation failed.
    InvalidSnapshot,
    /// An existing journal contains a valid but different complete history.
    ImportConflict,
    /// Commit outcome could not be established safely.
    CommitOutcomeUnknown,
}

/// A typed failure from opening, validating, reading, or writing an owned journal.
#[derive(Debug)]
#[non_exhaustive]
pub enum OwnedJournalError {
    /// The caller request violates the owned-journal policy.
    InvalidRequest(String),
    /// The complete dictionary failed model validation.
    InvalidDictionary(DictionaryError),
    /// A journal or model version is unsupported.
    UnsupportedVersion {
        /// Journal schema version found in the fixed header.
        found: u32,
    },
    /// The path belongs to another database family.
    WrongFamily,
    /// Persisted head differs from the expected head.
    Conflict {
        /// Head supplied by the caller.
        expected: EntityId,
        /// Head persisted when the writer acquired its transaction.
        actual: EntityId,
    },
    /// A previously committed typed row was altered, removed, or reordered.
    ImmutableRecord {
        /// Versioned typed-vector section containing the changed row.
        section: String,
        /// Zero-based row position that differs from the committed prefix.
        ordinal: u64,
    },
    /// The requested revision is not present.
    NotFound(EntityId),
    /// Persisted journal state is structurally or cryptographically invalid.
    Integrity(String),
    /// SQLite reported a busy/locked transaction.
    Busy(String),
    /// Filesystem operation failed.
    Io(io::Error),
    /// SQLite operation failed.
    Backend(rusqlite::Error),
    /// The journal must be explicitly migrated before curation operations.
    MigrationRequired,
    /// Curation document validation or persisted curation integrity failed.
    InvalidCuration(OwnedCurationError),
    /// The incoming snapshot failed pure validation.
    InvalidSnapshot(OwnedSnapshotError),
    /// An existing journal contains a valid but different complete history.
    ImportConflict(String),
    /// Commit and rollback observations cannot establish whether a commit occurred.
    CommitOutcomeUnknown(String),
}

impl OwnedJournalError {
    /// Returns the stable machine-readable failure category.
    #[must_use]
    pub fn category(&self) -> OwnedJournalErrorCategory {
        match self {
            Self::InvalidRequest(_) => OwnedJournalErrorCategory::InvalidRequest,
            Self::InvalidDictionary(_) => OwnedJournalErrorCategory::InvalidDictionary,
            Self::UnsupportedVersion { .. } => OwnedJournalErrorCategory::UnsupportedVersion,
            Self::WrongFamily => OwnedJournalErrorCategory::WrongFamily,
            Self::Conflict { .. } => OwnedJournalErrorCategory::Conflict,
            Self::ImmutableRecord { .. } => OwnedJournalErrorCategory::ImmutableRecord,
            Self::NotFound(_) => OwnedJournalErrorCategory::NotFound,
            Self::Integrity(_) => OwnedJournalErrorCategory::Integrity,
            Self::Busy(_) => OwnedJournalErrorCategory::Busy,
            Self::Io(_) => OwnedJournalErrorCategory::Io,
            Self::Backend(_) => OwnedJournalErrorCategory::Backend,
            Self::MigrationRequired => OwnedJournalErrorCategory::MigrationRequired,
            Self::InvalidCuration(_) => OwnedJournalErrorCategory::InvalidCuration,
            Self::InvalidSnapshot(_) => OwnedJournalErrorCategory::InvalidSnapshot,
            Self::ImportConflict(_) => OwnedJournalErrorCategory::ImportConflict,
            Self::CommitOutcomeUnknown(_) => OwnedJournalErrorCategory::CommitOutcomeUnknown,
        }
    }
}

impl fmt::Display for OwnedJournalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(detail) => write!(f, "invalid owned journal request: {detail}"),
            Self::InvalidDictionary(error) => write!(f, "invalid owned dictionary: {error}"),
            Self::UnsupportedVersion { found } => {
                write!(f, "unsupported owned journal version {found}")
            }
            Self::WrongFamily => f.write_str("database is not an owned dictionary journal"),
            Self::Conflict { expected, actual } => write!(
                f,
                "owned journal head conflict: expected {expected}, found {actual}"
            ),
            Self::ImmutableRecord { section, ordinal } => {
                write!(f, "immutable owned record changed at {section}[{ordinal}]")
            }
            Self::NotFound(revision) => write!(f, "owned revision not found: {revision}"),
            Self::Integrity(detail) => write!(f, "owned journal integrity failure: {detail}"),
            Self::Busy(detail) => write!(f, "owned journal is busy: {detail}"),
            Self::Io(error) => write!(f, "owned journal filesystem error: {error}"),
            Self::Backend(error) => write!(f, "owned journal backend error: {error}"),
            Self::MigrationRequired => f.write_str("owned journal requires curation migration"),
            Self::InvalidCuration(error) => write!(f, "invalid owned curation: {error}"),
            Self::InvalidSnapshot(error) => write!(f, "invalid owned snapshot: {error}"),
            Self::ImportConflict(detail) => write!(f, "owned snapshot import conflict: {detail}"),
            Self::CommitOutcomeUnknown(detail) => {
                write!(f, "owned journal commit outcome is unknown: {detail}")
            }
        }
    }
}
impl Error for OwnedJournalError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidDictionary(error) => Some(error),
            Self::Io(error) => Some(error),
            Self::Backend(error) => Some(error),
            Self::InvalidCuration(error) => Some(error),
            Self::InvalidSnapshot(error) => Some(error),
            _ => None,
        }
    }
}
impl From<io::Error> for OwnedJournalError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}
impl From<rusqlite::Error> for OwnedJournalError {
    fn from(value: rusqlite::Error) -> Self {
        map_busy(value)
    }
}

/// A writable handle to one explicit owned-dictionary journal.
pub struct OwnedJournal {
    connection: Connection,
    poisoned: bool,
}

/// A read-only pinned snapshot of one owned-dictionary journal.
pub struct OwnedJournalReader {
    connection: Connection,
}

impl OwnedJournal {
    /// Validates and creates a genesis journal under an existing parent directory.
    ///
    /// The root is created exclusively and is never replaced.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid genesis policy/model, an occupied root, or a
    /// filesystem/SQLite failure. A failed partial root is retained for diagnosis.
    pub fn create(
        root: &Path,
        genesis: &Dictionary,
    ) -> Result<(Self, OwnedWriteReport), OwnedJournalError> {
        validate_genesis(genesis)?;
        match fs::symlink_metadata(root) {
            Ok(_) => {
                return Err(OwnedJournalError::Io(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    "owned journal root already exists",
                )));
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        let parent = root
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        if !parent.is_dir() {
            return Err(OwnedJournalError::Io(io::Error::new(
                io::ErrorKind::NotFound,
                "owned journal parent must exist",
            )));
        }
        fs::create_dir(root)?;
        let path = root.join(owned_schema::FILE);
        let connection = Connection::open_with_flags(
            &path,
            OpenFlags::SQLITE_OPEN_READ_WRITE
                | OpenFlags::SQLITE_OPEN_CREATE
                | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        connection.busy_timeout(std::time::Duration::ZERO)?;
        owned_schema::configure(&connection)?;
        connection.execute_batch("BEGIN IMMEDIATE")?;
        let journal = Self {
            connection,
            poisoned: false,
        };
        let transaction = RollbackGuard::new(&journal.connection, None);
        let source = genesis.registry().as_document().sources[0].id.clone();
        let prepared = (|| {
            owned_schema::initialize(&journal.connection)?;
            let state = sections(genesis)?;
            let revision = genesis.metadata().owned_revisions[0].clone();
            let hash = state_hash(genesis)?;
            insert_initial(&journal.connection, &state, &revision, &hash, &source)?;
            Ok::<_, OwnedJournalError>((revision, hash))
        })();
        let (revision, state_hash) = match prepared {
            Ok(value) => value,
            Err(error) => {
                let result = rollback_error(&journal.connection, error);
                transaction.disarm();
                return result;
            }
        };
        match journal.connection.execute_batch("COMMIT") {
            Ok(()) => transaction.disarm(),
            Err(error) => {
                let result = resolve_commit_error(&journal.connection, error);
                transaction.disarm();
                return Err(result);
            }
        }
        let summary = OwnedCommitSummary {
            sequence: 0,
            revision,
            state_sha256: state_hash,
        };
        let report = OwnedWriteReport {
            version: 1,
            operation: OwnedWriteOperation::Init,
            source,
            previous_head: None,
            head: summary,
        };
        Ok((journal, report))
    }

    /// Opens an existing journal for writes without creating or migrating it.
    ///
    /// # Errors
    ///
    /// Returns a typed path, family, version, or backend error.
    pub fn open(root: &Path) -> Result<Self, OwnedJournalError> {
        let path = checked_database_path(root)?;
        let connection = Connection::open_with_flags(
            &path,
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        connection.busy_timeout(std::time::Duration::ZERO)?;
        check_header(&connection)?;
        connection.execute_batch("PRAGMA foreign_keys=ON; PRAGMA synchronous=FULL;")?;
        Ok(Self {
            connection,
            poisoned: false,
        })
    }

    /// Imports a validated complete snapshot, or recognizes an exact replay.
    ///
    /// A missing root is created exclusively and populated in one transaction.
    /// An existing owned journal is compared as complete history without mutation.
    ///
    /// # Errors
    ///
    /// Returns an invalid-snapshot, path, integrity, conflict, busy, or backend
    /// error. A failed fresh import retains its partial root for diagnosis.
    pub fn import_snapshot(
        root: &Path,
        snapshot: &OwnedSnapshot,
    ) -> Result<(Self, OwnedImportReport), OwnedJournalError> {
        let payload = snapshot
            .to_json_bytes()
            .map_err(OwnedJournalError::InvalidSnapshot)?;
        let snapshot_sha256 = hex(&Sha256::digest(&payload));
        let data = snapshot.as_data();
        let journal_schema = i32::try_from(data.journal_schema).map_err(|_| {
            OwnedJournalError::InvalidSnapshot(OwnedSnapshotError::UnsupportedVersion {
                kind: "owned journal schema",
                found: data.journal_schema,
            })
        })?;
        let outcome = match fs::symlink_metadata(root) {
            Ok(_) => {
                let mut journal = Self::open(root)?;
                journal
                    .connection
                    .execute_batch("BEGIN IMMEDIATE")
                    .map_err(map_busy)?;
                let transaction =
                    RollbackGuard::new(&journal.connection, Some(&mut journal.poisoned));
                let existing = (|| {
                    let (schema, source, head) = read_header_head(&journal.connection)?;
                    let existing = snapshot_from_connection(&journal.connection)?;
                    let existing_payload = existing.to_json_bytes().map_err(|error| {
                        OwnedJournalError::Integrity(format!(
                            "existing snapshot could not be canonically serialized: {error}"
                        ))
                    })?;
                    if existing_payload != payload {
                        return Err(OwnedJournalError::ImportConflict(
                            "existing journal history differs from the imported snapshot".into(),
                        ));
                    }
                    if schema != journal_schema
                        || source != data.source
                        || head.revision.id != data.head
                    {
                        return Err(OwnedJournalError::Integrity(
                            "exact snapshot replay disagrees with persisted header".into(),
                        ));
                    }
                    Ok(())
                })();
                match existing {
                    Ok(()) => match journal.connection.execute_batch("COMMIT") {
                        Ok(()) => {
                            transaction.disarm();
                            (journal, OwnedImportOutcome::AlreadyPresent)
                        }
                        Err(error) => {
                            let error = resolve_commit_error(&journal.connection, error);
                            transaction.disarm();
                            if matches!(error, OwnedJournalError::CommitOutcomeUnknown(_)) {
                                journal.poisoned = true;
                            }
                            return Err(error);
                        }
                    },
                    Err(error) => {
                        let result = rollback_error(&journal.connection, error);
                        transaction.disarm();
                        if matches!(result, Err(OwnedJournalError::CommitOutcomeUnknown(_))) {
                            journal.poisoned = true;
                        }
                        result?;
                        unreachable!("rollback_error always returns an error")
                    }
                }
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                let parent = root
                    .parent()
                    .filter(|path| !path.as_os_str().is_empty())
                    .unwrap_or_else(|| Path::new("."));
                if !parent.is_dir() {
                    return Err(OwnedJournalError::Io(io::Error::new(
                        io::ErrorKind::NotFound,
                        "owned journal parent must exist",
                    )));
                }
                fs::create_dir(root)?;
                let path = root.join(owned_schema::FILE);
                let connection = Connection::open_with_flags(
                    &path,
                    OpenFlags::SQLITE_OPEN_READ_WRITE
                        | OpenFlags::SQLITE_OPEN_CREATE
                        | OpenFlags::SQLITE_OPEN_NO_MUTEX,
                )?;
                connection.busy_timeout(std::time::Duration::ZERO)?;
                owned_schema::configure(&connection)?;
                connection
                    .execute_batch("BEGIN IMMEDIATE")
                    .map_err(map_busy)?;
                let guard = RollbackGuard::new(&connection, None);
                let ddl = (|| {
                    owned_schema::initialize(&connection)?;
                    if journal_schema == owned_schema::CURATION_SCHEMA_VERSION {
                        owned_schema::initialize_curation(&connection)?;
                    }
                    Ok::<_, OwnedJournalError>(())
                })();
                if let Err(error) = ddl {
                    let result = rollback_error::<()>(&connection, error);
                    guard.disarm();
                    return match result {
                        Err(error) => Err(error),
                        Ok(()) => unreachable!("DDL failure cannot succeed"),
                    };
                }
                import_rows_and_finish(&connection, snapshot, guard)?;
                (
                    Self {
                        connection,
                        poisoned: false,
                    },
                    OwnedImportOutcome::Imported,
                )
            }
            Err(error) => return Err(error.into()),
        };
        let report = OwnedImportReport {
            version: 1,
            outcome: outcome.1,
            source: data.source.clone(),
            head: data.head.clone(),
            journal_schema: data.journal_schema,
            revision_count: data.commits.len() as u64,
            snapshot_sha256,
        };
        Ok((outcome.0, report))
    }

    /// Appends exactly one revision if `expected_head` is still persisted.
    ///
    /// # Errors
    ///
    /// Returns conflict before append-policy failures when the persisted head
    /// differs, or a typed validation, immutability, busy, or integrity error.
    pub fn commit(
        &mut self,
        expected_head: &EntityId,
        candidate: &Dictionary,
    ) -> Result<OwnedWriteReport, OwnedJournalError> {
        if self.poisoned {
            return Err(OwnedJournalError::CommitOutcomeUnknown(
                "writer handle is poisoned".into(),
            ));
        }
        self.connection
            .execute_batch("BEGIN IMMEDIATE")
            .map_err(map_busy)?;
        let transaction = RollbackGuard::new(&self.connection, Some(&mut self.poisoned));
        let prepared = (|| {
            let current = current_view(&self.connection)?;
            if current.commit.revision.id != *expected_head {
                return Err(OwnedJournalError::Conflict {
                    expected: expected_head.clone(),
                    actual: current.commit.revision.id,
                });
            }
            let (old_sections, new_sections) = validate_edit(&current.dictionary, candidate)?;
            let revision = candidate
                .metadata()
                .owned_revisions
                .last()
                .cloned()
                .ok_or_else(|| {
                    OwnedJournalError::Integrity("candidate revision is missing".into())
                })?;
            let sequence = current
                .commit
                .sequence
                .checked_add(1)
                .filter(|n| *n <= i64::MAX as u64)
                .ok_or_else(|| {
                    OwnedJournalError::InvalidRequest(
                        "journal sequence exceeds SQLite INTEGER range".into(),
                    )
                })?;
            let hash = state_hash(candidate)?;
            let schema = journal_schema(&self.connection)?;
            insert_suffix(
                &self.connection,
                &old_sections,
                &new_sections,
                &revision,
                sequence,
                &hash,
                &current.source,
            )?;
            if schema == owned_schema::CURATION_SCHEMA_VERSION {
                let prior_curation = load_curation_document(
                    &self.connection,
                    &current.commit,
                    current.source.clone(),
                    current.dictionary.clone(),
                )?;
                let mut carried = prior_curation.into_data();
                let previous_active = carried
                    .revisions
                    .last()
                    .map(|row| row.active.clone())
                    .unwrap_or_default();
                carried.dictionary = candidate.clone();
                carried.revisions.push(OwnedCurationRevision {
                    revision: revision.id.clone(),
                    active: previous_active,
                    tombstones: Vec::new(),
                });
                let carried = OwnedCurationDocument::from_data(carried)
                    .map_err(OwnedJournalError::InvalidCuration)?;
                persist_curation_revision(
                    &self.connection,
                    &current.commit,
                    sequence,
                    &current.source,
                    &revision,
                    &hash,
                    carried.as_data(),
                )?;
            }
            Ok::<_, OwnedJournalError>((
                current.source,
                current.commit.revision.id,
                sequence,
                revision,
                hash,
            ))
        })();
        let (source, previous, sequence, revision, hash) = match prepared {
            Ok(value) => value,
            Err(error) => {
                let result = rollback_error(&self.connection, error);
                transaction.disarm();
                if matches!(result, Err(OwnedJournalError::CommitOutcomeUnknown(_))) {
                    self.poisoned = true;
                }
                return result;
            }
        };
        match self.connection.execute_batch("COMMIT") {
            Ok(()) => {
                transaction.disarm();
                Ok(OwnedWriteReport {
                    version: 1,
                    operation: OwnedWriteOperation::Edit,
                    source,
                    previous_head: Some(previous),
                    head: OwnedCommitSummary {
                        sequence,
                        revision,
                        state_sha256: hash,
                    },
                })
            }
            Err(error) => {
                let resolved = resolve_commit_error(&self.connection, error);
                transaction.disarm();
                if matches!(resolved, OwnedJournalError::CommitOutcomeUnknown(_)) {
                    self.poisoned = true;
                }
                Err(resolved)
            }
        }
    }

    /// Explicitly adds the curation sidecar to a schema-one journal.
    ///
    /// # Errors
    ///
    /// Returns a head conflict, a typed integrity/version error, or a busy
    /// failure if another writer or a pinned reader prevents the migration.
    pub fn migrate_curation(
        &mut self,
        expected_head: &EntityId,
    ) -> Result<OwnedCurationMigrationReport, OwnedJournalError> {
        if self.poisoned {
            return Err(OwnedJournalError::CommitOutcomeUnknown(
                "writer handle is poisoned".into(),
            ));
        }
        self.connection
            .execute_batch("BEGIN IMMEDIATE")
            .map_err(map_busy)?;
        let transaction = RollbackGuard::new(&self.connection, Some(&mut self.poisoned));
        let result = (|| {
            let (schema, source, head) = read_header_head(&self.connection)?;
            if &head.revision.id != expected_head {
                return Err(OwnedJournalError::Conflict {
                    expected: expected_head.clone(),
                    actual: head.revision.id,
                });
            }
            if schema == owned_schema::CURATION_SCHEMA_VERSION {
                return Ok(OwnedCurationMigrationReport {
                    version: 1,
                    source,
                    head: head.revision.id,
                    from_schema: 2,
                    to_schema: 2,
                    outcome: OwnedMigrationOutcome::AlreadyCurrent,
                });
            }
            if schema != owned_schema::SCHEMA_VERSION {
                return Err(OwnedJournalError::UnsupportedVersion {
                    found: schema as u32,
                });
            }
            let history = read_history_summaries(&self.connection)?;
            if history.last() != Some(&head) {
                return Err(OwnedJournalError::Integrity(
                    "current head does not match the complete commit history".into(),
                ));
            }
            // Migration is the explicit full-history verification boundary.
            for summary in &history {
                load_state(&self.connection, summary)?;
            }
            owned_schema::initialize_curation(&self.connection)?;
            let mut curation_prefix = Vec::with_capacity(history.len());
            for summary in &history {
                let revision = summary.revision.id.to_string();
                let curation_revision = OwnedCurationRevision {
                    revision: summary.revision.id.clone(),
                    active: Vec::new(),
                    tombstones: Vec::new(),
                };
                let revision_payload = serde_json::to_vec(&curation_revision)
                    .map_err(|error| OwnedJournalError::Integrity(error.to_string()))?;
                curation_prefix.push(curation_revision);
                self.connection.execute(
                    "INSERT INTO owned_curation_revisions(revision_id,payload) VALUES(?1,?2)",
                    params![revision, revision_payload],
                )?;
                let hash = curation_hash(&source, summary, &[], &[], &curation_prefix)?;
                self.connection.execute(
                    "INSERT INTO owned_curation_prefixes(revision_id,contexts_length,records_length,curation_sha256) VALUES(?1,0,0,?2)",
                    params![revision, hash],
                )?;
            }
            self.connection.execute_batch(&format!(
                "PRAGMA user_version={}; UPDATE owned_meta SET schema_version={} WHERE singleton=1;",
                owned_schema::CURATION_SCHEMA_VERSION,
                owned_schema::CURATION_SCHEMA_VERSION
            ))?;
            Ok(OwnedCurationMigrationReport {
                version: 1,
                source,
                head: head.revision.id,
                from_schema: 1,
                to_schema: 2,
                outcome: OwnedMigrationOutcome::Migrated,
            })
        })();
        match result {
            Ok(report) => match self.connection.execute_batch("COMMIT") {
                Ok(()) => {
                    transaction.disarm();
                    Ok(report)
                }
                Err(error) => {
                    let error = resolve_commit_error(&self.connection, error);
                    transaction.disarm();
                    if matches!(error, OwnedJournalError::CommitOutcomeUnknown(_)) {
                        self.poisoned = true;
                    }
                    Err(error)
                }
            },
            Err(error) => {
                let error = rollback_error(&self.connection, error);
                transaction.disarm();
                if matches!(error, Err(OwnedJournalError::CommitOutcomeUnknown(_))) {
                    self.poisoned = true;
                }
                error
            }
        }
    }

    /// Appends a validated curation document with one matching base revision.
    ///
    /// # Errors
    ///
    /// Returns a migration-required error on schema one, a head conflict, an
    /// immutable-prefix error, or a typed curation/journal failure.
    pub fn commit_curated(
        &mut self,
        expected_head: &EntityId,
        candidate: &OwnedCurationDocument,
    ) -> Result<OwnedWriteReport, OwnedJournalError> {
        if self.poisoned {
            return Err(OwnedJournalError::CommitOutcomeUnknown(
                "writer handle is poisoned".into(),
            ));
        }
        self.connection
            .execute_batch("BEGIN IMMEDIATE")
            .map_err(map_busy)?;
        let transaction = RollbackGuard::new(&self.connection, Some(&mut self.poisoned));
        let prepared = (|| {
            if journal_schema(&self.connection)? != owned_schema::CURATION_SCHEMA_VERSION {
                return Err(OwnedJournalError::MigrationRequired);
            }
            let current = current_view(&self.connection)?;
            if &current.commit.revision.id != expected_head {
                return Err(OwnedJournalError::Conflict {
                    expected: expected_head.clone(),
                    actual: current.commit.revision.id,
                });
            }
            let candidate_data = candidate.as_data();
            let (old_sections, new_sections) =
                validate_edit(&current.dictionary, &candidate_data.dictionary)?;
            let prior = load_curation_document(
                &self.connection,
                &current.commit,
                current.source.clone(),
                current.dictionary.clone(),
            )?;
            let prior_data = prior.as_data();
            if candidate_data.contexts.len() < prior_data.contexts.len()
                || candidate_data.contexts[..prior_data.contexts.len()] != prior_data.contexts[..]
                || candidate_data.records.len() < prior_data.records.len()
                || candidate_data.records[..prior_data.records.len()] != prior_data.records[..]
                || candidate_data.revisions.len() != prior_data.revisions.len() + 1
                || candidate_data.revisions[..prior_data.revisions.len()]
                    != prior_data.revisions[..]
            {
                return Err(OwnedJournalError::InvalidCuration(
                    OwnedCurationError::InvalidDocument(
                        "curation edit changed an immutable prefix".into(),
                    ),
                ));
            }
            let revision = candidate_data
                .dictionary
                .metadata()
                .owned_revisions
                .last()
                .cloned()
                .ok_or_else(|| {
                    OwnedJournalError::Integrity("candidate revision is missing".into())
                })?;
            let sequence = current
                .commit
                .sequence
                .checked_add(1)
                .filter(|n| *n <= i64::MAX as u64)
                .ok_or_else(|| {
                    OwnedJournalError::InvalidRequest(
                        "journal sequence exceeds SQLite INTEGER range".into(),
                    )
                })?;
            let hash = state_hash(&candidate_data.dictionary)?;
            insert_suffix(
                &self.connection,
                &old_sections,
                &new_sections,
                &revision,
                sequence,
                &hash,
                &current.source,
            )?;
            persist_curation_revision(
                &self.connection,
                &current.commit,
                sequence,
                &current.source,
                &revision,
                &hash,
                candidate_data,
            )?;
            Ok((
                current.source,
                current.commit.revision.id,
                sequence,
                revision,
                hash,
            ))
        })();
        match prepared {
            Ok((source, previous, sequence, revision, hash)) => {
                match self.connection.execute_batch("COMMIT") {
                    Ok(()) => {
                        transaction.disarm();
                        Ok(OwnedWriteReport {
                            version: 1,
                            operation: OwnedWriteOperation::Edit,
                            source,
                            previous_head: Some(previous),
                            head: OwnedCommitSummary {
                                sequence,
                                revision,
                                state_sha256: hash,
                            },
                        })
                    }
                    Err(error) => {
                        let error = resolve_commit_error(&self.connection, error);
                        transaction.disarm();
                        if matches!(error, OwnedJournalError::CommitOutcomeUnknown(_)) {
                            self.poisoned = true;
                        }
                        Err(error)
                    }
                }
            }
            Err(error) => {
                let error = rollback_error(&self.connection, error);
                transaction.disarm();
                if matches!(error, Err(OwnedJournalError::CommitOutcomeUnknown(_))) {
                    self.poisoned = true;
                }
                error
            }
        }
    }
}

impl OwnedJournalReader {
    /// Opens a read-only, pinned snapshot without creating or migrating it.
    ///
    /// # Errors
    ///
    /// Returns an error for a missing path, wrong family/version, or backend failure.
    pub fn open(root: &Path) -> Result<Self, OwnedJournalError> {
        let connection = Connection::open_with_flags(
            checked_database_path(root)?,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        connection.execute_batch("BEGIN DEFERRED")?;
        check_header(&connection)?;
        Ok(Self { connection })
    }

    /// Exports the complete validated history visible in this pinned reader.
    ///
    /// # Errors
    ///
    /// Returns an integrity or backend error if any retained base or curation
    /// history is malformed or incomplete.
    pub fn snapshot(&self) -> Result<OwnedSnapshot, OwnedJournalError> {
        snapshot_from_connection(&self.connection)
    }

    /// Reads the checked current-head summary from this pinned snapshot.
    ///
    /// # Errors
    ///
    /// Returns an integrity or backend error if the current commit summary is invalid.
    pub fn head(&self) -> Result<OwnedHeadReport, OwnedJournalError> {
        let (source, summary) = read_current_summary(&self.connection)?;
        Ok(OwnedHeadReport {
            version: 1,
            source,
            head: summary,
        })
    }

    /// Reads the compact checked identity of the current head.
    ///
    /// This validates the local revision summary, including its typed roster,
    /// but does not reconstruct the dictionary or walk historical revisions.
    ///
    /// # Errors
    ///
    /// Returns an integrity or backend error when the journal header, summary,
    /// or schema-two curation prefix is malformed.
    pub fn head_identity(&self) -> Result<OwnedRevisionIdentity, OwnedJournalError> {
        let schema = journal_schema(&self.connection)?;
        let (_, summary) = read_current_summary(&self.connection)?;
        revision_identity_for(&self.connection, schema, summary)
    }

    /// Reads the compact checked identity of one historical revision.
    ///
    /// The selected row's typed revision summary is validated, but the
    /// dictionary body and unrelated history are not reconstructed.
    ///
    /// # Errors
    ///
    /// Returns [`OwnedJournalError::NotFound`] for an unknown revision and an
    /// integrity or backend error for a malformed summary or curation prefix.
    pub fn revision_identity(
        &self,
        revision: &EntityId,
    ) -> Result<OwnedRevisionIdentity, OwnedJournalError> {
        check_header(&self.connection)?;
        let schema = journal_schema(&self.connection)?;
        let summary = summary_for(&self.connection, revision)?;
        revision_identity_for(&self.connection, schema, summary)
    }

    /// Reads the contiguous typed commit chain without reconstructing every model body.
    ///
    /// # Errors
    ///
    /// Returns an integrity or backend error for malformed summary/chain metadata.
    pub fn history(&self) -> Result<OwnedHistoryReport, OwnedJournalError> {
        let (source, head) = read_current_summary(&self.connection)?;
        let mut statement = self.connection.prepare("SELECT revision_id,parent_revision,sequence,state_sha256 FROM owned_commits ORDER BY sequence")?;
        let mut rows = statement.query([])?;
        let mut commits = Vec::new();
        let mut previous: Option<EntityId> = None;
        while let Some(row) = rows.next()? {
            let id_text: String = row.get(0)?;
            let id = EntityId::new(id_text)
                .map_err(|error| OwnedJournalError::Integrity(error.to_string()))?;
            let parent_text: Option<String> = row.get(1)?;
            let parent = parent_text
                .map(EntityId::new)
                .transpose()
                .map_err(|error| OwnedJournalError::Integrity(error.to_string()))?;
            let sequence: i64 = row.get(2)?;
            let hash: String = row.get(3)?;
            if sequence < 0
                || sequence as usize != commits.len()
                || parent != previous
                || !is_sha256(&hash)
            {
                return Err(OwnedJournalError::Integrity(
                    "owned commit chain is not contiguous".into(),
                ));
            }
            let revision = revision_for(&self.connection, &id, sequence as u64)?;
            commits.push(OwnedCommitSummary {
                sequence: sequence as u64,
                revision: revision.clone(),
                state_sha256: hash,
            });
            previous = Some(id);
        }
        if commits.last() != Some(&head) {
            return Err(OwnedJournalError::Integrity(
                "current head does not match commit chain".into(),
            ));
        }
        Ok(OwnedHistoryReport {
            version: 1,
            source,
            head: head.revision.id,
            commits,
        })
    }

    /// Reconstructs, validates, and hashes one requested historical state.
    ///
    /// # Errors
    ///
    /// Returns [`OwnedJournalError::NotFound`] for an unknown revision and an
    /// integrity/model error for invalid persisted bodies or hashes.
    pub fn show(&self, revision: &EntityId) -> Result<OwnedRevisionView, OwnedJournalError> {
        let summary = summary_for(&self.connection, revision)?;
        let state = load_state(&self.connection, &summary)?;
        let (source, _) = read_current_summary(&self.connection)?;
        Ok(OwnedRevisionView {
            version: 1,
            source,
            commit: summary,
            dictionary: state,
        })
    }

    /// Reconstructs the current head from this pinned snapshot.
    ///
    /// # Errors
    ///
    /// Returns a model, integrity, or backend error when the current state is invalid.
    pub fn show_head(&self) -> Result<OwnedRevisionView, OwnedJournalError> {
        let (_, summary) = read_current_summary(&self.connection)?;
        self.show(&summary.revision.id)
    }

    /// Reconstructs and verifies one revision's complete curation document.
    ///
    /// # Errors
    ///
    /// Returns `NotFound` for an absent revision and typed integrity/curation
    /// errors for malformed persisted prefixes or bodies.
    pub fn curation_show(
        &self,
        revision: &EntityId,
    ) -> Result<OwnedCurationDocument, OwnedJournalError> {
        if journal_schema(&self.connection)? != owned_schema::CURATION_SCHEMA_VERSION {
            return Err(OwnedJournalError::MigrationRequired);
        }
        let summary = summary_for(&self.connection, revision)?;
        let dictionary = load_state(&self.connection, &summary)?;
        let source = dictionary
            .registry()
            .as_document()
            .sources
            .first()
            .ok_or_else(|| OwnedJournalError::Integrity("owned source missing".into()))?
            .id
            .clone();
        load_curation_document(&self.connection, &summary, source, dictionary)
    }

    /// Reconstructs and verifies curation state at the pinned current head.
    ///
    /// # Errors
    ///
    /// Returns a migration, integrity, curation, or backend error.
    pub fn curation_show_head(&self) -> Result<OwnedCurationDocument, OwnedJournalError> {
        let (_, summary) = read_current_summary(&self.connection)?;
        self.curation_show(&summary.revision.id)
    }
}

fn checked_database_path(root: &Path) -> Result<PathBuf, OwnedJournalError> {
    let root_meta = fs::symlink_metadata(root)?;
    if !root_meta.file_type().is_dir() {
        return Err(OwnedJournalError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            "owned journal root must be a real directory",
        )));
    }
    let path = root.join(owned_schema::FILE);
    let file_meta = fs::symlink_metadata(&path).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            OwnedJournalError::Io(io::Error::new(
                io::ErrorKind::NotFound,
                "owned journal database is missing",
            ))
        } else {
            OwnedJournalError::Io(error)
        }
    })?;
    if !file_meta.file_type().is_file() {
        return Err(OwnedJournalError::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            "owned journal database must be a regular file",
        )));
    }
    Ok(path)
}

#[derive(Clone)]
struct Sections(std::collections::BTreeMap<&'static str, Vec<Vec<u8>>>);
impl std::ops::Index<&str> for Sections {
    type Output = Vec<Vec<u8>>;
    fn index(&self, index: &str) -> &Self::Output {
        &self.0[index]
    }
}

fn sections(dictionary: &Dictionary) -> Result<Sections, OwnedJournalError> {
    let registry = dictionary.registry().as_document();
    let metadata = dictionary.metadata();
    let mut out = std::collections::BTreeMap::new();
    macro_rules! add {
        ($name:literal, $rows:expr) => {
            out.insert(
                $name,
                ($rows)
                    .iter()
                    .map(serde_json::to_vec)
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(|e| OwnedJournalError::Integrity(e.to_string()))?,
            );
        };
    }
    add!("registry.sources", registry.sources);
    add!("registry.contributors", registry.contributors);
    add!("registry.distributors", registry.distributors);
    add!("registry.aliases", registry.aliases);
    add!("registry.lineage", registry.lineage);
    add!("registry.snapshots", registry.snapshots);
    add!("registry.occurrences", registry.occurrences);
    add!("captures", dictionary.captures());
    add!("metadata.code_notices", metadata.code_notices);
    add!("metadata.collections", metadata.collections);
    add!("metadata.components", metadata.components);
    add!("metadata.releases", metadata.releases);
    add!("metadata.entries", metadata.entries);
    add!("metadata.assertions", metadata.assertions);
    add!("metadata.contexts", metadata.contexts);
    add!("metadata.frames", metadata.frames);
    add!("metadata.forms", metadata.forms);
    add!("metadata.etymologies", metadata.etymologies);
    add!("metadata.relations", metadata.relations);
    add!("metadata.owned_revisions", metadata.owned_revisions);
    add!("metadata.editorial_senses", metadata.editorial_senses);
    add!("metadata.computations", metadata.computations);
    add!("metadata.annotations", metadata.annotations);
    add!("metadata.extensions", metadata.extensions);
    Ok(Sections(out))
}

fn state_hash(dictionary: &Dictionary) -> Result<String, OwnedJournalError> {
    Ok(hex(&Sha256::digest(
        dictionary
            .to_json_bytes()
            .map_err(OwnedJournalError::InvalidDictionary)?,
    )))
}
fn hex(bytes: &[u8]) -> String {
    const D: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(D[(b >> 4) as usize] as char);
        s.push(D[(b & 15) as usize] as char);
    }
    s
}
fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn validate_genesis(dictionary: &Dictionary) -> Result<(), OwnedJournalError> {
    let document = dictionary.document();
    let registry = dictionary.registry().as_document();
    let metadata = dictionary.metadata();
    if document.version != 1
        || document.legacy_archive.is_some()
        || registry.sources.len() != 1
        || registry.sources[0].origin != SourceOrigin::OwnedDraft
        || metadata.owned_heads.len() != 1
        || metadata.owned_revisions.len() != 1
    {
        return Err(OwnedJournalError::InvalidRequest(
            "genesis must contain one owned source, head, and revision without a legacy archive"
                .into(),
        ));
    }
    let source = &registry.sources[0].id;
    let head = &metadata.owned_heads[0];
    let revision = &metadata.owned_revisions[0];
    if &head.source != source
        || head.revision != revision.id
        || &revision.source != source
        || revision.parent.is_some()
        || !revision.tombstones.is_empty()
        || metadata.catalogue_revision != 0
    {
        return Err(OwnedJournalError::InvalidRequest(
            "genesis ownership/head/revision controls are inconsistent".into(),
        ));
    }
    if registry
        .contributors
        .iter()
        .all(|row| row.id != revision.editor)
    {
        return Err(OwnedJournalError::InvalidRequest(
            "genesis editor must be registered".into(),
        ));
    }
    let owned_collections: std::collections::BTreeSet<_> = metadata
        .collections
        .iter()
        .filter(|collection| collection.source == *source)
        .map(|collection| collection.id.clone())
        .collect();
    let owned_components: std::collections::BTreeSet<_> = metadata
        .components
        .iter()
        .filter(|component| owned_collections.contains(&component.collection))
        .map(|component| component.id.clone())
        .collect();
    let releases: Vec<_> = metadata
        .releases
        .iter()
        .filter(|release| owned_components.contains(&release.component))
        .collect();
    if releases.is_empty() || owned_collections.is_empty() || owned_components.is_empty() {
        return Err(OwnedJournalError::InvalidRequest(
            "genesis requires owned collection, component, and release rows".into(),
        ));
    }
    if releases
        .iter()
        .any(|release| release.content_notices.is_empty())
    {
        return Err(OwnedJournalError::InvalidRequest(
            "each owned release requires a content notice".into(),
        ));
    }
    Ok(())
}

#[derive(Deserialize)]
struct DictionaryHeader {
    version: u32,
    legacy_archive: Option<Vec<u8>>,
}

fn header_fields(dictionary: &Dictionary) -> Result<DictionaryHeader, OwnedJournalError> {
    let bytes = dictionary
        .to_json_bytes()
        .map_err(OwnedJournalError::InvalidDictionary)?;
    serde_json::from_slice(&bytes).map_err(|error| {
        OwnedJournalError::Integrity(format!("invalid dictionary header: {error}"))
    })
}

fn validate_edit(
    current: &Dictionary,
    candidate: &Dictionary,
) -> Result<(Sections, Sections), OwnedJournalError> {
    let old_registry = current.registry().as_document();
    let new_registry = candidate.registry().as_document();
    let old = current.metadata();
    let new = candidate.metadata();
    let old_header = header_fields(current)?;
    let new_header = header_fields(candidate)?;
    let source = old_registry
        .sources
        .first()
        .ok_or_else(|| OwnedJournalError::Integrity("current source is missing".into()))?
        .id
        .clone();
    let next_catalogue_revision = old
        .catalogue_revision
        .checked_add(1)
        .filter(|revision| *revision <= i64::MAX as u64)
        .ok_or_else(|| {
            OwnedJournalError::InvalidRequest(
                "catalogue revision exceeds SQLite INTEGER range".into(),
            )
        })?;
    if new_header.version != old_header.version
        || new_header.legacy_archive != old_header.legacy_archive
        || new_registry.sources.len() != 1
        || new_registry.sources[0].id != source
        || new.owned_heads.len() != 1
        || new.owned_revisions.len() != old.owned_revisions.len() + 1
        || new.catalogue_revision != next_catalogue_revision
    {
        return Err(OwnedJournalError::InvalidRequest(
            "edit must append exactly one revision for the same owned source".into(),
        ));
    }
    let prior = &old
        .owned_revisions
        .last()
        .ok_or_else(|| OwnedJournalError::Integrity("current revision is missing".into()))?;
    let next = new
        .owned_revisions
        .last()
        .expect("candidate revision length checked");
    if next.source != source
        || next.parent.as_ref() != Some(&prior.id)
        || new.owned_heads[0].source != source
        || new.owned_heads[0].revision != next.id
    {
        return Err(OwnedJournalError::InvalidRequest(
            "new revision must parent the persisted head and become the sole head".into(),
        ));
    }
    let old_active: Vec<_> = prior.assertions.iter().map(ToString::to_string).collect();
    let new_active: Vec<_> = next.assertions.iter().map(ToString::to_string).collect();
    let expected: Vec<_> = old_active
        .iter()
        .filter(|id| !new_active.contains(id))
        .cloned()
        .collect();
    let tombstones: Vec<_> = next.tombstones.iter().map(ToString::to_string).collect();
    if tombstones != expected {
        return Err(OwnedJournalError::InvalidRequest(
            "new revision tombstones must exactly list removed prior active assertions in order"
                .into(),
        ));
    }
    let old_rows = sections(current)?;
    let new_rows = sections(candidate)?;
    for section in SECTIONS {
        let a = &old_rows[section];
        let b = &new_rows[section];
        if b.len() < a.len() || a.iter().zip(b).any(|(x, y)| x != y) {
            let ordinal = a.iter().zip(b).position(|(x, y)| x != y).unwrap_or(b.len());
            return Err(OwnedJournalError::ImmutableRecord {
                section: section.to_owned(),
                ordinal: ordinal as u64,
            });
        }
    }
    let prior_assertions = old.assertions.len();
    if new.assertions[prior_assertions..]
        .iter()
        .any(|assertion| !next.assertions.contains(&assertion.id))
    {
        return Err(OwnedJournalError::InvalidRequest(
            "every appended assertion must be present in the new active set".into(),
        ));
    }
    if new.owned_revisions[..old.owned_revisions.len()] != old.owned_revisions[..] {
        return Err(OwnedJournalError::ImmutableRecord {
            section: "metadata.owned_revisions".into(),
            ordinal: (old.owned_revisions.len() - 1) as u64,
        });
    }
    Ok((old_rows, new_rows))
}

fn insert_initial(
    connection: &Connection,
    rows: &Sections,
    revision: &OwnedRevisionRecord,
    hash: &str,
    source: &SourceId,
) -> Result<(), OwnedJournalError> {
    let revision_text = revision.id.to_string();
    connection.execute("INSERT INTO owned_commits(revision_id,parent_revision,sequence,state_sha256) VALUES(?1,NULL,0,?2)",params![revision_text,hash])?;
    for section in SECTIONS {
        for (ordinal, payload) in rows[section].iter().enumerate() {
            connection.execute(
                "INSERT INTO owned_records(section,ordinal,payload) VALUES(?1,?2,?3)",
                params![section, ordinal as i64, payload],
            )?;
        }
        connection.execute(
            "INSERT INTO owned_commit_sections(revision_id,section,length) VALUES(?1,?2,?3)",
            params![revision_text, section, rows[section].len() as i64],
        )?;
    }
    connection.execute("UPDATE owned_meta SET source_id=?1,head_revision=?2,catalogue_revision=0 WHERE singleton=1",params![source.to_string(),revision_text])?;
    Ok(())
}
fn insert_suffix(
    connection: &Connection,
    old: &Sections,
    new: &Sections,
    revision: &OwnedRevisionRecord,
    sequence: u64,
    hash: &str,
    source: &SourceId,
) -> Result<(), OwnedJournalError> {
    let id = revision.id.to_string();
    let parent = revision.parent.as_ref().map(ToString::to_string);
    connection.execute("INSERT INTO owned_commits(revision_id,parent_revision,sequence,state_sha256) VALUES(?1,?2,?3,?4)",params![id,parent,sequence as i64,hash])?;
    for section in SECTIONS {
        for (offset, payload) in new[section].iter().enumerate().skip(old[section].len()) {
            connection.execute(
                "INSERT INTO owned_records(section,ordinal,payload) VALUES(?1,?2,?3)",
                params![section, offset as i64, payload],
            )?;
        }
        connection.execute(
            "INSERT INTO owned_commit_sections(revision_id,section,length) VALUES(?1,?2,?3)",
            params![id, section, new[section].len() as i64],
        )?;
    }
    let changed=connection.execute("UPDATE owned_meta SET head_revision=?1,catalogue_revision=?2 WHERE singleton=1 AND head_revision=?3 AND source_id=?4",params![id,sequence as i64,revision.parent.as_ref().map(ToString::to_string),source.to_string()])?;
    if changed != 1 {
        return Err(OwnedJournalError::Integrity(
            "conditional owned head update did not affect exactly one row".into(),
        ));
    }
    Ok(())
}
fn map_busy(error: rusqlite::Error) -> OwnedJournalError {
    if matches!(error,rusqlite::Error::SqliteFailure(ref e,_) if e.code==rusqlite::ErrorCode::DatabaseBusy || e.code==rusqlite::ErrorCode::DatabaseLocked)
    {
        OwnedJournalError::Busy(error.to_string())
    } else {
        OwnedJournalError::Backend(error)
    }
}
fn rollback_error<T>(
    connection: &Connection,
    error: OwnedJournalError,
) -> Result<T, OwnedJournalError> {
    if connection.is_autocommit() {
        return Err(error);
    }
    match connection.execute_batch("ROLLBACK") {
        Ok(()) if connection.is_autocommit() => Err(error),
        Ok(()) => Err(OwnedJournalError::CommitOutcomeUnknown(format!(
            "rollback returned but autocommit is false after {error}"
        ))),
        Err(rollback) => Err(OwnedJournalError::CommitOutcomeUnknown(format!(
            "{error}; rollback failed: {rollback}"
        ))),
    }
}

/// Best-effort rollback during unwinding; normal exits use observed rollback paths.
struct RollbackGuard<'a> {
    connection: &'a Connection,
    poisoned: Option<&'a mut bool>,
    armed: bool,
}

impl<'a> RollbackGuard<'a> {
    fn new(connection: &'a Connection, poisoned: Option<&'a mut bool>) -> Self {
        Self {
            connection,
            poisoned,
            armed: true,
        }
    }

    fn disarm(mut self) {
        self.armed = false;
    }
}

impl Drop for RollbackGuard<'_> {
    fn drop(&mut self) {
        if !self.armed || self.connection.is_autocommit() {
            return;
        }
        let rollback_failed = self.connection.execute_batch("ROLLBACK").is_err();
        if (rollback_failed || !self.connection.is_autocommit())
            && let Some(poisoned) = self.poisoned.as_deref_mut()
        {
            *poisoned = true;
        }
    }
}
fn resolve_commit_error(connection: &Connection, error: rusqlite::Error) -> OwnedJournalError {
    if connection.is_autocommit() {
        OwnedJournalError::CommitOutcomeUnknown(format!(
            "commit failed after transaction ended: {error}"
        ))
    } else {
        match connection.execute_batch("ROLLBACK") {
            Ok(()) if connection.is_autocommit() => map_busy(error),
            Ok(()) => OwnedJournalError::CommitOutcomeUnknown(
                "rollback did not return to autocommit".into(),
            ),
            Err(rollback) => OwnedJournalError::CommitOutcomeUnknown(format!(
                "commit failed: {error}; rollback failed: {rollback}"
            )),
        }
    }
}

fn check_header(connection: &Connection) -> Result<(), OwnedJournalError> {
    let application_id: i32 =
        connection.pragma_query_value(None, "application_id", |r| r.get(0))?;
    if application_id != owned_schema::APPLICATION_ID {
        return Err(OwnedJournalError::WrongFamily);
    }
    let schema: i32 = connection.pragma_query_value(None, "user_version", |r| r.get(0))?;
    let (family, schema_meta, model, registry): (String, i32, i32, i32) = connection.query_row(
        "SELECT family,schema_version,model_version,registry_version FROM owned_meta WHERE singleton=1",
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
    )?;
    if family != FAMILY {
        return Err(OwnedJournalError::WrongFamily);
    }
    if (schema != 1 && schema != 2) || schema_meta != schema || model != 1 || registry != 1 {
        return Err(OwnedJournalError::UnsupportedVersion {
            found: schema as u32,
        });
    }
    Ok(())
}

fn journal_schema(connection: &Connection) -> Result<i32, OwnedJournalError> {
    let pragma: i32 = connection.pragma_query_value(None, "user_version", |r| r.get(0))?;
    let meta: i32 = connection.query_row(
        "SELECT schema_version FROM owned_meta WHERE singleton=1",
        [],
        |row| row.get(0),
    )?;
    if pragma != meta {
        return Err(OwnedJournalError::UnsupportedVersion {
            found: pragma as u32,
        });
    }
    Ok(pragma)
}

fn read_header_head(
    connection: &Connection,
) -> Result<(i32, SourceId, OwnedCommitSummary), OwnedJournalError> {
    check_header(connection)?;
    let schema = journal_schema(connection)?;
    let (source, summary) = read_current_summary(connection)?;
    Ok((schema, source, summary))
}
fn read_current_summary(
    connection: &Connection,
) -> Result<(SourceId, OwnedCommitSummary), OwnedJournalError> {
    check_header(connection)?;
    let (source_text, head, catalogue_revision): (String, String, i64) = connection.query_row(
        "SELECT source_id,head_revision,catalogue_revision FROM owned_meta WHERE singleton=1",
        [],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
    )?;
    let source =
        SourceId::new(source_text).map_err(|e| OwnedJournalError::Integrity(e.to_string()))?;
    let id = EntityId::new(head).map_err(|e| OwnedJournalError::Integrity(e.to_string()))?;
    let summary = summary_for(connection, &id)?;
    if catalogue_revision < 0
        || catalogue_revision as u64 != summary.sequence
        || summary.revision.source != source
    {
        return Err(OwnedJournalError::Integrity(
            "owned header disagrees with current revision summary".into(),
        ));
    }
    Ok((source, summary))
}

fn revision_identity_for(
    connection: &Connection,
    schema: i32,
    summary: OwnedCommitSummary,
) -> Result<OwnedRevisionIdentity, OwnedJournalError> {
    let curation_state_sha256 = match schema {
        1 => None,
        2 => Some(read_curation_prefix_identity(
            connection,
            &summary.revision.id,
        )?),
        other => {
            return Err(OwnedJournalError::UnsupportedVersion {
                found: other as u32,
            });
        }
    };
    Ok(OwnedRevisionIdentity {
        journal_schema: schema as u32,
        source: summary.revision.source,
        revision: summary.revision.id,
        sequence: summary.sequence,
        state_sha256: summary.state_sha256,
        curation_state_sha256,
    })
}

fn read_curation_prefix_identity(
    connection: &Connection,
    revision: &EntityId,
) -> Result<String, OwnedJournalError> {
    let prefix: (i64, i64, String) = connection
        .query_row(
            "SELECT contexts_length,records_length,curation_sha256 FROM owned_curation_prefixes WHERE revision_id=?1",
            [revision.to_string()],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|error| match error {
            rusqlite::Error::QueryReturnedNoRows => {
                OwnedJournalError::Integrity("missing curation prefix".into())
            }
            other => other.into(),
        })?;
    let (contexts_length, records_length, curation_sha256) = prefix;
    if contexts_length < 0 || records_length < 0 || !is_sha256(&curation_sha256) {
        return Err(OwnedJournalError::Integrity(
            "malformed curation prefix".into(),
        ));
    }
    Ok(curation_sha256)
}

fn read_history_summaries(
    connection: &Connection,
) -> Result<Vec<OwnedCommitSummary>, OwnedJournalError> {
    let mut statement = connection.prepare(
        "SELECT revision_id,parent_revision,sequence,state_sha256 FROM owned_commits ORDER BY sequence",
    )?;
    let mut rows = statement.query([])?;
    let mut summaries = Vec::new();
    let mut previous: Option<EntityId> = None;
    while let Some(row) = rows.next()? {
        let id_text: String = row.get(0)?;
        let id = EntityId::new(id_text)
            .map_err(|error| OwnedJournalError::Integrity(error.to_string()))?;
        let parent = row
            .get::<_, Option<String>>(1)?
            .map(EntityId::new)
            .transpose()
            .map_err(|error| OwnedJournalError::Integrity(error.to_string()))?;
        let sequence: i64 = row.get(2)?;
        let hash: String = row.get(3)?;
        if sequence < 0
            || sequence as usize != summaries.len()
            || parent != previous
            || !is_sha256(&hash)
        {
            return Err(OwnedJournalError::Integrity(
                "owned commit chain is not contiguous".into(),
            ));
        }
        let revision = revision_for(connection, &id, sequence as u64)?;
        summaries.push(OwnedCommitSummary {
            sequence: sequence as u64,
            revision,
            state_sha256: hash,
        });
        previous = Some(id);
    }
    Ok(summaries)
}

#[derive(Serialize)]
struct CurationHashBody<'a> {
    version: u32,
    source: &'a SourceId,
    revision: &'a EntityId,
    base_state_sha256: &'a str,
    contexts: &'a [OwnedContextPin],
    records: &'a [OwnedCurationRecord],
    revisions: &'a [OwnedCurationRevision],
}

fn curation_hash(
    source: &SourceId,
    summary: &OwnedCommitSummary,
    contexts: &[OwnedContextPin],
    records: &[OwnedCurationRecord],
    revisions: &[OwnedCurationRevision],
) -> Result<String, OwnedJournalError> {
    let body = CurationHashBody {
        version: 1,
        source,
        revision: &summary.revision.id,
        base_state_sha256: &summary.state_sha256,
        contexts,
        records,
        revisions,
    };
    let bytes = serde_json::to_vec(&body)
        .map_err(|error| OwnedJournalError::Integrity(error.to_string()))?;
    Ok(hex(&Sha256::digest(bytes)))
}

fn persist_curation_revision(
    connection: &Connection,
    previous: &OwnedCommitSummary,
    sequence: u64,
    source: &SourceId,
    revision: &OwnedRevisionRecord,
    state_sha256: &str,
    data: &OwnedCurationData,
) -> Result<(), OwnedJournalError> {
    if data.revisions.len() as u64 != sequence + 1
        || data.revisions.last().map(|row| &row.revision) != Some(&revision.id)
    {
        return Err(OwnedJournalError::InvalidCuration(
            OwnedCurationError::InvalidDocument(
                "curation history does not match new base revision".into(),
            ),
        ));
    }
    let previous_lengths: (i64, i64) = connection.query_row(
        "SELECT contexts_length,records_length FROM owned_curation_prefixes WHERE revision_id=?1",
        [previous.revision.id.to_string()],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    let (contexts_start, records_start) = previous_lengths;
    if contexts_start < 0
        || records_start < 0
        || data.contexts.len() < contexts_start as usize
        || data.records.len() < records_start as usize
    {
        return Err(OwnedJournalError::Integrity(
            "curation append prefix is malformed".into(),
        ));
    }
    for (ordinal, pin) in data
        .contexts
        .iter()
        .enumerate()
        .skip(contexts_start as usize)
    {
        let payload = serde_json::to_vec(pin)
            .map_err(|error| OwnedJournalError::Integrity(error.to_string()))?;
        connection.execute(
            "INSERT INTO owned_context_pins(ordinal,id,payload) VALUES(?1,?2,?3)",
            params![ordinal as i64, pin.id.to_string(), payload],
        )?;
    }
    for (ordinal, record) in data.records.iter().enumerate().skip(records_start as usize) {
        let payload = serde_json::to_vec(record)
            .map_err(|error| OwnedJournalError::Integrity(error.to_string()))?;
        connection.execute(
            "INSERT INTO owned_curation_records(ordinal,id,payload) VALUES(?1,?2,?3)",
            params![ordinal as i64, record.id.to_string(), payload],
        )?;
    }
    let curation_revision = data.revisions.last().expect("length was checked");
    let payload = serde_json::to_vec(curation_revision)
        .map_err(|error| OwnedJournalError::Integrity(error.to_string()))?;
    connection.execute(
        "INSERT INTO owned_curation_revisions(revision_id,payload) VALUES(?1,?2)",
        params![revision.id.to_string(), payload],
    )?;
    let summary = OwnedCommitSummary {
        sequence,
        revision: revision.clone(),
        state_sha256: state_sha256.to_owned(),
    };
    let hash = curation_hash(
        source,
        &summary,
        &data.contexts,
        &data.records,
        &data.revisions,
    )?;
    connection.execute(
        "INSERT INTO owned_curation_prefixes(revision_id,contexts_length,records_length,curation_sha256) VALUES(?1,?2,?3,?4)",
        params![revision.id.to_string(), data.contexts.len() as i64, data.records.len() as i64, hash],
    )?;
    Ok(())
}

fn load_curation_document(
    connection: &Connection,
    summary: &OwnedCommitSummary,
    source: SourceId,
    dictionary: Dictionary,
) -> Result<OwnedCurationDocument, OwnedJournalError> {
    let prefix: (i64, i64, String) = connection.query_row(
        "SELECT contexts_length,records_length,curation_sha256 FROM owned_curation_prefixes WHERE revision_id=?1",
        [summary.revision.id.to_string()],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    ).map_err(|error| match error {
        rusqlite::Error::QueryReturnedNoRows => OwnedJournalError::Integrity("missing curation prefix".into()),
        other => other.into(),
    })?;
    let (contexts_len, records_len, expected_hash) = prefix;
    if contexts_len < 0 || records_len < 0 || !is_sha256(&expected_hash) {
        return Err(OwnedJournalError::Integrity(
            "malformed curation prefix".into(),
        ));
    }
    let contexts =
        read_curation_rows::<OwnedContextPin>(connection, "owned_context_pins", contexts_len)?;
    let records = read_curation_rows::<OwnedCurationRecord>(
        connection,
        "owned_curation_records",
        records_len,
    )?;
    let mut statement = connection.prepare(
        "SELECT r.revision_id,r.payload FROM owned_curation_revisions r JOIN owned_commits c ON c.revision_id=r.revision_id WHERE c.sequence<=?1 ORDER BY c.sequence",
    )?;
    let mut rows = statement.query([summary.sequence as i64])?;
    let mut revisions = Vec::new();
    while let Some(row) = rows.next()? {
        let row_id: String = row.get(0)?;
        let payload: Vec<u8> = row.get(1)?;
        let revision: OwnedCurationRevision =
            serde_json::from_slice(&payload).map_err(|error| {
                OwnedJournalError::Integrity(format!("malformed curation revision: {error}"))
            })?;
        if revision.revision.as_str() != row_id {
            return Err(OwnedJournalError::Integrity(
                "curation revision key differs from its payload".into(),
            ));
        }
        revisions.push(revision);
    }
    if revisions.len() as u64 != summary.sequence + 1 {
        return Err(OwnedJournalError::Integrity(
            "curation revision prefix is incomplete".into(),
        ));
    }
    if curation_hash(&source, summary, &contexts, &records, &revisions)? != expected_hash {
        return Err(OwnedJournalError::Integrity(
            "curation prefix hash differs".into(),
        ));
    }
    let document = OwnedCurationDocument::from_data(OwnedCurationData {
        version: 1,
        dictionary,
        contexts,
        records,
        revisions,
    })
    .map_err(OwnedJournalError::InvalidCuration)?;
    Ok(document)
}

/// Read and validate the complete logical history from one already-open connection.
fn snapshot_from_connection(connection: &Connection) -> Result<OwnedSnapshot, OwnedJournalError> {
    let (schema, source, head) = read_header_head(connection)?;
    let history = {
        let mut statement = connection.prepare(
            "SELECT revision_id,parent_revision,sequence,state_sha256 FROM owned_commits ORDER BY sequence",
        )?;
        let mut rows = statement.query([])?;
        let mut commits = Vec::new();
        let mut previous: Option<EntityId> = None;
        while let Some(row) = rows.next()? {
            let id_text: String = row.get(0)?;
            let id = EntityId::new(id_text)
                .map_err(|error| OwnedJournalError::Integrity(error.to_string()))?;
            let parent_text: Option<String> = row.get(1)?;
            let parent = parent_text
                .map(EntityId::new)
                .transpose()
                .map_err(|error| OwnedJournalError::Integrity(error.to_string()))?;
            let sequence: i64 = row.get(2)?;
            let hash: String = row.get(3)?;
            if sequence < 0
                || sequence as usize != commits.len()
                || parent != previous
                || !is_sha256(&hash)
            {
                return Err(OwnedJournalError::Integrity(
                    "owned commit chain is not contiguous".into(),
                ));
            }
            let revision = revision_for(connection, &id, sequence as u64)?;
            let mut roster = connection
                .prepare("SELECT section,length FROM owned_commit_sections WHERE revision_id=?1")?;
            let rows = roster.query_map([id.to_string()], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })?;
            let mut lengths = std::collections::BTreeMap::new();
            for item in rows {
                let (section, length) = item?;
                if !SECTIONS.contains(&section.as_str())
                    || length < 0
                    || lengths.insert(section, length as u64).is_some()
                {
                    return Err(OwnedJournalError::Integrity(
                        "owned commit has an invalid section roster".into(),
                    ));
                }
            }
            if lengths.len() != SECTIONS.len() {
                return Err(OwnedJournalError::Integrity(
                    "owned commit section roster is incomplete".into(),
                ));
            }
            let prefixes = SECTIONS
                .iter()
                .map(|section| OwnedSectionPrefix {
                    section: (*section).to_owned(),
                    length: lengths[*section],
                })
                .collect();
            commits.push(OwnedSnapshotCommit {
                revision: revision.id.clone(),
                state_sha256: hash,
                prefixes,
            });
            previous = Some(id);
        }
        if commits.is_empty() || commits.last().map(|row| &row.revision) != Some(&head.revision.id)
        {
            return Err(OwnedJournalError::Integrity(
                "current head does not match a nonempty commit history".into(),
            ));
        }
        let roster_count: i64 =
            connection.query_row("SELECT COUNT(*) FROM owned_commit_sections", [], |row| {
                row.get(0)
            })?;
        let expected_roster_count = commits
            .len()
            .checked_mul(SECTIONS.len())
            .ok_or_else(|| OwnedJournalError::Integrity("commit roster count overflow".into()))?;
        if roster_count < 0 || roster_count as usize != expected_roster_count {
            return Err(OwnedJournalError::Integrity(
                "owned commit section rows do not exactly cover committed revisions".into(),
            ));
        }
        commits
    };

    let dictionary = load_state(connection, &head)?;
    for section in SECTIONS {
        let expected = section_length_in_roster(history.last().expect("nonempty"), section)?;
        let actual: i64 = connection.query_row(
            "SELECT COUNT(*) FROM owned_records WHERE section=?1",
            [section],
            |row| row.get(0),
        )?;
        if u64::try_from(actual).ok() != Some(expected) {
            return Err(OwnedJournalError::Integrity(format!(
                "retained {section} rows do not match the final committed prefix"
            )));
        }
    }

    let curation = if schema == owned_schema::CURATION_SCHEMA_VERSION {
        let mut prefixes = Vec::with_capacity(history.len());
        for commit in &history {
            let lengths: (i64, i64, String) = connection.query_row(
                "SELECT contexts_length,records_length,curation_sha256 FROM owned_curation_prefixes WHERE revision_id=?1",
                [commit.revision.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            ).map_err(|error| match error {
                rusqlite::Error::QueryReturnedNoRows => OwnedJournalError::Integrity(
                    "missing curation prefix for committed revision".into(),
                ),
                other => other.into(),
            })?;
            if lengths.0 < 0 || lengths.1 < 0 || !is_sha256(&lengths.2) {
                return Err(OwnedJournalError::Integrity(
                    "malformed curation prefix".into(),
                ));
            }
            prefixes.push(OwnedSnapshotCurationPrefix {
                revision: commit.revision.clone(),
                contexts_length: lengths.0 as u64,
                records_length: lengths.1 as u64,
                curation_sha256: lengths.2,
            });
        }
        let last = prefixes.last().expect("one curation prefix per commit");
        let contexts = read_curation_rows::<OwnedContextPin>(
            connection,
            "owned_context_pins",
            last.contexts_length as i64,
        )?;
        let records = read_curation_rows::<OwnedCurationRecord>(
            connection,
            "owned_curation_records",
            last.records_length as i64,
        )?;
        let actual_contexts: i64 =
            connection.query_row("SELECT COUNT(*) FROM owned_context_pins", [], |row| {
                row.get(0)
            })?;
        let actual_records: i64 =
            connection.query_row("SELECT COUNT(*) FROM owned_curation_records", [], |row| {
                row.get(0)
            })?;
        let actual_revisions: i64 =
            connection.query_row("SELECT COUNT(*) FROM owned_curation_revisions", [], |row| {
                row.get(0)
            })?;
        let actual_prefixes: i64 =
            connection.query_row("SELECT COUNT(*) FROM owned_curation_prefixes", [], |row| {
                row.get(0)
            })?;
        if actual_contexts < 0
            || actual_records < 0
            || actual_revisions as usize != history.len()
            || actual_prefixes as usize != history.len()
            || actual_contexts as u64 != last.contexts_length
            || actual_records as u64 != last.records_length
        {
            return Err(OwnedJournalError::Integrity(
                "curation rows do not match the final committed prefixes".into(),
            ));
        }
        let mut revision_rows = connection.prepare(
            "SELECT r.revision_id,r.payload FROM owned_curation_revisions r JOIN owned_commits c ON c.revision_id=r.revision_id ORDER BY c.sequence",
        )?;
        let mut rows = revision_rows.query([])?;
        let mut curation_revisions = Vec::with_capacity(history.len());
        while let Some(row) = rows.next()? {
            let revision_id: String = row.get(0)?;
            let bytes: Vec<u8> = row.get(1)?;
            let revision: OwnedCurationRevision =
                serde_json::from_slice(&bytes).map_err(|error| {
                    OwnedJournalError::Integrity(format!("malformed curation revision: {error}"))
                })?;
            if revision.revision.as_str() != revision_id {
                return Err(OwnedJournalError::Integrity(
                    "curation revision key differs from its payload".into(),
                ));
            }
            curation_revisions.push(revision);
        }
        if curation_revisions.len() != history.len() {
            return Err(OwnedJournalError::Integrity(
                "curation revision rows do not cover every commit".into(),
            ));
        }
        Some(OwnedSnapshotCuration {
            version: 1,
            contexts,
            records,
            revisions: curation_revisions,
            prefixes,
        })
    } else {
        let sidecar_tables: i64 = connection.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('owned_context_pins','owned_curation_records','owned_curation_revisions','owned_curation_prefixes')",
            [],
            |row| row.get(0),
        )?;
        if sidecar_tables != 0 {
            return Err(OwnedJournalError::Integrity(
                "schema-one journal contains curation sidecar tables".into(),
            ));
        }
        None
    };
    OwnedSnapshot::from_data(OwnedSnapshotData {
        version: 1,
        journal_schema: schema as u32,
        source,
        head: head.revision.id,
        dictionary,
        commits: history,
        curation,
    })
    .map_err(|error| OwnedJournalError::Integrity(error.to_string()))
}

fn section_length_in_roster(
    commit: &OwnedSnapshotCommit,
    section: &str,
) -> Result<u64, OwnedJournalError> {
    commit
        .prefixes
        .iter()
        .find(|prefix| prefix.section == section)
        .map(|prefix| prefix.length)
        .ok_or_else(|| OwnedJournalError::Integrity("missing section prefix".into()))
}

fn import_rows_and_finish<'a>(
    connection: &'a Connection,
    snapshot: &OwnedSnapshot,
    guard: RollbackGuard<'a>,
) -> Result<(), OwnedJournalError> {
    let data = snapshot.as_data();
    let prepared = (|| {
        let rows = sections(&data.dictionary)?;
        let source_text = data.source.to_string();
        let head_text = data.head.to_string();
        for section in SECTIONS {
            for (ordinal, payload) in rows[section].iter().enumerate() {
                connection.execute(
                    "INSERT INTO owned_records(section,ordinal,payload) VALUES(?1,?2,?3)",
                    params![section, ordinal as i64, payload],
                )?;
            }
        }
        for (sequence, commit) in data.commits.iter().enumerate() {
            let revision = data
                .dictionary
                .metadata()
                .owned_revisions
                .get(sequence)
                .ok_or_else(|| {
                    OwnedJournalError::Integrity(
                        "snapshot commit has no matching final revision record".into(),
                    )
                })?;
            if revision.id != commit.revision {
                return Err(OwnedJournalError::Integrity(
                    "snapshot commit order differs from final revision records".into(),
                ));
            }
            let parent = revision.parent.as_ref().map(ToString::to_string);
            connection.execute(
                "INSERT INTO owned_commits(revision_id,parent_revision,sequence,state_sha256) VALUES(?1,?2,?3,?4)",
                params![commit.revision.to_string(), parent, sequence as i64, commit.state_sha256],
            )?;
            for prefix in &commit.prefixes {
                connection.execute(
                    "INSERT INTO owned_commit_sections(revision_id,section,length) VALUES(?1,?2,?3)",
                    params![commit.revision.to_string(), prefix.section, prefix.length as i64],
                )?;
            }
        }
        let updated = connection.execute(
            "UPDATE owned_meta SET schema_version=?1,source_id=?2,head_revision=?3,catalogue_revision=?4 WHERE singleton=1",
            params![
                data.journal_schema as i64,
                source_text,
                head_text,
                (data.commits.len() - 1) as i64,
            ],
        )?;
        if updated != 1 {
            return Err(OwnedJournalError::Integrity(
                "snapshot import did not update exactly one journal header".into(),
            ));
        }
        connection.pragma_update(None, "user_version", i64::from(data.journal_schema))?;
        if let Some(curation) = &data.curation {
            for (ordinal, pin) in curation.contexts.iter().enumerate() {
                let payload = serde_json::to_vec(pin)
                    .map_err(|error| OwnedJournalError::Integrity(error.to_string()))?;
                connection.execute(
                    "INSERT INTO owned_context_pins(ordinal,id,payload) VALUES(?1,?2,?3)",
                    params![ordinal as i64, pin.id.to_string(), payload],
                )?;
            }
            for (ordinal, record) in curation.records.iter().enumerate() {
                let payload = serde_json::to_vec(record)
                    .map_err(|error| OwnedJournalError::Integrity(error.to_string()))?;
                connection.execute(
                    "INSERT INTO owned_curation_records(ordinal,id,payload) VALUES(?1,?2,?3)",
                    params![ordinal as i64, record.id.to_string(), payload],
                )?;
            }
            for revision in &curation.revisions {
                let payload = serde_json::to_vec(revision)
                    .map_err(|error| OwnedJournalError::Integrity(error.to_string()))?;
                connection.execute(
                    "INSERT INTO owned_curation_revisions(revision_id,payload) VALUES(?1,?2)",
                    params![revision.revision.to_string(), payload],
                )?;
            }
            for prefix in &curation.prefixes {
                connection.execute(
                    "INSERT INTO owned_curation_prefixes(revision_id,contexts_length,records_length,curation_sha256) VALUES(?1,?2,?3,?4)",
                    params![
                        prefix.revision.to_string(),
                        prefix.contexts_length as i64,
                        prefix.records_length as i64,
                        prefix.curation_sha256,
                    ],
                )?;
            }
        }
        Ok::<_, OwnedJournalError>(())
    })();
    if let Err(error) = prepared {
        let result = rollback_error(connection, error);
        guard.disarm();
        return result;
    }
    match connection.execute_batch("COMMIT") {
        Ok(()) => {
            guard.disarm();
            Ok(())
        }
        Err(error) => {
            let result = resolve_commit_error(connection, error);
            guard.disarm();
            Err(result)
        }
    }
}

trait CurationRow: DeserializeOwned {
    fn identity(&self) -> &EntityId;
}

impl CurationRow for OwnedContextPin {
    fn identity(&self) -> &EntityId {
        &self.id
    }
}

impl CurationRow for OwnedCurationRecord {
    fn identity(&self) -> &EntityId {
        &self.id
    }
}

fn read_curation_rows<T: CurationRow>(
    connection: &Connection,
    table: &str,
    length: i64,
) -> Result<Vec<T>, OwnedJournalError> {
    let sql = match table {
        "owned_context_pins" | "owned_curation_records" => {
            format!("SELECT ordinal,id,payload FROM {table} WHERE ordinal<?1 ORDER BY ordinal")
        }
        _ => {
            return Err(OwnedJournalError::Integrity(
                "invalid curation table".into(),
            ));
        }
    };
    let mut statement = connection.prepare(&sql)?;
    let mut rows = statement.query([length])?;
    let mut values = Vec::new();
    while let Some(row) = rows.next()? {
        let ordinal: i64 = row.get(0)?;
        if ordinal < 0 || ordinal as usize != values.len() {
            return Err(OwnedJournalError::Integrity(
                "curation ordinals are not contiguous".into(),
            ));
        }
        let row_id: String = row.get(1)?;
        let payload: Vec<u8> = row.get(2)?;
        let value: T = serde_json::from_slice(&payload).map_err(|error| {
            OwnedJournalError::Integrity(format!("malformed curation row: {error}"))
        })?;
        if value.identity().as_str() != row_id {
            return Err(OwnedJournalError::Integrity(
                "curation identity key differs from its payload".into(),
            ));
        }
        values.push(value);
    }
    if values.len() as i64 != length {
        return Err(OwnedJournalError::Integrity(
            "curation prefix length differs from rows".into(),
        ));
    }
    Ok(values)
}
fn summary_for(
    connection: &Connection,
    id: &EntityId,
) -> Result<OwnedCommitSummary, OwnedJournalError> {
    let result = connection.query_row(
        "SELECT sequence,state_sha256,parent_revision FROM owned_commits WHERE revision_id=?1",
        [id.to_string()],
        |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
            ))
        },
    );
    let (sequence, hash, parent_text) = match result {
        Ok(v) => v,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return Err(OwnedJournalError::NotFound(id.clone()));
        }
        Err(e) => return Err(e.into()),
    };
    if sequence < 0 || !is_sha256(&hash) {
        return Err(OwnedJournalError::Integrity(
            "commit sequence/hash is malformed".into(),
        ));
    }
    let revision = revision_for(connection, id, sequence as u64)?;
    let parent = parent_text
        .map(EntityId::new)
        .transpose()
        .map_err(|e| OwnedJournalError::Integrity(e.to_string()))?;
    if revision.parent != parent {
        return Err(OwnedJournalError::Integrity(
            "typed revision parent differs from commit parent".into(),
        ));
    }
    Ok(OwnedCommitSummary {
        sequence: sequence as u64,
        revision,
        state_sha256: hash,
    })
}
fn revision_for(
    connection: &Connection,
    id: &EntityId,
    sequence: u64,
) -> Result<OwnedRevisionRecord, OwnedJournalError> {
    let ordinal = i64::try_from(sequence).map_err(|_| {
        OwnedJournalError::Integrity("revision ordinal exceeds SQLite range".into())
    })?;
    let payload: Vec<u8> = connection.query_row(
        "SELECT payload FROM owned_records WHERE section='metadata.owned_revisions' AND ordinal=?1",
        [ordinal],
        |row| row.get(0),
    )?;
    let revision: OwnedRevisionRecord = serde_json::from_slice(&payload)
        .map_err(|e| OwnedJournalError::Integrity(e.to_string()))?;
    let (commit_seq, parent_text): (i64, Option<String>) = connection.query_row(
        "SELECT sequence,parent_revision FROM owned_commits WHERE revision_id=?1",
        [id.to_string()],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let mut roster_statement = connection.prepare(
        "SELECT section FROM owned_commit_sections WHERE revision_id=?1 ORDER BY section",
    )?;
    let roster_iter =
        roster_statement.query_map([id.to_string()], |row| row.get::<_, String>(0))?;
    let roster = roster_iter.collect::<Result<Vec<_>, _>>()?;
    let mut expected: Vec<_> = SECTIONS.iter().map(|s| s.to_string()).collect();
    expected.sort();
    let header_source: String = connection.query_row(
        "SELECT source_id FROM owned_meta WHERE singleton=1",
        [],
        |r| r.get(0),
    )?;
    let commit_parent = parent_text
        .map(EntityId::new)
        .transpose()
        .map_err(|e| OwnedJournalError::Integrity(e.to_string()))?;
    if commit_seq < 0
        || commit_seq as u64 != sequence
        || revision.id != *id
        || roster != expected
        || revision.source.to_string() != header_source
        || revision.parent != commit_parent
        || (sequence == 0 && revision.parent.is_some())
        || (sequence > 0 && revision.parent.is_none())
    {
        return Err(OwnedJournalError::Integrity(
            "revision summary disagrees with commit metadata".into(),
        ));
    }
    Ok(revision)
}
fn current_view(connection: &Connection) -> Result<OwnedRevisionView, OwnedJournalError> {
    let (_, summary) = read_current_summary(connection)?;
    let dictionary = load_state(connection, &summary)?;
    let (source, _) = read_current_summary(connection)?;
    Ok(OwnedRevisionView {
        version: 1,
        source,
        commit: summary,
        dictionary,
    })
}

fn read_section<T: DeserializeOwned>(
    connection: &Connection,
    section: &str,
    revision: &EntityId,
) -> Result<Vec<T>, OwnedJournalError> {
    let len: i64 = match connection.query_row(
        "SELECT length FROM owned_commit_sections WHERE revision_id=?1 AND section=?2",
        params![revision.to_string(), section],
        |r| r.get(0),
    ) {
        Ok(length) => length,
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            return Err(OwnedJournalError::Integrity(format!(
                "missing {section} commit prefix"
            )));
        }
        Err(error) => return Err(error.into()),
    };
    if len < 0 {
        return Err(OwnedJournalError::Integrity(format!(
            "negative {section} length"
        )));
    }
    let mut statement=connection.prepare("SELECT ordinal,payload FROM owned_records WHERE section=?1 AND ordinal<?2 ORDER BY ordinal")?;
    let mut rows = statement.query(params![section, len])?;
    let mut out = Vec::new();
    while let Some(row) = rows.next()? {
        let ordinal: i64 = row.get(0)?;
        if ordinal as usize != out.len() {
            return Err(OwnedJournalError::Integrity(format!(
                "{section} ordinals are not contiguous"
            )));
        }
        let bytes: Vec<u8> = row.get(1)?;
        out.push(serde_json::from_slice(&bytes).map_err(|e| {
            OwnedJournalError::Integrity(format!("{section} row is malformed: {e}"))
        })?);
    }
    if out.len() as i64 != len {
        return Err(OwnedJournalError::Integrity(format!(
            "{section} prefix length differs from rows"
        )));
    }
    Ok(out)
}
fn load_state(
    connection: &Connection,
    summary: &OwnedCommitSummary,
) -> Result<Dictionary, OwnedJournalError> {
    let id = &summary.revision.id;
    validate_record_sections(connection)?;
    let registry = RegistryDocument {
        version: 1,
        sources: read_section(connection, "registry.sources", id)?,
        contributors: read_section(connection, "registry.contributors", id)?,
        distributors: read_section(connection, "registry.distributors", id)?,
        aliases: read_section(connection, "registry.aliases", id)?,
        lineage: read_section(connection, "registry.lineage", id)?,
        snapshots: read_section(connection, "registry.snapshots", id)?,
        occurrences: read_section(connection, "registry.occurrences", id)?,
    };
    let doc = DictionaryDocument {
        version: 1,
        legacy_archive: None,
        registry: SourceRegistry::from_document(registry)
            .map_err(|e| OwnedJournalError::Integrity(e.to_string()))?,
        captures: read_section(connection, "captures", id)?,
        metadata: DictionaryMetadata {
            catalogue_revision: summary.sequence,
            code_notices: read_section(connection, "metadata.code_notices", id)?,
            collections: read_section(connection, "metadata.collections", id)?,
            components: read_section(connection, "metadata.components", id)?,
            releases: read_section(connection, "metadata.releases", id)?,
            entries: read_section(connection, "metadata.entries", id)?,
            assertions: read_section(connection, "metadata.assertions", id)?,
            contexts: read_section(connection, "metadata.contexts", id)?,
            frames: read_section(connection, "metadata.frames", id)?,
            forms: read_section(connection, "metadata.forms", id)?,
            etymologies: read_section(connection, "metadata.etymologies", id)?,
            relations: read_section(connection, "metadata.relations", id)?,
            owned_revisions: read_section(connection, "metadata.owned_revisions", id)?,
            owned_heads: vec![OwnedHead {
                source: summary.revision.source.clone(),
                revision: id.clone(),
            }],
            editorial_senses: read_section(connection, "metadata.editorial_senses", id)?,
            computations: read_section(connection, "metadata.computations", id)?,
            annotations: read_section(connection, "metadata.annotations", id)?,
            extensions: read_section(connection, "metadata.extensions", id)?,
        },
    };
    let dictionary = Dictionary::from_document(doc)
        .map_err(|error| OwnedJournalError::Integrity(error.to_string()))?;
    if state_hash(&dictionary)? != summary.state_sha256 {
        return Err(OwnedJournalError::Integrity(
            "reconstructed canonical state hash differs from commit".into(),
        ));
    }
    validate_current_section_lengths(connection, summary)?;
    Ok(dictionary)
}

fn validate_record_sections(connection: &Connection) -> Result<(), OwnedJournalError> {
    let known: std::collections::HashSet<&str> = SECTIONS.iter().copied().collect();
    let mut statement = connection.prepare("SELECT DISTINCT section FROM owned_records")?;
    let rows = statement.query_map([], |row| row.get::<_, String>(0))?;
    for row in rows {
        let section = row?;
        if !known.contains(section.as_str()) {
            return Err(OwnedJournalError::Integrity(format!(
                "unknown owned record section {section}"
            )));
        }
    }
    Ok(())
}

fn validate_current_section_lengths(
    connection: &Connection,
    summary: &OwnedCommitSummary,
) -> Result<(), OwnedJournalError> {
    let head: String = connection.query_row(
        "SELECT head_revision FROM owned_meta WHERE singleton=1",
        [],
        |row| row.get(0),
    )?;
    if head != summary.revision.id.as_str() {
        return Ok(());
    }
    let mut statement = connection.prepare(
        "SELECT section,length FROM owned_commit_sections WHERE revision_id=?1 ORDER BY section",
    )?;
    let rows = statement.query_map([head], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    })?;
    for row in rows {
        let (section, expected) = row?;
        let actual: i64 = connection.query_row(
            "SELECT COUNT(*) FROM owned_records WHERE section=?1",
            [&section],
            |row| row.get(0),
        )?;
        if actual != expected {
            return Err(OwnedJournalError::Integrity(format!(
                "current {section} row count {actual} differs from committed length {expected}"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod import_snapshot_tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use vlajmaji_lexicon::{
        AbbreviationKind, Availability, CollectionRecord, ComponentRecord, ContentNotice,
        ContributorId, ContributorRecord, DictionaryDocument, Presence, RedistributionStatus,
        RegistryDocument, ReleaseAccounting, ReleaseRecord, SourceRecord, TextValue,
    };

    static NEXT_ROOT: AtomicUsize = AtomicUsize::new(0);

    fn schema_objects(
        connection: &Connection,
        schema: &str,
    ) -> Vec<(String, String, String, Option<String>)> {
        let sql = format!(
            "SELECT type,name,tbl_name,sql FROM {schema}.sqlite_master \
             WHERE name NOT LIKE 'sqlite_%' ORDER BY type,name"
        );
        let mut statement = connection.prepare(&sql).unwrap();
        statement
            .query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    }

    fn abort_snapshot() -> OwnedSnapshot {
        let source = SourceId::new("late-abort-source").unwrap();
        let editor = ContributorId::new("late-abort-editor").unwrap();
        let collection = EntityId::new("late-abort-collection").unwrap();
        let component = EntityId::new("late-abort-component").unwrap();
        let genesis_id = EntityId::new("late-abort-r0").unwrap();
        let mut metadata = DictionaryMetadata::default();
        metadata.collections.push(CollectionRecord {
            id: collection.clone(),
            source: source.clone(),
            display_name: "Late abort collection".into(),
            description: Presence::Missing,
        });
        metadata.components.push(ComponentRecord {
            id: component.clone(),
            collection,
            native_name: "Late abort component".into(),
            languages: vec!["jbo".into()],
        });
        metadata.releases.push(ReleaseRecord {
            id: EntityId::new("late-abort-release").unwrap(),
            component,
            distributor: None,
            upstream_revision: Presence::Missing,
            distributor_revision: Presence::Missing,
            captured_at: Presence::Missing,
            time_precision: vlajmaji_lexicon::TimePrecision::Unknown,
            schema_version: 1,
            normalizer: "test".into(),
            normalizer_version: "1".into(),
            captures: Vec::new(),
            assertions: Vec::new(),
            accounting: ReleaseAccounting {
                input_occurrences: Some(0),
                normalized_occurrences: Some(0),
                query_excluded_occurrences: Some(0),
                unit: "occurrences".into(),
            },
            diagnostics: Vec::new(),
            selected: true,
            availability: Availability::Available,
            content_notices: vec![ContentNotice {
                text: Presence::Value("test-only".into()),
                locator: Presence::Missing,
                license: Presence::Missing,
            }],
            redistribution: RedistributionStatus::Held,
            redistribution_evidence: Vec::new(),
        });
        metadata.owned_revisions.push(OwnedRevisionRecord {
            id: genesis_id.clone(),
            source: source.clone(),
            parent: None,
            editor: editor.clone(),
            timestamp: Presence::Missing,
            time_precision: vlajmaji_lexicon::TimePrecision::Unknown,
            assertions: Vec::new(),
            tombstones: Vec::new(),
            rationale: Presence::Value(TextValue {
                text: "genesis".into(),
                language: Presence::Missing,
                markup: Presence::Missing,
            }),
        });
        metadata.owned_heads.push(OwnedHead {
            source: source.clone(),
            revision: genesis_id.clone(),
        });
        let registry = SourceRegistry::from_document(RegistryDocument {
            sources: vec![SourceRecord {
                id: source.clone(),
                display_name: "Late abort source".into(),
                abbreviation: "LAB".into(),
                abbreviation_kind: AbbreviationKind::ProjectAssigned,
                abbreviation_evidence: None,
                origin: SourceOrigin::OwnedDraft,
            }],
            contributors: vec![ContributorRecord {
                id: editor.clone(),
                display_name: "Test editor".into(),
            }],
            ..RegistryDocument::default()
        })
        .unwrap();
        let genesis = Dictionary::from_document(DictionaryDocument {
            registry,
            metadata,
            ..DictionaryDocument::default()
        })
        .unwrap();
        let mut final_document = genesis.document();
        let head_id = EntityId::new("late-abort-r1").unwrap();
        final_document.metadata.catalogue_revision = 1;
        final_document
            .metadata
            .owned_revisions
            .push(OwnedRevisionRecord {
                id: head_id.clone(),
                source: source.clone(),
                parent: Some(genesis_id.clone()),
                editor,
                timestamp: Presence::Missing,
                time_precision: vlajmaji_lexicon::TimePrecision::Unknown,
                assertions: Vec::new(),
                tombstones: Vec::new(),
                rationale: Presence::Value(TextValue {
                    text: "second commit".into(),
                    language: Presence::Missing,
                    markup: Presence::Missing,
                }),
            });
        final_document.metadata.owned_heads[0].revision = head_id.clone();
        let final_dictionary = Dictionary::from_document(final_document).unwrap();
        let genesis_sections = sections(&genesis).unwrap();
        let final_sections = sections(&final_dictionary).unwrap();
        let final_hash = state_hash(&final_dictionary).unwrap();
        let prefixes = |rows: &Sections| {
            SECTIONS
                .iter()
                .map(|section| OwnedSectionPrefix {
                    section: (*section).to_owned(),
                    length: rows[*section].len() as u64,
                })
                .collect::<Vec<_>>()
        };
        OwnedSnapshot::from_data(OwnedSnapshotData {
            version: 1,
            journal_schema: 1,
            source,
            head: head_id,
            dictionary: final_dictionary,
            commits: vec![
                OwnedSnapshotCommit {
                    revision: genesis_id,
                    state_sha256: state_hash(&genesis).unwrap(),
                    prefixes: prefixes(&genesis_sections),
                },
                OwnedSnapshotCommit {
                    revision: EntityId::new("late-abort-r1").unwrap(),
                    state_sha256: final_hash,
                    prefixes: prefixes(&final_sections),
                },
            ],
            curation: None,
        })
        .unwrap()
    }

    #[test]
    fn late_sequence_one_import_failure_rolls_back_then_explicit_import_succeeds() {
        let snapshot = abort_snapshot();
        let root = std::env::temp_dir().join(format!(
            "vlajmaji-owned-import-late-{}-{}",
            std::process::id(),
            NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let connection = Connection::open(root.join(owned_schema::FILE)).unwrap();
        owned_schema::configure(&connection).unwrap();
        let empty_schema = (
            schema_objects(&connection, "main"),
            schema_objects(&connection, "temp"),
            connection
                .pragma_query_value(None, "user_version", |row| row.get::<_, i32>(0))
                .unwrap(),
            connection
                .pragma_query_value(None, "application_id", |row| row.get::<_, i32>(0))
                .unwrap(),
        );
        connection.execute_batch("BEGIN IMMEDIATE").unwrap();
        let guard = RollbackGuard::new(&connection, None);
        owned_schema::initialize(&connection).unwrap();
        connection
            .execute_batch(
                "CREATE TEMP TRIGGER reject_late_import
                 BEFORE INSERT ON owned_commits
                 WHEN NEW.sequence=1 AND EXISTS(SELECT 1 FROM owned_commits WHERE sequence=0)
                 BEGIN SELECT RAISE(ABORT,'owned-import-late-sequence-one'); END;",
            )
            .unwrap();
        let error = import_rows_and_finish(&connection, &snapshot, guard).unwrap_err();
        assert_eq!(error.category(), OwnedJournalErrorCategory::Backend);
        assert!(error.to_string().contains("owned-import-late-sequence-one"));
        assert!(connection.is_autocommit());
        let rolled_back_schema = (
            schema_objects(&connection, "main"),
            schema_objects(&connection, "temp"),
            connection
                .pragma_query_value(None, "user_version", |row| row.get::<_, i32>(0))
                .unwrap(),
            connection
                .pragma_query_value(None, "application_id", |row| row.get::<_, i32>(0))
                .unwrap(),
        );
        assert_eq!(rolled_back_schema, empty_schema);

        connection.execute_batch("BEGIN IMMEDIATE").unwrap();
        let guard = RollbackGuard::new(&connection, None);
        owned_schema::initialize(&connection).unwrap();
        import_rows_and_finish(&connection, &snapshot, guard).unwrap();
        check_header(&connection).unwrap();
        assert_eq!(snapshot_from_connection(&connection).unwrap(), snapshot);
    }

    #[test]
    fn test_revision_identity_checks_local_summary_and_schema_two_prefix() {
        let snapshot = abort_snapshot();
        let root = std::env::temp_dir().join(format!(
            "vlajmaji-owned-identity-{}-{}",
            std::process::id(),
            NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
        ));
        let (journal, imported) = OwnedJournal::import_snapshot(&root, &snapshot).unwrap();
        drop(journal);

        let reader = OwnedJournalReader::open(&root).unwrap();
        let head = reader.head_identity().unwrap();
        assert_eq!(head.journal_schema, 1);
        assert_eq!(head.source, snapshot.as_data().source);
        assert_eq!(head.revision, imported.head);
        assert_eq!(head.sequence, 1);
        assert_eq!(
            head.state_sha256,
            snapshot.as_data().commits[1].state_sha256
        );
        assert_eq!(head.curation_state_sha256, None);

        let genesis_id = &snapshot.as_data().commits[0].revision;
        let genesis = reader.revision_identity(genesis_id).unwrap();
        assert_eq!(genesis.journal_schema, 1);
        assert_eq!(genesis.revision, *genesis_id);
        assert_eq!(genesis.sequence, 0);
        assert_eq!(
            genesis.state_sha256,
            snapshot.as_data().commits[0].state_sha256
        );
        assert_eq!(genesis.curation_state_sha256, None);
        let missing = EntityId::new("absent-owned-identity").unwrap();
        assert!(matches!(
            reader.revision_identity(&missing),
            Err(OwnedJournalError::NotFound(id)) if id == missing
        ));
        drop(reader);

        let mut writer = OwnedJournal::open(&root).unwrap();
        writer.migrate_curation(&imported.head).unwrap();
        drop(writer);
        let reader = OwnedJournalReader::open(&root).unwrap();
        let migrated = reader.head_identity().unwrap();
        assert_eq!(migrated.journal_schema, 2);
        assert_eq!(migrated.revision, imported.head);
        assert_eq!(migrated.sequence, 1);
        assert!(migrated.curation_state_sha256.is_some());
        assert_eq!(reader.revision_identity(&imported.head).unwrap(), migrated);
        drop(reader);

        let connection = Connection::open(root.join(owned_schema::FILE)).unwrap();
        let saved_prefix: (i64, i64, String) = connection
            .query_row(
                "SELECT contexts_length,records_length,curation_sha256 FROM owned_curation_prefixes WHERE revision_id=?1",
                [imported.head.to_string()],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        connection
            .execute(
                "DELETE FROM owned_curation_prefixes WHERE revision_id=?1",
                [imported.head.to_string()],
            )
            .unwrap();
        let reader = OwnedJournalReader::open(&root).unwrap();
        assert!(matches!(
            reader.head_identity(),
            Err(OwnedJournalError::Integrity(message)) if message == "missing curation prefix"
        ));
        drop(reader);
        connection
            .execute(
                "INSERT INTO owned_curation_prefixes(revision_id,contexts_length,records_length,curation_sha256) VALUES(?1,?2,?3,?4)",
                params![imported.head.to_string(), saved_prefix.0, saved_prefix.1, saved_prefix.2],
            )
            .unwrap();

        connection
            .execute(
                "UPDATE owned_curation_prefixes SET curation_sha256='not-a-sha256' WHERE revision_id=?1",
                [imported.head.to_string()],
            )
            .unwrap();
        let reader = OwnedJournalReader::open(&root).unwrap();
        assert!(matches!(
            reader.head_identity(),
            Err(OwnedJournalError::Integrity(message)) if message == "malformed curation prefix"
        ));
        drop(reader);
        connection
            .execute(
                "UPDATE owned_curation_prefixes SET curation_sha256=?1 WHERE revision_id=?2",
                params![saved_prefix.2, imported.head.to_string()],
            )
            .unwrap();

        connection
            .execute(
                "UPDATE owned_commits SET state_sha256='not-a-sha256' WHERE revision_id=?1",
                [imported.head.to_string()],
            )
            .unwrap();
        let reader = OwnedJournalReader::open(&root).unwrap();
        assert!(matches!(
            reader.head_identity(),
            Err(OwnedJournalError::Integrity(message)) if message == "commit sequence/hash is malformed"
        ));
        drop(reader);
        connection
            .execute(
                "UPDATE owned_commits SET state_sha256=?1 WHERE revision_id=?2",
                params![
                    snapshot.as_data().commits[1].state_sha256,
                    imported.head.to_string()
                ],
            )
            .unwrap();

        connection
            .execute(
                "UPDATE owned_meta SET schema_version=1 WHERE singleton=1",
                [],
            )
            .unwrap();
        assert!(matches!(
            OwnedJournalReader::open(&root),
            Err(OwnedJournalError::UnsupportedVersion { found: 2 })
        ));
        connection
            .execute(
                "UPDATE owned_meta SET schema_version=2 WHERE singleton=1",
                [],
            )
            .unwrap();
        let reader = OwnedJournalReader::open(&root).unwrap();
        assert_eq!(reader.head_identity().unwrap(), migrated);
        drop(reader);
        drop(connection);
        fs::remove_dir_all(root).unwrap();
    }
}
