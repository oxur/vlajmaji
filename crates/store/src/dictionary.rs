//! Read-only persistence for the versioned [`Dictionary`](vlajmaji_lexicon::Dictionary),
//! with explicit verification of persisted integrity boundaries.
//!
//! A store owns a read-only SQLite connection and a minimally checked header.
//! Comprehensive schema, byte, model, foreign-key, and projection checks run
//! only when explicitly requested. It does not migrate or repair files.

use std::{
    error::Error,
    fmt, fs, io,
    path::{Path, PathBuf},
};

use rusqlite::{Connection, OpenFlags, types::Value};
use vlajmaji_lexicon::{
    Dictionary, DictionaryDocument, DictionaryError, RegistryDocument, SourceRecord,
};

use crate::dictionary_schema::{self, DATABASE_FILE};

/// A typed reason that an existing dictionary store failed a specific boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum DictionaryStoreInvalidity {
    /// The expected metadata or canonical-document row is missing or malformed.
    MissingDatabase,
    /// The file belongs to another store family.
    Format,
    /// The dictionary-store schema version is unsupported.
    SchemaVersion,
    /// The store has not committed its complete marker.
    Incomplete,
    /// Canonical bytes or their digest are absent or inconsistent.
    CanonicalBytes,
    /// The database schema differs from the exact expected schema.
    SchemaShape,
    /// SQLite reports a foreign-key violation.
    ForeignKeys,
    /// Relational rows or cells differ from the canonical model projection.
    Projection,
}

/// Failure to create, minimally open, or explicitly read/verify a dictionary store.
#[derive(Debug)]
#[non_exhaustive]
pub enum DictionaryStoreError {
    /// The destination already exists; creation never replaces it.
    ExistingPath(PathBuf),
    /// Canonical model bytes did not parse as a valid dictionary.
    InvalidDictionary(DictionaryError),
    /// The existing database violates a named format or integrity boundary.
    InvalidStore {
        /// The boundary that failed.
        reason: DictionaryStoreInvalidity,
        /// A concise diagnostic.
        detail: String,
    },
    /// SQLite failed while reading or writing the store.
    Backend(rusqlite::Error),
    /// The filesystem failed while creating or resolving the store.
    Io(io::Error),
}

impl fmt::Display for DictionaryStoreInvalidity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::Display for DictionaryStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ExistingPath(path) => write!(
                f,
                "dictionary store path already exists: {}",
                path.display()
            ),
            Self::InvalidDictionary(error) => write!(f, "invalid dictionary: {error}"),
            Self::InvalidStore { reason, detail } => {
                write!(f, "invalid dictionary store ({reason}): {detail}")
            }
            Self::Backend(error) => write!(f, "dictionary store backend error: {error}"),
            Self::Io(error) => write!(f, "dictionary store filesystem error: {error}"),
        }
    }
}

impl Error for DictionaryStoreError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidDictionary(error) => Some(error),
            Self::Backend(error) => Some(error),
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for DictionaryStoreError {
    fn from(value: rusqlite::Error) -> Self {
        Self::Backend(value)
    }
}
impl From<io::Error> for DictionaryStoreError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

/// A read-only dictionary store held on one minimally checked SQLite snapshot.
pub struct DictionaryStore {
    path: PathBuf,
    database_path: PathBuf,
    _connection: Connection,
    metadata: std::collections::BTreeMap<String, String>,
}

/// SQL-derived counts and source rows from the store snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictionaryStoreSummary {
    /// Number of persisted captures.
    pub captures: usize,
    /// Number of persisted raw occurrences.
    pub occurrences: usize,
    /// Number of persisted dictionary entries.
    pub entries: usize,
    /// Number of persisted assertions.
    pub assertions: usize,
    /// Source rows in their persisted ordinal order.
    pub sources: Vec<SourceRecord>,
}

impl fmt::Debug for DictionaryStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DictionaryStore")
            .field("path", &self.path)
            .field("format", &dictionary_schema::FORMAT)
            .finish_non_exhaustive()
    }
}

