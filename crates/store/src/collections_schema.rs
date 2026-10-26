//! Private catalogue schema and its minimal family/version checks.

use std::{
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
};

use rusqlite::{Connection, OpenFlags, OptionalExtension, params};

use crate::collections::CollectionError;

pub(crate) const CATALOGUE_FILE: &str = "catalogue.sqlite3";
const FORMAT: &str = "vlajmaji-collection-catalogue";
const VERSION: &str = "1";

pub(crate) fn database_path(root: &Path) -> PathBuf {
    root.join(CATALOGUE_FILE)
}

pub(crate) fn open_readonly(root: &Path) -> Result<Connection, CollectionError> {
    let root_meta = fs::symlink_metadata(root).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            CollectionError::MissingCatalogue(root.to_path_buf())
        } else {
            CollectionError::ArtifactIoSource {
                path: Some(root.to_path_buf()),
                source: Box::new(error),
            }
        }
    })?;
    if !root_meta.is_dir() || root_meta.file_type().is_symlink() {
        return Err(CollectionError::MissingCatalogue(root.to_path_buf()));
    }
    let path = database_path(root);
    let db_meta = fs::symlink_metadata(&path).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            CollectionError::MissingCatalogue(root.to_path_buf())
        } else {
            CollectionError::ArtifactIoSource {
                path: Some(path.clone()),
                source: Box::new(error),
            }
        }
    })?;
    if !db_meta.is_file() || db_meta.file_type().is_symlink() {
        return Err(CollectionError::MissingCatalogue(root.to_path_buf()));
    }
    let connection = Connection::open_with_flags(
        &path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|error| CollectionError::ArtifactIoSource {
        path: Some(path.clone()),
        source: Box::new(error),
    })?;
    connection
        .execute_batch("BEGIN DEFERRED")
        .map_err(|error| CollectionError::ArtifactIoSource {
            path: Some(path.clone()),
            source: Box::new(error),
        })?;
    check_family(&connection, &path)?;
    Ok(connection)
}

pub(crate) fn initialize_exclusive(root: &Path) -> Result<Connection, CollectionError> {
    match fs::symlink_metadata(root) {
        Ok(_) => {
            let db = database_path(root);
            if fs::symlink_metadata(&db).is_ok() {
                return Err(CollectionError::InvalidRequest(
                    "catalogue already exists; use the existing catalogue operation".into(),
                ));
            }
            return Err(CollectionError::OccupiedDestination(root.to_path_buf()));
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(CollectionError::ArtifactIoSource {
                path: Some(root.to_path_buf()),
                source: Box::new(error),
            });
        }
    }
    create_private_dir(root)?;
    if let Some(parent) = root.parent() {
        sync_dir(parent)?;
    }
    let path = database_path(root);
    let mut file_options = OpenOptions::new();
    file_options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        file_options.mode(0o600);
    }
    let mut file = file_options
        .open(&path)
        .map_err(|error| CollectionError::ArtifactIoSource {
            path: Some(path.clone()),
            source: Box::new(error),
        })?;
    file.write_all(&[])
        .and_then(|_| file.sync_all())
        .map_err(|error| CollectionError::ArtifactIoSource {
            path: Some(path.clone()),
            source: Box::new(error),
        })?;
    drop(file);
    let connection =
        Connection::open(&path).map_err(|error| CollectionError::ArtifactIoSource {
            path: Some(path.clone()),
            source: Box::new(error),
        })?;
    connection
        .busy_timeout(std::time::Duration::ZERO)
        .map_err(|error| CollectionError::ArtifactIoSource {
            path: Some(path.clone()),
            source: Box::new(error),
        })?;
    let host = std::env::consts::OS;
    let path_encoding = if cfg!(unix) { "unix-bytes" } else { "utf8" };
    let ddl = format!(
        "PRAGMA journal_mode=DELETE;
         PRAGMA synchronous=FULL;
         PRAGMA foreign_keys=ON;
         BEGIN IMMEDIATE;
         CREATE TABLE catalogue_meta(key TEXT PRIMARY KEY, value TEXT NOT NULL) STRICT;
         CREATE TABLE registrations(
           pack_id TEXT NOT NULL,
           release_sha256 TEXT NOT NULL,
           payload_kind TEXT NOT NULL,
           seal_json BLOB NOT NULL,
           descriptor_json BLOB NOT NULL,
           summary_json BLOB NOT NULL,
           input_paths_json BLOB NOT NULL,
           PRIMARY KEY(pack_id, release_sha256)
         ) STRICT;
         CREATE TABLE registration_locations(
           pack_id TEXT NOT NULL,
           release_sha256 TEXT NOT NULL,
           input_paths_json BLOB NOT NULL,
           PRIMARY KEY(pack_id, release_sha256, input_paths_json),
           FOREIGN KEY(pack_id, release_sha256) REFERENCES registrations(pack_id, release_sha256)
         ) STRICT;
         CREATE TABLE generations(
           generation_id TEXT PRIMARY KEY,
           content_id TEXT NOT NULL,
           canonical_sha256 TEXT NOT NULL,
           store_path BLOB NOT NULL,
           members_json BLOB NOT NULL,
           local_revision INTEGER NOT NULL
         ) STRICT;
         CREATE TABLE installed_heads(
           pack_id TEXT PRIMARY KEY,
           release_sha256 TEXT NOT NULL,
           payload_kind TEXT NOT NULL,
           generation_id TEXT NOT NULL REFERENCES generations(generation_id)
         ) STRICT;
         CREATE TABLE attempts(
           attempt_id TEXT PRIMARY KEY,
           operation TEXT NOT NULL,
           status TEXT NOT NULL,
           prior_revision INTEGER NOT NULL,
           detail TEXT NOT NULL
         ) STRICT;
         INSERT INTO catalogue_meta(key,value) VALUES
           ('format','vlajmaji-collection-catalogue'),
           ('schema_version','1'),
           ('local_revision','0'),
           ('complete','true'),
           ('path_host','{host}'),
           ('path_encoding','{path_encoding}'),
           ('catalogue_id', lower(hex(randomblob(16))));
         COMMIT;"
    );
    connection
        .execute_batch(&ddl)
        .map_err(|error| CollectionError::ArtifactIoSource {
            path: Some(path.clone()),
            source: Box::new(error),
        })?;
    sync_file(&path)?;
    sync_dir(root)?;
    check_family(&connection, &path)?;
    Ok(connection)
}

