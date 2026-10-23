use std::{
    collections::BTreeSet,
    fmt,
    path::{Path, PathBuf},
};

use rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha256};
use vlajmaji_lexicon::FieldPresence;

use crate::{
    EdgeHit, LexicalStore, PresenceState, SearchQuery, SearchResult, StoreError, archive_sha256,
    directory_bytes, edge_hits, empty_result, record_id, require_fresh, role_name,
    validate_archive,
};

const SCHEMA_VERSION: &str = "1";

/// A synchronous SQLite/FTS5 database in a caller-owned directory.
pub struct SqliteStore {
    path: PathBuf,
    connection: Connection,
}

impl fmt::Debug for SqliteStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SqliteStore")
            .field("path", &self.path)
            .field("native_index", &"SQLite/FTS5")
            .finish_non_exhaustive()
    }
}

impl SqliteStore {
    /// Builds a new native disk index from a valid accepted archive.
    ///
    /// The supplied directory must not already exist. The archive is validated
    /// before a database is populated; an interrupted build remains visibly
    /// incomplete and is rejected by [`Self::open`].
    pub fn create(path: impl AsRef<Path>, archive: &[u8]) -> Result<Self, StoreError> {
        let path = path.as_ref();
        require_fresh(path)?;
        let lexicon = validate_archive(archive)?;
        std::fs::create_dir_all(path).map_err(StoreError::Io)?;
        let db_path = path.join("index.sqlite3");
        let mut connection =
            Connection::open(&db_path).map_err(|error| StoreError::Backend(error.to_string()))?;
        connection.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             CREATE TABLE metadata(key TEXT PRIMARY KEY, value TEXT NOT NULL);
             CREATE TABLE archive(data BLOB NOT NULL);
             CREATE TABLE records(id TEXT PRIMARY KEY, word TEXT, definition TEXT, ascii_lower TEXT);
             CREATE INDEX records_word_idx ON records(word COLLATE BINARY);
             CREATE TABLE forms(record_id TEXT NOT NULL, value TEXT NOT NULL, role TEXT NOT NULL);
             CREATE INDEX forms_lookup_idx ON forms(value COLLATE BINARY, role);
             CREATE TABLE fields(record_id TEXT NOT NULL, name TEXT NOT NULL, state TEXT NOT NULL, raw_value TEXT NOT NULL);
             CREATE INDEX fields_lookup_idx ON fields(name, state, raw_value COLLATE BINARY);
             CREATE INDEX fields_record_idx ON fields(record_id, name);
             CREATE TABLE edges(edge_id TEXT NOT NULL, origin_id TEXT NOT NULL, target_id TEXT, target_spelling TEXT NOT NULL, byte_start INTEGER NOT NULL, byte_end INTEGER NOT NULL, rule_version TEXT NOT NULL);
             CREATE INDEX edges_origin_idx ON edges(origin_id);
             CREATE TABLE proposals(id TEXT PRIMARY KEY, spelling TEXT NOT NULL);
             CREATE INDEX proposals_spelling_idx ON proposals(spelling COLLATE BINARY);
             CREATE VIRTUAL TABLE definition_fts USING fts5(body, record_id UNINDEXED, tokenize='trigram case_sensitive 1');
             INSERT INTO metadata VALUES ('schema_version', '1');
             INSERT INTO metadata VALUES ('complete', 'false');",
        )
        .map_err(|error| StoreError::Backend(error.to_string()))?;

        let tx = connection
            .transaction()
            .map_err(|error| StoreError::Backend(error.to_string()))?;
        tx.execute("INSERT INTO archive(data) VALUES (?1)", [archive])
            .map_err(|error| StoreError::Backend(error.to_string()))?;
        tx.execute(
            "INSERT INTO metadata VALUES ('archive_sha256', ?1)",
            [archive_sha256(archive)],
        )
        .map_err(|error| StoreError::Backend(error.to_string()))?;