impl DictionaryStore {
    pub(crate) fn runtime_attached_limit(&self) -> Result<i32, rusqlite::Error> {
        self._connection
            .limit(rusqlite::limits::Limit::SQLITE_LIMIT_ATTACHED)
    }

    /// Validate a dictionary, build a fresh store under an existing parent, run
    /// every verification check, then return its read-only snapshot.
    ///
    /// The destination leaf must not exist, even as a dangling symlink. Creation
    /// never merges, replaces, repairs, or deletes caller-owned paths. A failed
    /// build is not reused; callers should inspect or remove their own failed path.
    ///
    /// # Errors
    ///
    /// Returns [`DictionaryStoreError::ExistingPath`] for any existing leaf,
    /// [`DictionaryStoreError::InvalidDictionary`] for a failed canonical roundtrip,
    /// or a backend/filesystem error for creation and validation failures.
    pub fn create(
        path: impl AsRef<Path>,
        dictionary: &Dictionary,
    ) -> Result<Self, DictionaryStoreError> {
        let path = path.as_ref().to_path_buf();
        match fs::symlink_metadata(&path) {
            Ok(_) => return Err(DictionaryStoreError::ExistingPath(path)),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(DictionaryStoreError::Io(error)),
        }
        let canonical = dictionary
            .to_json_bytes()
            .map_err(DictionaryStoreError::InvalidDictionary)?;
        let parsed = Dictionary::from_json_bytes(&canonical)
            .map_err(DictionaryStoreError::InvalidDictionary)?;
        if parsed != *dictionary {
            return Err(invalid(
                DictionaryStoreInvalidity::CanonicalBytes,
                "canonical dictionary roundtrip changed the validated model",
            ));
        }
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        if !parent.is_dir() {
            return Err(DictionaryStoreError::Io(io::Error::new(
                io::ErrorKind::NotFound,
                "dictionary store parent directory must already exist",
            )));
        }
        fs::create_dir(&path).map_err(|error| {
            if error.kind() == io::ErrorKind::AlreadyExists {
                DictionaryStoreError::ExistingPath(path.clone())
            } else {
                DictionaryStoreError::Io(error)
            }
        })?;
        let database_path = path.join(DATABASE_FILE);
        let result = build(&database_path, dictionary, &canonical);
        result?;
        let store = Self::open(&path)?;
        let report = crate::DictionaryStoreVerification::verify(
            &path,
            crate::DictionaryStoreVerificationCheck::All,
        )?;
        if !report.ok {
            return Err(invalid(
                DictionaryStoreInvalidity::Projection,
                "newly created dictionary store failed complete verification",
            ));
        }
        Ok(store)
    }

