//! Immutable validated persistence for the versioned [`Dictionary`](vlajmaji_lexicon::Dictionary).
//!
//! A store owns a read-only SQLite connection and the snapshot whose canonical
//! bytes, model, schema, foreign keys, and complete relational projection passed
//! validation when it was opened. It does not migrate or repair files.

use std::{
    error::Error,
    fmt, fs, io,
    path::{Path, PathBuf},
};

use rusqlite::{Connection, OpenFlags, types::Value};
use sha2::{Digest, Sha256};
use vlajmaji_lexicon::{Dictionary, DictionaryError};

use crate::dictionary_schema::{self, DATABASE_FILE};

/// A typed reason that an existing dictionary store is not acceptable.
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

/// Failure to create or strictly open an immutable dictionary store.
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

/// A complete dictionary store held on one validated read-only SQLite snapshot.
pub struct DictionaryStore {
    path: PathBuf,
    database_path: PathBuf,
    _connection: Connection,
    dictionary: Dictionary,
    canonical_bytes: Vec<u8>,
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
    /// Validate a dictionary, build a fresh store under an existing parent, then
    /// reopen and return the complete read-only validated snapshot.
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
        Self::open(path)
    }

    /// Open an existing dictionary store without creating, migrating, or writing it.
    ///
    /// The returned borrow and SQL state refer to the read transaction pinned by
    /// the first metadata read; dropping the store releases that snapshot.
    ///
    /// # Errors
    ///
    /// Returns a typed family/version/completeness/schema/canonical/FK/projection
    /// reason or the underlying SQLite/filesystem diagnostic.
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
        check_metadata(&metadata)?;
        check_schema(&connection)?;
        let (canonical_bytes, legacy_bytes) = read_canonical(&connection)?;
        let expected_hash = metadata.get("canonical_sha256").ok_or_else(|| {
            invalid(
                DictionaryStoreInvalidity::CanonicalBytes,
                "canonical hash metadata is missing",
            )
        })?;
        let actual_hash = format!("{:x}", Sha256::digest(&canonical_bytes));
        if &actual_hash != expected_hash {
            return Err(invalid(
                DictionaryStoreInvalidity::CanonicalBytes,
                "canonical document digest does not match metadata",
            ));
        }
        let dictionary = Dictionary::from_json_bytes(&canonical_bytes)
            .map_err(DictionaryStoreError::InvalidDictionary)?;
        if dictionary.document().legacy_archive.as_deref() != legacy_bytes.as_deref() {
            return Err(invalid(
                DictionaryStoreInvalidity::CanonicalBytes,
                "canonical document legacy archive column differs from the document",
            ));
        }
        check_foreign_keys(&connection)?;
        check_projection(&connection, &dictionary, &canonical_bytes)?;
        Ok(Self {
            path,
            database_path,
            _connection: connection,
            dictionary,
            canonical_bytes,
        })
    }

    /// Borrow the complete validated model represented by this snapshot.
    #[must_use]
    pub fn dictionary(&self) -> &Dictionary {
        &self.dictionary
    }

    /// Borrow the deterministic canonical JSON bytes used to build this store.
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
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
}

fn invalid(reason: DictionaryStoreInvalidity, detail: impl Into<String>) -> DictionaryStoreError {
    DictionaryStoreError::InvalidStore {
        reason,
        detail: detail.into(),
    }
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

fn read_metadata(
    conn: &Connection,
) -> Result<std::collections::BTreeMap<String, String>, DictionaryStoreError> {
    let has_table: bool = conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_schema WHERE type='table' AND name='metadata')",
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
        let shape = conn.prepare("SELECT * FROM metadata LIMIT 0")?;
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
    let mut stmt = conn.prepare("SELECT key,value FROM metadata ORDER BY key")?;
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
        result.insert(key, value);
    }
    Ok(result)
}

fn check_metadata(
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

fn check_schema(conn: &Connection) -> Result<(), DictionaryStoreError> {
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

fn read_canonical(conn: &Connection) -> Result<(Vec<u8>, Option<Vec<u8>>), DictionaryStoreError> {
    let mut stmt = conn.prepare("SELECT json,legacy_archive FROM canonical_document WHERE id=1")?;
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

fn check_foreign_keys(conn: &Connection) -> Result<(), DictionaryStoreError> {
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

fn check_projection(
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