pub(crate) fn open_writable(root: &Path) -> Result<Connection, CollectionError> {
    let root_meta = fs::symlink_metadata(root).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            CollectionError::MissingCatalogue(root.to_path_buf())
        } else {
            CollectionError::ArtifactIoSource {
                path: Some(root.to_path_buf()),
                source: Box::new(error),
            }
        }
    })?;
    if !root_meta.is_dir() || root_meta.file_type().is_symlink() {
        return Err(CollectionError::OccupiedDestination(root.to_path_buf()));
    }
    let path = database_path(root);
    let meta = fs::symlink_metadata(&path).map_err(|error| {
        if error.kind() == io::ErrorKind::NotFound {
            CollectionError::MissingCatalogue(root.to_path_buf())
        } else {
            CollectionError::ArtifactIoSource {
                path: Some(path.clone()),
                source: Box::new(error),
            }
        }
    })?;
    if !meta.is_file() || meta.file_type().is_symlink() {
        return Err(CollectionError::OccupiedDestination(path));
    }
    let connection = Connection::open_with_flags(
        &path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|error| CollectionError::ArtifactIoSource {
        path: Some(path.clone()),
        source: Box::new(error),
    })?;
    connection
        .busy_timeout(std::time::Duration::ZERO)
        .map_err(|error| CollectionError::ArtifactIoSource {
            path: Some(path.clone()),
            source: Box::new(error),
        })?;
    connection
        .pragma_update(None, "foreign_keys", "ON")
        .map_err(|error| CollectionError::ArtifactIoSource {
            path: Some(path.clone()),
            source: Box::new(error),
        })?;
    connection
        .pragma_update(None, "synchronous", "FULL")
        .map_err(|error| CollectionError::ArtifactIoSource {
            path: Some(path.clone()),
            source: Box::new(error),
        })?;
    check_family(&connection, &path)?;
    Ok(connection)
}