    /// Open an existing dictionary store after checking its minimum readable header.
    ///
    /// The returned borrow and SQL state refer to the read transaction pinned by
    /// the first metadata read; dropping the store releases that snapshot.
    ///
    /// # Errors
    ///
    /// This does not verify the schema, canonical bytes, model, hashes, foreign
    /// keys, or relational projection. Use [`crate::DictionaryStoreVerification`]
    /// for those explicit checks.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, DictionaryStoreError> {
        let path = path.as_ref().to_path_buf();
        let database_path = path.join(DATABASE_FILE);
        if !database_path.is_file() {
            return Err(invalid(
                DictionaryStoreInvalidity::MissingDatabase,
                format!("expected {}", database_path.display()),
            ));
        }
        let connection = Connection::open_with_flags(
            &database_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )?;
        connection.execute_batch("PRAGMA foreign_keys=ON; BEGIN DEFERRED;")?;
        let metadata = read_metadata(&connection)?;
        check_minimal_metadata(&metadata)?;
        Ok(Self {
            path,
            database_path,
            _connection: connection,
            metadata,
        })
    }

    /// Load and strictly validate the canonical model on explicit request.
    ///
    /// This validates the model graph but does not certify the store hashes,
    /// schema, foreign keys, or relational projection.
    ///
    /// # Errors
    ///
    /// Returns a backend or canonical-model validation error.
    pub fn dictionary(&self) -> Result<Dictionary, DictionaryStoreError> {
        let (bytes, _) = read_canonical(&self._connection)?;
        Dictionary::from_json_bytes(&bytes).map_err(DictionaryStoreError::InvalidDictionary)
    }

    /// Read the canonical JSON bytes on explicit request.
    ///
    /// Reading these bytes does not validate their hash or model.
    ///
    /// # Errors
    ///
    /// Returns a backend or malformed/missing canonical-row error.
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, DictionaryStoreError> {
        read_canonical(&self._connection).map(|(bytes, _)| bytes)
    }

    /// Load the typed canonical draft without claiming whole-model validation.
    ///
    /// # Errors
    ///
    /// Returns a backend, missing-row, or typed-document parsing error.
    pub fn unverified_document(
        &self,
    ) -> Result<DictionaryDocument<RegistryDocument>, DictionaryStoreError> {
        let (bytes, _) = read_canonical(&self._connection)?;
        DictionaryDocument::from_unverified_json_bytes(&bytes)
            .map_err(DictionaryStoreError::InvalidDictionary)
    }

    pub(crate) fn unverified_document_with_legacy_archive(
        &self,
    ) -> Result<(Option<DictionaryDocument<RegistryDocument>>, bool), DictionaryStoreError> {
        let (bytes, legacy) = read_canonical(&self._connection)?;
        if legacy.is_some() {
            return Ok((None, true));
        }
        let document = DictionaryDocument::from_unverified_json_bytes(&bytes)
            .map_err(DictionaryStoreError::InvalidDictionary)?;
        Ok((Some(document), false))
    }

    /// Return the recorded canonical digest from the minimally checked header.
    /// This value is not authenticated until the `hashes` check passes.
    pub fn recorded_state_sha256(&self) -> &str {
        self.metadata
            .get("canonical_sha256")
            .map(String::as_str)
            .unwrap_or_default()
    }

    /// Return persisted summary values without loading the canonical model.
    ///
    /// # Errors
    ///
    /// Returns a backend error or a projection error for malformed source rows.
    pub fn summary(&self) -> Result<DictionaryStoreSummary, DictionaryStoreError> {
        let count = |table: &str| -> Result<usize, DictionaryStoreError> {
            let sql = format!("SELECT count(*) FROM {table}");
            let count: i64 = self._connection.query_row(&sql, [], |row| row.get(0))?;
            usize::try_from(count)
                .map_err(|error| DictionaryStoreError::Io(io::Error::other(error.to_string())))
        };
        let mut statement = self._connection.prepare(
            "SELECT id,display_name,abbreviation,abbreviation_kind,abbreviation_evidence,origin FROM sources ORDER BY ordinal",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, String>(5)?,
            ))
        })?;
        let mut sources = Vec::new();
        for row in rows {
            let (id, display_name, abbreviation, kind, evidence, origin) = row?;
            sources.push(SourceRecord {
                id: vlajmaji_lexicon::SourceId::new(id).map_err(|error| {
                    invalid(DictionaryStoreInvalidity::Projection, error.to_string())
                })?,
                display_name,
                abbreviation,
                abbreviation_kind: match kind.as_str() {
                    "Conventional" => vlajmaji_lexicon::AbbreviationKind::Conventional,
                    "ProjectAssigned" => vlajmaji_lexicon::AbbreviationKind::ProjectAssigned,
                    _ => {
                        return Err(invalid(
                            DictionaryStoreInvalidity::Projection,
                            "unknown source abbreviation kind",
                        ));
                    }
                },
                abbreviation_evidence: evidence,
                origin: match origin.as_str() {
                    "Upstream" => vlajmaji_lexicon::SourceOrigin::Upstream,
                    "OwnedDraft" => vlajmaji_lexicon::SourceOrigin::OwnedDraft,
                    _ => {
                        return Err(invalid(
                            DictionaryStoreInvalidity::Projection,
                            "unknown source origin",
                        ));
                    }
                },
            });
        }
        Ok(DictionaryStoreSummary {
            captures: count("raw_captures")?,
            occurrences: count("raw_occurrences")?,
            entries: count("entries")?,
            assertions: count("assertions")?,
            sources,
        })
    }

    /// Return the caller-owned store directory.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Return the concrete SQLite database path for independent read-only inspection.
    #[must_use]
    pub fn database_path(&self) -> PathBuf {
        self.database_path.clone()
    }

    /// Return stable assertion entity IDs whose entry spelling is an exact
    /// UTF-8 byte match and whose assertion is lexically eligible.
    ///
    /// Candidate order is the binary order of source ID, spelling, and the
    /// occurrence tuple ID. The returned values are assertion IDs, not
    /// occurrence tuple IDs. This query uses the validated snapshot held
    /// by this store; it does not reopen the database or normalize the query.
    ///
    /// # Errors
    ///
    /// Returns the underlying SQLite error, or `InvalidStore(Projection)` if a
    /// persisted candidate ID cannot be represented by the dictionary model.
    pub fn exact_assertion_ids(
        &self,
        spelling: &str,
    ) -> Result<Vec<vlajmaji_lexicon::EntityId>, DictionaryStoreError> {
        let mut statement = self._connection.prepare(
            "SELECT a.id FROM entries AS e \
             JOIN assertions AS a ON a.entry_id = e.id \
             WHERE e.spelling_state = 'value' \
               AND e.spelling_value = ?1 COLLATE BINARY \
               AND a.eligibility = 'Lexical' \
             ORDER BY e.source_id COLLATE BINARY, \
                      e.spelling_value COLLATE BINARY, \
                      a.occurrence_id COLLATE BINARY",
        )?;
        let mut rows = statement.query([spelling])?;
        let mut ids = Vec::new();
        while let Some(row) = rows.next()? {
            let raw: String = row.get(0)?;
            let id = vlajmaji_lexicon::EntityId::new(raw).map_err(|error| {
                invalid(
                    DictionaryStoreInvalidity::Projection,
                    format!("exact lexical candidate has invalid assertion ID: {error}"),
                )
            })?;
            ids.push(id);
        }
        Ok(ids)
    }
}

