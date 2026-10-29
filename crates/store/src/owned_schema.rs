//! Private version-1 SQLite schema for authoritative owned dictionaries.
use rusqlite::Connection;

pub(crate) const FILE: &str = "owned.sqlite";
pub(crate) const APPLICATION_ID: i32 = 0x564c4f57;
pub(crate) const SCHEMA_VERSION: i32 = 1;
pub(crate) const CURATION_SCHEMA_VERSION: i32 = 2;
pub(crate) const MODEL_VERSION: i32 = 1;
pub(crate) const REGISTRY_VERSION: i32 = 1;
pub(crate) const SECTIONS: [&str; 24] = [
    "registry.sources",
    "registry.contributors",
    "registry.distributors",
    "registry.aliases",
    "registry.lineage",
    "registry.snapshots",
    "registry.occurrences",
    "captures",
    "metadata.code_notices",
    "metadata.collections",
    "metadata.components",
    "metadata.releases",
    "metadata.entries",
    "metadata.assertions",
    "metadata.contexts",
    "metadata.frames",
    "metadata.forms",
    "metadata.etymologies",
    "metadata.relations",
    "metadata.owned_revisions",
    "metadata.editorial_senses",
    "metadata.computations",
    "metadata.annotations",
    "metadata.extensions",
];

/// Configure connection durability before the caller begins the atomic create transaction.
pub(crate) fn configure(connection: &Connection) -> Result<(), rusqlite::Error> {
    connection.execute_batch(
        "PRAGMA journal_mode=DELETE; PRAGMA synchronous=FULL; PRAGMA foreign_keys=ON;",
    )
}

/// Create schema and genesis-independent header inside the caller's transaction.
pub(crate) fn initialize(connection: &Connection) -> Result<(), rusqlite::Error> {
    connection.execute_batch(&format!(
        "PRAGMA application_id={APPLICATION_ID};
         PRAGMA user_version={SCHEMA_VERSION};
         CREATE TABLE owned_meta(singleton INTEGER PRIMARY KEY CHECK(singleton=1), family TEXT NOT NULL, schema_version INTEGER NOT NULL, model_version INTEGER NOT NULL, registry_version INTEGER NOT NULL, source_id TEXT NOT NULL, head_revision TEXT NOT NULL, catalogue_revision INTEGER NOT NULL) STRICT;
         CREATE TABLE owned_records(section TEXT NOT NULL, ordinal INTEGER NOT NULL CHECK(ordinal>=0), payload BLOB NOT NULL, PRIMARY KEY(section,ordinal)) STRICT;
         CREATE TABLE owned_commits(revision_id TEXT NOT NULL PRIMARY KEY, parent_revision TEXT, sequence INTEGER NOT NULL UNIQUE CHECK(sequence>=0), state_sha256 TEXT NOT NULL) STRICT;
         CREATE TABLE owned_commit_sections(revision_id TEXT NOT NULL REFERENCES owned_commits(revision_id), section TEXT NOT NULL, length INTEGER NOT NULL CHECK(length>=0), PRIMARY KEY(revision_id,section)) STRICT;
         INSERT INTO owned_meta VALUES(1,'vlajmaji-owned',{SCHEMA_VERSION},{MODEL_VERSION},{REGISTRY_VERSION},'','',0);"
    ))
}

/// Add the version-two curation sidecar inside the caller's transaction.
pub(crate) fn initialize_curation(connection: &Connection) -> Result<(), rusqlite::Error> {
    connection.execute_batch(
        "CREATE TABLE owned_context_pins(ordinal INTEGER PRIMARY KEY CHECK(ordinal>=0), id TEXT NOT NULL UNIQUE, payload BLOB NOT NULL) STRICT;
         CREATE TABLE owned_curation_records(ordinal INTEGER PRIMARY KEY CHECK(ordinal>=0), id TEXT NOT NULL UNIQUE, payload BLOB NOT NULL) STRICT;
         CREATE TABLE owned_curation_revisions(revision_id TEXT PRIMARY KEY NOT NULL REFERENCES owned_commits(revision_id), payload BLOB NOT NULL) STRICT;
         CREATE TABLE owned_curation_prefixes(revision_id TEXT PRIMARY KEY NOT NULL REFERENCES owned_commits(revision_id), contexts_length INTEGER NOT NULL CHECK(contexts_length>=0), records_length INTEGER NOT NULL CHECK(records_length>=0), curation_sha256 TEXT NOT NULL) STRICT;",
    )
}