        let mut record_ids = Vec::with_capacity(lexicon.records().len());
        for record in lexicon.records() {
            let id = record_id(record.identity());
            record_ids.push(id.clone());
            let definition = record.definition();
            let lower = definition.map(str::to_ascii_lowercase);
            tx.execute(
                "INSERT INTO records(id,word,definition,ascii_lower) VALUES (?1,?2,?3,?4)",
                params![id, record.word(), definition, lower],
            )
            .map_err(|error| StoreError::Backend(error.to_string()))?;
            if let Some(body) = lower {
                tx.execute(
                    "INSERT INTO definition_fts(body,record_id) VALUES (?1,?2)",
                    params![body, id],
                )
                .map_err(|error| StoreError::Backend(error.to_string()))?;
            }
        }
        for form in lexicon.forms() {
            tx.execute(
                "INSERT INTO forms(record_id,value,role) VALUES (?1,?2,?3)",
                params![
                    record_id(form.source_record()),
                    form.form(),
                    role_name(form.role())
                ],
            )
            .map_err(|error| StoreError::Backend(error.to_string()))?;
        }
        let mut field_names: BTreeSet<String> = [
            "notes".to_owned(),
            "selmaho".to_owned(),
            "unknown".to_owned(),
        ]
        .into_iter()
        .collect();
        for record in lexicon.records() {
            field_names.extend(record.fields().iter().map(|field| field.name().to_owned()));
        }
        for (record, id) in lexicon.records().iter().zip(&record_ids) {
            for name in &field_names {
                let (state, raw) = match record.field(name) {
                    FieldPresence::Missing => continue,
                    FieldPresence::Null => ("null", "null"),
                    FieldPresence::Value(value) => ("value", value),
                    _ => ("value", ""),
                };
                tx.execute(
                    "INSERT INTO fields(record_id,name,state,raw_value) VALUES (?1,?2,?3,?4)",
                    params![id, name, state, raw],
                )
                .map_err(|error| StoreError::Backend(error.to_string()))?;
            }
        }
        for reference in lexicon.cross_references() {
            for hit in edge_hits(reference) {
                let edge_id = edge_id(&hit);
                tx.execute(
                    "INSERT INTO edges(edge_id,origin_id,target_id,target_spelling,byte_start,byte_end,rule_version) VALUES (?1,?2,?3,?4,?5,?6,?7)",
                    params![edge_id, hit.origin_id, hit.target_id, hit.target_spelling, hit.byte_start as i64, hit.byte_end as i64, hit.rule_version],
                )
                .map_err(|error| StoreError::Backend(error.to_string()))?;
            }
        }
        for proposal in lexicon.proposals() {
            tx.execute(
                "INSERT INTO proposals(id,spelling) VALUES (?1,?2)",
                params![proposal.id(), proposal.spelling()],
            )
            .map_err(|error| StoreError::Backend(error.to_string()))?;
        }
        tx.execute("UPDATE metadata SET value='true' WHERE key='complete'", [])
            .map_err(|error| StoreError::Backend(error.to_string()))?;
        tx.commit()
            .map_err(|error| StoreError::Backend(error.to_string()))?;
        connection
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")
            .map_err(|error| StoreError::Backend(error.to_string()))?;
        Ok(Self {
            path: path.to_owned(),
            connection,
        })
    }

    /// Opens an initialized database after verifying its schema and archive hash.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let path = path.as_ref();
        let db_path = path.join("index.sqlite3");
        let connection = Connection::open_with_flags(
            &db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .map_err(|error| StoreError::InvalidStore(error.to_string()))?;
        let version: Option<String> = connection
            .query_row(
                "SELECT value FROM metadata WHERE key='schema_version'",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| StoreError::InvalidStore(error.to_string()))?;
        let complete: Option<String> = connection
            .query_row(
                "SELECT value FROM metadata WHERE key='complete'",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| StoreError::InvalidStore(error.to_string()))?;
        if version.as_deref() != Some(SCHEMA_VERSION) || complete.as_deref() != Some("true") {
            return Err(StoreError::InvalidStore(
                "schema mismatch or incomplete build marker".into(),
            ));
        }
        let bytes = archive_from_connection(&connection)?;
        let expected: String = connection
            .query_row(
                "SELECT value FROM metadata WHERE key='archive_sha256'",
                [],
                |row| row.get(0),
            )
            .map_err(|error| StoreError::InvalidStore(error.to_string()))?;
        if archive_sha256(&bytes) != expected {
            return Err(StoreError::InvalidStore(
                "canonical archive hash mismatch".into(),
            ));
        }
        validate_archive(&bytes)?;
        Ok(Self {
            path: path.to_owned(),
            connection,
        })
    }
}

fn archive_from_connection(connection: &Connection) -> Result<Vec<u8>, StoreError> {
    connection
        .query_row("SELECT data FROM archive LIMIT 1", [], |row| row.get(0))
        .map_err(|error| StoreError::InvalidStore(error.to_string()))
}

fn edge_id(hit: &EdgeHit) -> String {
    let mut h = Sha256::new();
    h.update(hit.origin_id.as_bytes());
    h.update(hit.target_id.as_deref().unwrap_or("unresolved").as_bytes());
    h.update(hit.byte_start.to_be_bytes());
    h.update(hit.byte_end.to_be_bytes());
    format!("{:x}", h.finalize())
}