pub(crate) fn check_family(connection: &Connection, path: &Path) -> Result<(), CollectionError> {
    let get = |key: &str| -> Result<Option<String>, CollectionError> {
        connection
            .query_row(
                "SELECT value FROM catalogue_meta WHERE key=?1",
                [key],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| CollectionError::ArtifactIoSource {
                path: Some(path.to_path_buf()),
                source: Box::new(error),
            })
    };
    if get("format")?.as_deref() != Some(FORMAT) {
        return Err(CollectionError::UnsupportedVersion(
            "catalogue family marker is missing or different".into(),
        ));
    }
    if get("schema_version")?.as_deref() != Some(VERSION) {
        return Err(CollectionError::UnsupportedVersion(
            "catalogue schema_version is not supported".into(),
        ));
    }
    if get("complete")?.as_deref() != Some("true") {
        return Err(CollectionError::UnsupportedVersion(
            "catalogue is not marked complete".into(),
        ));
    }
    let host: Option<String> = connection
        .query_row(
            "SELECT value FROM catalogue_meta WHERE key='path_host'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| CollectionError::ArtifactIoSource {
            path: Some(path.to_path_buf()),
            source: Box::new(error),
        })?;
    let expected_host = if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unsupported"
    };
    if host.as_deref() != Some(expected_host) {
        return Err(CollectionError::UnsupportedVersion(
            "catalogue native path host tag does not match this runtime".into(),
        ));
    }
    let encoding: Option<String> = connection
        .query_row(
            "SELECT value FROM catalogue_meta WHERE key='path_encoding'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|error| CollectionError::ArtifactIoSource {
            path: Some(path.to_path_buf()),
            source: Box::new(error),
        })?;
    let expected_encoding = if cfg!(unix) { "unix-bytes" } else { "utf8" };
    if encoding.as_deref() != Some(expected_encoding) {
        return Err(CollectionError::UnsupportedVersion(
            "catalogue native path encoding is not supported by this runtime".into(),
        ));
    }
    Ok(())
}

pub(crate) fn local_revision(connection: &Connection, path: &Path) -> Result<u64, CollectionError> {
    let raw: String = connection
        .query_row(
            "SELECT value FROM catalogue_meta WHERE key='local_revision'",
            [],
            |row| row.get(0),
        )
        .map_err(|error| CollectionError::ArtifactIoSource {
            path: Some(path.to_path_buf()),
            source: Box::new(error),
        })?;
    raw.parse().map_err(|_| {
        CollectionError::UnsupportedVersion("catalogue local_revision is malformed".into())
    })
}

pub(crate) fn sync_file(path: &Path) -> Result<(), CollectionError> {
    fs::File::open(path)
        .and_then(|file| file.sync_all())
        .map_err(|error| CollectionError::ArtifactIoSource {
            path: Some(path.to_path_buf()),
            source: Box::new(error),
        })
}
pub(crate) fn sync_dir(path: &Path) -> Result<(), CollectionError> {
    fs::File::open(path)
        .and_then(|file| file.sync_all())
        .map_err(|error| CollectionError::ArtifactIoSource {
            path: Some(path.to_path_buf()),
            source: Box::new(error),
        })
}
fn create_private_dir(path: &Path) -> Result<(), CollectionError> {
    let mut builder = fs::DirBuilder::new();
    builder.recursive(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt;
        builder.mode(0o700);
    }
    builder
        .create(path)
        .map_err(|error| CollectionError::ArtifactIoSource {
            path: Some(path.to_path_buf()),
            source: Box::new(error),
        })
}

pub(crate) struct RegistrationInsert<'a> {
    pub(crate) pack_id: &'a str,
    pub(crate) release_sha256: &'a str,
    pub(crate) payload_kind: &'a str,
    pub(crate) seal_json: &'a [u8],
    pub(crate) descriptor_json: &'a [u8],
    pub(crate) summary_json: &'a [u8],
    pub(crate) input_paths_json: &'a [u8],
}

pub(crate) fn insert_registration(
    connection: &Connection,
    record: RegistrationInsert<'_>,
) -> Result<(), rusqlite::Error> {
    connection.execute(
        "INSERT INTO registrations(pack_id,release_sha256,payload_kind,seal_json,descriptor_json,summary_json,input_paths_json) VALUES(?1,?2,?3,?4,?5,?6,?7)",
        params![record.pack_id, record.release_sha256, record.payload_kind, record.seal_json, record.descriptor_json, record.summary_json, record.input_paths_json],
    )?;
    connection.execute(
        "INSERT INTO registration_locations(pack_id,release_sha256,input_paths_json) VALUES(?1,?2,?3)",
        params![record.pack_id, record.release_sha256, record.input_paths_json],
    )?;
    Ok(())
}

pub(crate) fn catalogue_identity(connection: &Connection) -> Result<String, rusqlite::Error> {
    connection.query_row(
        "SELECT value FROM catalogue_meta WHERE key='catalogue_id'",
        [],
        |row| row.get(0),
    )
}

pub(crate) fn update_revision(
    connection: &Connection,
    revision: u64,
) -> Result<(), rusqlite::Error> {
    connection.execute(
        "UPDATE catalogue_meta SET value=?1 WHERE key='local_revision'",
        [revision.to_string()],
    )?;
    Ok(())
}