fn invalid(reason: DictionaryStoreInvalidity, detail: impl Into<String>) -> DictionaryStoreError {
    DictionaryStoreError::InvalidStore {
        reason,
        detail: detail.into(),
    }
}

pub(crate) fn check_minimal_metadata(
    metadata: &std::collections::BTreeMap<String, String>,
) -> Result<(), DictionaryStoreError> {
    check_metadata(metadata)?;
    let digest = metadata
        .get("canonical_sha256")
        .map(String::as_str)
        .unwrap_or("");
    if digest.len() != 64
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        return Err(invalid(
            DictionaryStoreInvalidity::CanonicalBytes,
            "canonical_sha256 must be 64 lowercase hexadecimal characters",
        ));
    }
    let dictionary_version = metadata
        .get("dictionary_version")
        .and_then(|value| value.parse::<u32>().ok())
        .ok_or_else(|| {
            invalid(
                DictionaryStoreInvalidity::CanonicalBytes,
                "dictionary_version is not an unsigned integer",
            )
        })?;
    let registry_version = metadata
        .get("registry_version")
        .and_then(|value| value.parse::<u32>().ok())
        .ok_or_else(|| {
            invalid(
                DictionaryStoreInvalidity::CanonicalBytes,
                "registry_version is not an unsigned integer",
            )
        })?;
    if dictionary_version != 1 || registry_version != 1 {
        return Err(invalid(
            DictionaryStoreInvalidity::CanonicalBytes,
            "dictionary or registry version is unsupported",
        ));
    }
    metadata
        .get("catalogue_revision")
        .and_then(|value| value.parse::<u64>().ok())
        .ok_or_else(|| {
            invalid(
                DictionaryStoreInvalidity::CanonicalBytes,
                "catalogue_revision is not an unsigned integer",
            )
        })?;
    Ok(())
}