impl LexicalStore for SqliteStore {
    fn search(&self, query: &SearchQuery) -> Result<SearchResult, StoreError> {
        let mut result = empty_result("sqlite-parameterized", None);
        match query {
            SearchQuery::ExactSpelling(spelling) => {
                let mut stmt = self
                    .connection
                    .prepare("SELECT id FROM records WHERE word=?1 COLLATE BINARY")
                    .map_err(sql_error)?;
                for row in stmt
                    .query_map([spelling], |r| r.get::<_, String>(0))
                    .map_err(sql_error)?
                {
                    result.record_ids.insert(row.map_err(sql_error)?);
                }
            }
            SearchQuery::DefinitionContains(term) => {
                let needle = term.to_ascii_lowercase();
                if needle.is_empty() {
                    return Ok(result);
                }
                let chars = needle.chars().count();
                if term.contains('\0') || chars < 3 {
                    result.path = "sqlite-native-scan-fallback".into();
                    result.fallback_reason = Some(if term.contains('\0') {
                        "NUL is unsupported by FTS5 MATCH".into()
                    } else {
                        format!("query has {chars} Unicode scalar values; trigram needs at least 3")
                    });
                    result.candidates = self
                        .connection
                        .query_row(
                            "SELECT COUNT(*) FROM records WHERE instr(ascii_lower,?1)>0",
                            [&needle],
                            |r| r.get::<_, i64>(0),
                        )
                        .map_err(sql_error)? as usize;
                    let mut stmt = self
                        .connection
                        .prepare("SELECT id FROM records WHERE instr(ascii_lower,?1)>0")
                        .map_err(sql_error)?;
                    for row in stmt
                        .query_map([needle], |r| r.get::<_, String>(0))
                        .map_err(sql_error)?
                    {
                        result.record_ids.insert(row.map_err(sql_error)?);
                    }
                } else {
                    let quoted = format!("\"{}\"", needle.replace('"', "\"\""));
                    let mut stmt = self
                        .connection
                        .prepare("SELECT record_id FROM definition_fts WHERE body MATCH ?1")
                        .map_err(sql_error)?;
                    let rows = stmt
                        .query_map([quoted], |r| r.get::<_, String>(0))
                        .map_err(sql_error)?;
                    let mut candidates = Vec::new();
                    for row in rows {
                        candidates.push(row.map_err(sql_error)?);
                    }
                    result.candidates = candidates.len();
                    let mut exact = self
                        .connection
                        .prepare("SELECT 1 FROM records WHERE id=?1 AND instr(ascii_lower,?2)>0")
                        .map_err(sql_error)?;
                    for id in candidates {
                        if exact.exists(params![id, needle]).map_err(sql_error)? {
                            result.record_ids.insert(id);
                        }
                    }
                    result.path = "sqlite-fts5-trigram-case-sensitive-quoted-postfiltered".into();
                }
            }
            SearchQuery::Form { value, role } => {
                let mut stmt = if role.is_some() {
                    self.connection.prepare("SELECT DISTINCT record_id FROM forms WHERE value=?1 COLLATE BINARY AND role=?2")
                        .map_err(sql_error)?
                } else {
                    self.connection
                        .prepare(
                            "SELECT DISTINCT record_id FROM forms WHERE value=?1 COLLATE BINARY",
                        )
                        .map_err(sql_error)?
                };
                if let Some(role) = role {
                    for row in stmt
                        .query_map(params![value, role_name(*role)], |r| r.get::<_, String>(0))
                        .map_err(sql_error)?
                    {
                        result.record_ids.insert(row.map_err(sql_error)?);
                    }
                } else {
                    for row in stmt
                        .query_map([value], |r| r.get::<_, String>(0))
                        .map_err(sql_error)?
                    {
                        result.record_ids.insert(row.map_err(sql_error)?);
                    }
                }
            }
            SearchQuery::CrossReferences { spelling, depth } => {
                if !(1..=2).contains(depth) {
                    return Ok(result);
                }
                let mut stmt = self
                    .connection
                    .prepare(
                        "WITH RECURSIVE walk(target_id,edge_id,depth) AS (
                       SELECT e.target_id,e.edge_id,1
                       FROM edges e JOIN records r ON r.id=e.origin_id
                       WHERE r.word=?1 COLLATE BINARY
                       UNION ALL
                       SELECT e.target_id,e.edge_id,w.depth+1
                       FROM edges e JOIN walk w ON e.origin_id=w.target_id
                       WHERE w.depth<?2
                     ) SELECT DISTINCT target_id,edge_id FROM walk",
                    )
                    .map_err(sql_error)?;
                let rows = stmt
                    .query_map(params![spelling, *depth as i64], |r| {
                        Ok((r.get::<_, Option<String>>(0)?, r.get::<_, String>(1)?))
                    })
                    .map_err(sql_error)?;
                let mut edge_ids = BTreeSet::new();
                for row in rows {
                    let (id, edge) = row.map_err(sql_error)?;
                    if let Some(id) = id {
                        result.record_ids.insert(id);
                    }
                    edge_ids.insert(edge);
                }
                for edge in edge_ids {
                    let mut q = self.connection.prepare("SELECT origin_id,target_id,target_spelling,byte_start,byte_end,rule_version FROM edges WHERE edge_id=?1")
                        .map_err(sql_error)?;
                    let rows = q
                        .query_map([edge], |r| {
                            Ok(EdgeHit {
                                origin_id: r.get(0)?,
                                target_id: r.get(1)?,
                                target_spelling: r.get(2)?,
                                byte_start: r.get::<_, i64>(3)? as usize,
                                byte_end: r.get::<_, i64>(4)? as usize,
                                rule_version: r.get(5)?,
                            })
                        })
                        .map_err(sql_error)?;
                    for row in rows {
                        result.edges.insert(row.map_err(sql_error)?);
                    }
                }
                result.path = "sqlite-recursive-cte".into();
            }
            SearchQuery::Field {
                name,
                state,
                raw_value,
            } => {
                if *state == PresenceState::Missing {
                    if raw_value.is_some() {
                        return Ok(result);
                    }
                    let mut stmt = self.connection.prepare(
                        "SELECT id FROM records r WHERE NOT EXISTS (SELECT 1 FROM fields f WHERE f.record_id=r.id AND f.name=?1 COLLATE BINARY)",
                    ).map_err(sql_error)?;
                    for row in stmt
                        .query_map([name], |r| r.get::<_, String>(0))
                        .map_err(sql_error)?
                    {
                        result.record_ids.insert(row.map_err(sql_error)?);
                    }
                } else {
                    let encoded_state = match state {
                        PresenceState::Missing => "missing",
                        PresenceState::Null => "null",
                        PresenceState::Value => "value",
                    };
                    let mut stmt = if raw_value.is_some() {
                        self.connection.prepare("SELECT record_id FROM fields WHERE name=?1 COLLATE BINARY AND state=?2 AND raw_value=?3 COLLATE BINARY")
                            .map_err(sql_error)?
                    } else {
                        self.connection.prepare("SELECT record_id FROM fields WHERE name=?1 COLLATE BINARY AND state=?2")
                            .map_err(sql_error)?
                    };
                    if let Some(raw) = raw_value {
                        for row in stmt
                            .query_map(params![name, encoded_state, raw], |r| r.get::<_, String>(0))
                            .map_err(sql_error)?
                        {
                            result.record_ids.insert(row.map_err(sql_error)?);
                        }
                    } else {
                        for row in stmt
                            .query_map(params![name, encoded_state], |r| r.get::<_, String>(0))
                            .map_err(sql_error)?
                        {
                            result.record_ids.insert(row.map_err(sql_error)?);
                        }
                    }
                }
            }
            SearchQuery::ProposalExact(spelling) => {
                let mut stmt = self
                    .connection
                    .prepare("SELECT id FROM proposals WHERE spelling=?1 COLLATE BINARY")
                    .map_err(sql_error)?;
                for row in stmt
                    .query_map([spelling], |r| r.get::<_, String>(0))
                    .map_err(sql_error)?
                {
                    result.record_ids.insert(row.map_err(sql_error)?);
                }
            }
        }
        if result.candidates == 0 {
            result.candidates = result.record_ids.len();
        }
        Ok(result)
    }

    fn archive_bytes(&self) -> Result<Vec<u8>, StoreError> {
        archive_from_connection(&self.connection)
    }

    fn flush(&self) -> Result<(), StoreError> {
        self.connection
            .execute_batch("PRAGMA wal_checkpoint(TRUNCATE); PRAGMA optimize;")
            .map_err(sql_error)
    }

    fn logical_bytes(&self) -> Result<u64, StoreError> {
        directory_bytes(&self.path)
    }
}

fn sql_error(error: rusqlite::Error) -> StoreError {
    StoreError::Backend(error.to_string())
}