fn build(
    path: &Path,
    dictionary: &Dictionary,
    canonical: &[u8],
) -> Result<(), DictionaryStoreError> {
    let conn = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_WRITE
            | OpenFlags::SQLITE_OPEN_CREATE
            | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    conn.execute_batch("PRAGMA journal_mode=DELETE; PRAGMA foreign_keys=ON;")?;
    initialize(&conn)?;
    populate(&conn, dictionary, canonical)
}

/// Commit the durable schema and incomplete header before population begins.
fn initialize(conn: &Connection) -> Result<(), DictionaryStoreError> {
    conn.execute_batch("BEGIN IMMEDIATE;")?;
    dictionary_schema::create_schema(conn)?;
    conn.execute(
        "INSERT INTO metadata(key,value) VALUES('format',?1),('schema_version',?2),('complete','0')",
        (dictionary_schema::FORMAT, dictionary_schema::SCHEMA_VERSION),
    )?;
    conn.execute_batch("COMMIT;")?;
    Ok(())
}

/// Populate an initialized store atomically; `complete=1` is committed last.
fn populate(
    conn: &Connection,
    dictionary: &Dictionary,
    canonical: &[u8],
) -> Result<(), DictionaryStoreError> {
    conn.execute_batch("BEGIN IMMEDIATE;")?;
    dictionary_schema::write_projection(conn, dictionary, canonical)?;
    conn.execute("UPDATE metadata SET value='1' WHERE key='complete'", [])?;
    conn.execute_batch("COMMIT;")?;
    Ok(())
}

pub(crate) fn read_metadata(
    conn: &Connection,
) -> Result<std::collections::BTreeMap<String, String>, DictionaryStoreError> {
    read_metadata_from(conn, "main")
}

pub(crate) fn read_metadata_from(
    conn: &Connection,
    schema: &str,
) -> Result<std::collections::BTreeMap<String, String>, DictionaryStoreError> {
    // `schema` is supplied only by the closed snapshot provider (`main` or
    // `pack1`..`pack3`), never from caller-controlled input.
    let schema_catalog = format!("{schema}.sqlite_schema");
    let has_table: bool = conn.query_row(
        &format!(
            "SELECT EXISTS(SELECT 1 FROM {schema_catalog} WHERE type='table' AND name='metadata')"
        ),
        [],
        |row| row.get(0),
    )?;
    if !has_table {
        return Err(invalid(
            DictionaryStoreInvalidity::Format,
            "metadata table is missing",
        ));
    }
    let has_required_columns = {
        let shape = conn.prepare(&format!("SELECT * FROM {schema}.metadata LIMIT 0"))?;
        let names = shape.column_names();
        names.iter().any(|name| name.eq_ignore_ascii_case("key"))
            && names.iter().any(|name| name.eq_ignore_ascii_case("value"))
    };
    if !has_required_columns {
        return Err(invalid(
            DictionaryStoreInvalidity::Format,
            "metadata table is missing a required key/value column",
        ));
    }
    let mut stmt = conn.prepare(&format!(
        "SELECT key,value FROM {schema}.metadata ORDER BY key"
    ))?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, Value>(0)?, r.get::<_, Value>(1)?)))?;
    let mut result = std::collections::BTreeMap::new();
    for row in rows {
        let (key, value) = row?;
        let Value::Text(key) = key else {
            return Err(invalid(
                DictionaryStoreInvalidity::Format,
                "metadata key is not stored as TEXT",
            ));
        };
        let Value::Text(value) = value else {
            let reason = match key.as_str() {
                "format" => DictionaryStoreInvalidity::Format,
                "schema_version" => DictionaryStoreInvalidity::SchemaVersion,
                "complete" => DictionaryStoreInvalidity::Incomplete,
                "canonical_sha256" | "dictionary_version" | "registry_version"
                | "catalogue_revision" => DictionaryStoreInvalidity::CanonicalBytes,
                _ => DictionaryStoreInvalidity::Format,
            };
            return Err(invalid(
                reason,
                format!("metadata value for {key} is not stored as TEXT"),
            ));
        };
        if result.insert(key.clone(), value).is_some() {
            return Err(invalid(
                DictionaryStoreInvalidity::Format,
                format!("metadata key {key:?} is repeated"),
            ));
        }
    }
    Ok(result)
}

pub(crate) fn check_metadata(
    metadata: &std::collections::BTreeMap<String, String>,
) -> Result<(), DictionaryStoreError> {
    if metadata.get("format").map(String::as_str) != Some(dictionary_schema::FORMAT) {
        return Err(invalid(
            DictionaryStoreInvalidity::Format,
            "missing or incorrect format discriminator",
        ));
    }
    if metadata.get("schema_version").map(String::as_str) != Some(dictionary_schema::SCHEMA_VERSION)
    {
        return Err(invalid(
            DictionaryStoreInvalidity::SchemaVersion,
            "unsupported schema version",
        ));
    }
    if metadata.get("complete").map(String::as_str) != Some("1") {
        return Err(invalid(
            DictionaryStoreInvalidity::Incomplete,
            "complete marker is not committed",
        ));
    }
    for key in [
        "canonical_sha256",
        "dictionary_version",
        "registry_version",
        "catalogue_revision",
    ] {
        if !metadata.contains_key(key) {
            return Err(invalid(
                DictionaryStoreInvalidity::CanonicalBytes,
                format!("missing metadata key {key}"),
            ));
        }
    }
    Ok(())
}

pub(crate) fn check_schema(conn: &Connection) -> Result<(), DictionaryStoreError> {
    let expected = Connection::open_in_memory()?;
    dictionary_schema::create_schema(&expected)?;
    if dictionary_schema::schema_sql(conn)? != dictionary_schema::schema_sql(&expected)? {
        return Err(invalid(
            DictionaryStoreInvalidity::SchemaShape,
            "schema catalog differs from the exact version-1 DDL",
        ));
    }
    Ok(())
}

pub(crate) fn read_canonical(
    conn: &Connection,
) -> Result<(Vec<u8>, Option<Vec<u8>>), DictionaryStoreError> {
    read_canonical_from(conn, "main")
}

pub(crate) fn read_canonical_from(
    conn: &Connection,
    schema: &str,
) -> Result<(Vec<u8>, Option<Vec<u8>>), DictionaryStoreError> {
    // `schema` is supplied only by the closed snapshot provider.
    let mut stmt = conn.prepare(&format!(
        "SELECT json,legacy_archive FROM {schema}.canonical_document WHERE id=1"
    ))?;
    let rows: Vec<(Value, Value)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<Result<_, _>>()?;
    if rows.len() != 1 {
        return Err(invalid(
            DictionaryStoreInvalidity::CanonicalBytes,
            "canonical_document must contain exactly one row",
        ));
    }
    let (json, legacy) = rows.into_iter().next().ok_or_else(|| {
        invalid(
            DictionaryStoreInvalidity::CanonicalBytes,
            "canonical row disappeared during read",
        )
    })?;
    let Value::Blob(json) = json else {
        return Err(invalid(
            DictionaryStoreInvalidity::CanonicalBytes,
            "canonical JSON is not stored as BLOB",
        ));
    };
    let legacy = match legacy {
        Value::Blob(bytes) => Some(bytes),
        Value::Null => None,
        _ => {
            return Err(invalid(
                DictionaryStoreInvalidity::CanonicalBytes,
                "legacy archive is neither BLOB nor NULL",
            ));
        }
    };
    Ok((json, legacy))
}

pub(crate) fn check_foreign_keys(conn: &Connection) -> Result<(), DictionaryStoreError> {
    let mut stmt = conn.prepare("PRAGMA foreign_key_check")?;
    let mut rows = stmt.query([])?;
    if rows.next()?.is_some() {
        return Err(invalid(
            DictionaryStoreInvalidity::ForeignKeys,
            "foreign_key_check reported a violation",
        ));
    }
    Ok(())
}

pub(crate) fn check_projection(
    conn: &Connection,
    dictionary: &Dictionary,
    canonical: &[u8],
) -> Result<(), DictionaryStoreError> {
    dictionary_schema::check_projection(conn, dictionary, canonical)
        .map_err(|detail| invalid(DictionaryStoreInvalidity::Projection, detail))
}

#[cfg(test)]
mod lifecycle_tests {
    use super::{
        DATABASE_FILE, DictionaryStore, DictionaryStoreError, DictionaryStoreInvalidity,
        initialize, populate,
    };
    use rusqlite::Connection;
    use std::{fs, time::SystemTime};
    use vlajmaji_lexicon::Dictionary;

    #[test]
    fn committed_incomplete_header_survives_population_failure() {
        let nonce = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("system clock is after Unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "vlajmaji-s03-incomplete-header-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&root).expect("create unique test directory");
        let database = root.join(DATABASE_FILE);
        let dictionary = Dictionary::new();
        let canonical = dictionary
            .to_json_bytes()
            .expect("serialize empty dictionary");
        let writer = Connection::open(&database).expect("open test database");
        writer
            .execute_batch("PRAGMA foreign_keys=ON;")
            .expect("enable foreign keys");
        initialize(&writer).expect("commit initialized header");

        let reader = Connection::open(&database).expect("open concurrent header reader");
        let header: (String, String, String) = reader
            .query_row(
                "SELECT format.value,schema_version.value,complete.value FROM metadata AS format JOIN metadata AS schema_version ON schema_version.key='schema_version' JOIN metadata AS complete ON complete.key='complete' WHERE format.key='format'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("read committed header before population");
        assert_eq!(
            header,
            (
                super::dictionary_schema::FORMAT.to_owned(),
                super::dictionary_schema::SCHEMA_VERSION.to_owned(),
                "0".to_owned()
            )
        );
        drop(reader);

        writer
            .execute_batch(
                "CREATE TEMP TRIGGER fail_canonical BEFORE INSERT ON canonical_document BEGIN SELECT RAISE(ABORT, 'injected population failure'); END;",
            )
            .expect("install one-test population failure");
        assert!(matches!(
            populate(&writer, &dictionary, &canonical),
            Err(DictionaryStoreError::Backend(_))
        ));
        drop(writer);

        let reader = Connection::open(&database).expect("reopen persisted header");
        let values: Vec<(String, String)> = {
            let mut statement = reader
                .prepare("SELECT key,value FROM metadata ORDER BY key")
                .expect("prepare persisted header read");
            statement
                .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .expect("query persisted header")
                .collect::<Result<_, _>>()
                .expect("collect persisted header")
        };
        drop(reader);
        assert_eq!(
            values,
            vec![
                ("complete".to_owned(), "0".to_owned()),
                (
                    "format".to_owned(),
                    super::dictionary_schema::FORMAT.to_owned()
                ),
                (
                    "schema_version".to_owned(),
                    super::dictionary_schema::SCHEMA_VERSION.to_owned()
                ),
            ]
        );
        assert!(matches!(
            DictionaryStore::open(&root),
            Err(DictionaryStoreError::InvalidStore {
                reason: DictionaryStoreInvalidity::Incomplete,
                ..
            })
        ));
        assert!(matches!(
            DictionaryStore::create(&root, &dictionary),
            Err(DictionaryStoreError::ExistingPath(_))
        ));
        fs::remove_dir_all(&root).expect("remove test-owned directory");
    }
}
